use std::{cell::RefCell, collections::HashMap, fmt::Display};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;

use crate::{lexer::token::{Kwrd, Op, Tkn, TknType}, parser::tokens};

use super::{functions::FullFunctionDefinition, operators::{BinOp, OpAssoc, UnOp, OPERATOR_INFO_MAP}, stmt::Stmt, tokens::{expect_token, is_expected_token}, ExprBump, ParserError};

pub type Expr<'e, 't> = Expression<'e, 't>;
#[derive(Clone, Debug)]
pub struct Expression<'e, 't> {
    pub expr_data: &'e ExprData<'e>,
    pub expr_type: ExprTypeCons<'t>,
}

impl<'e, 't> Expression<'e, 't> {
    pub fn map(mut self, expr_bump: &'e ExprBump, f: impl FnOnce(&'e ExprData<'e>) -> ExprData<'e>) -> Self {
        self.expr_data = expr_bump.alloc(f(self.expr_data));
        self
    }
}

pub type ExprData<'ed> = ExpressionData<'ed>;

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionData<'ed> {
    Identifier(String),
    Literal(Lit),
    Conditional{ 
        conds: Vec<&'ed ExprData<'ed>>, 
        bodies: Vec<&'ed Stmt<'ed>> 
    },
    Function {
        name: String,
        left_args: Vec<&'ed ExprData<'ed>>,
        right_args: Vec<&'ed ExprData<'ed>>
    },
    BinaryOp(BinOp, &'ed ExprData<'ed>, &'ed ExprData<'ed>),
    UnaryOp(UnOp, &'ed ExprData<'ed>),
    Array(Vec<&'ed ExprData<'ed>>),
    AmbiguousArray(&'ed ExprData<'ed>)
}

pub type ExprTypeCons<'etp> = ExpressionTypeContainer<'etp>;
#[derive(Clone, Debug)]
pub enum ExpressionTypeContainer<'etp> {
    Stored(&'etp RefCell<ExprType>, Vec<&'etp RefCell<ExprType>>),
    Temp(RefCell<ExprType>)
}

impl<'etp> ExpressionTypeContainer<'etp> {
    pub fn new_temp(temp: ExprType) -> Self {
        ExprTypeCons::Temp(RefCell::new(temp))
    }

    pub fn new_stored<'tkn, 'et, 'i>(
        tokens: &'tkn [Tkn<'tkn>],
        peek: &usize,
        ident: &str, 
        variables: &'et StackFrameDictAllocator<'i, String, VariableData<'i>>, 
    ) -> Result<ExprTypeCons<'et>, ParserError<'tkn>> {
        //println!("finding type of {ident} at {}", &tokens[*peek]);
        //variables.print();
        return Ok(variables.get_in_stack(ident).ok_or(ParserError::new(
            &tokens[*peek],
            "Identifier does not exist"
        ))?.get().expr_type.clone());

        // Ok(ExprTypeCons::Stored(&variables.get(ident).ok_or(ParserError::new(
        //     &tokens[*peek],
        //     "Identifier does not exist"
        // ))?.expr_type, vec![]))
    }
    
    pub fn clone_inner(&self) -> ExprType {
        match self {
            Self::Stored(ptr, _) => ptr.borrow().clone(),
            Self::Temp(ref value) => value.borrow().clone()
        }
    }

    pub fn get(&self) -> &RefCell<ExprType> {
        match self {
            Self::Stored(ptr, _) => ptr,
            Self::Temp(ref value) => value
        }
    }

    pub fn set(&mut self, expr_type: ExprType) {
        match self {
            Self::Stored(ptr, _) => *ptr.borrow_mut() = expr_type,
            Self::Temp(_) => *self = Self::Temp(RefCell::new(expr_type))
        };
    }

    pub fn combine(self, other: ExprTypeCons<'etp>) -> ExprTypeCons<'etp> {
        use ExprTypeCons as ETC;
        match (self, other) {
            (ETC::Temp(l), ETC::Temp(_)) => {
                ETC::Temp(l)
            },
            (ETC::Stored(value, stored), ETC::Temp(_)) => {
                ETC::Stored(value, stored)
            },
            (ETC::Temp(_), ETC::Stored(value, stored)) => {
                ETC::Stored(value, stored)
            },
            (ETC::Stored(value, mut stored), ETC::Stored(other, ref mut append)) => {
                stored.push(other);
                stored.append(append);
                ETC::Stored(value, stored)
            }
        }
    }

    pub fn match_type(mut self, mut other: ExprTypeCons<'etp>) -> Option<ExprTypeCons<'etp>> {
        use ExprTypeCons as ETC;

        match (&mut self, &mut other) {
            (ETC::Temp(ref l), ETC::Temp(ref r)) => {
                l.borrow_mut().match_type(&mut r.borrow_mut());
                return Some(self);
            },
            (ETC::Stored(ref l, ref mut stored), ETC::Temp(ref r)) |
            (ETC::Temp(ref r), ETC::Stored(ref l, ref mut stored)) => {
                l.borrow_mut().match_type(&mut r.borrow_mut());
                *stored = stored.into_iter().map::<&RefCell<_>, _>(|e| {
                    *e.borrow_mut() = l.borrow().clone(); 
                    e
                }).collect();
                return Some(other);
            },
            (ETC::Stored(ref l, ref mut stored), ETC::Stored(ref r, ref mut append)) => {
                l.borrow_mut().match_type(&mut r.borrow_mut());
                stored.push(r);
                stored.append(append);
                *stored = stored.into_iter().map::<&RefCell<_>, _>(|e| {
                    *e.borrow_mut() = l.borrow().clone(); 
                    e
                }).collect();
                return Some(self);
            }
        }
    }
}

pub type ExprType = ExpressionType;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExpressionType {
    AmbiguousType,
    I8, I16, I32, I64, I128, AmbiguousNegInteger,
    U8, U16, U32, U64, U128, AmbiguousPosInteger,
    F32, F64, AmbiguousFloat,
    Char, StringLiteral, Bool,
    Ref(Box<ExprType>),
    MutRef(Box<ExprType>),
    Array {
        length: usize, 
        expr_type: Box<ExprType>
    },
    Group (Vec<ExprType>),
    Function {
        name: String,
        return_type: Box<ExprType>,
        left_args: Vec<ExprType>,
        right_args: Vec<ExprType>
    },
    FunctionPass {
        return_type: Box<ExprType>,
        left_args: Vec<ExprType>,
        right_args: Vec<ExprType>
    },
    Discard,
    DiscardRest,
    Custom {
        ident: String
    },
    Void
}

impl ExprType {
    pub fn match_type(&mut self, other: &mut ExprType) -> bool {
        use ExprType as ET;
        
        match (&self, &other) {
            (ET::AmbiguousType, new_type) => *self = (**new_type).clone(),
            (new_type, ET::AmbiguousType) => *other = (**new_type).clone(),
            (ET::AmbiguousPosInteger, new_type @ (
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
            )) => *self = (**new_type).clone(),
            (new_type @ 
                (
                    ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
                ),
                ET::AmbiguousPosInteger, 
            ) => *other = (**new_type).clone(),
            (ET::AmbiguousNegInteger, 
                new_type @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128)
            ) => *self = (**new_type).clone(),
            (new_type @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128),
                ET::AmbiguousNegInteger, 
            ) => *other = (**new_type).clone(),
            (ET::AmbiguousFloat, new_type @ (ET::F32 | ET::F64)) => *self = (**new_type).clone(),
            (new_type @ (ET::F32 | ET::F64), ET::AmbiguousFloat) => *other = (**new_type).clone(),
            (old_type, new_type) if old_type != new_type => return false,
            _ => ()
        };
        return true;
    }
}

#[derive(Debug)]
pub struct VariableData<'v> {
    _mutable: bool,
    expr_type: ExprTypeCons<'v>
}

impl<'v> VariableData<'v> {
    pub fn new(mutable: bool, expr_type: ExprTypeCons<'v>) -> Self {
        VariableData {
            _mutable: mutable,
            expr_type: expr_type
        }
    }

    pub fn get(&self) -> &RefCell<ExprType> {
        &self.expr_type.get()
    }
}

impl<'v> Display for VariableData<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Lit = Literal;

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    IntegerLiteral(i128),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),
    BooleanLiteral(bool),
}

//recursive
fn parse_expression<'pe, 'sfda, 'i>(
    expr_bump: &'pe ExprBump,
    tokens: &'pe [Tkn], 
    index: &mut usize,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pe>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>,
    min_prec: u32,

) -> Result<Expr<'pe, 'sfda>, ParserError<'pe>> {
    let mut peek = *index;
    //let very_start_expr = peek;
    let Expr {
        expr_data: mut left_expr_data, 
        expr_type: mut left_expr_type
    } = parse_atom(expr_bump, tokens, &mut peek, functions, variables)?;

    loop {
        let mut operator = &tokens[peek].token;

        if let TknType::Either(left, right) = operator {
            if OPERATOR_INFO_MAP.contains_key(left) {
                operator = left;
            } else if OPERATOR_INFO_MAP.contains_key(right) {
                operator = right;
            } else {
                break;
            }
        } else {
            if !OPERATOR_INFO_MAP.contains_key(operator) {
                break;
            }
        }

        let (prec, assoc) = OPERATOR_INFO_MAP[operator];
        let prec = prec as u32;

        if prec < min_prec {
            break;
        }

        let next_min_prec = if assoc == OpAssoc::Left {
            prec + 1
        } else {
            prec
        };

        peek += 1;

        let start_expr = peek;
        let Expr {
            expr_data: right_expr_data, 
            expr_type: right_expr_type
        } = parse_expression(
            expr_bump, 
            tokens, 
            &mut peek, 
            functions, 
            variables, 
            next_min_prec
        )?;

        let (_temp_left_expr_type, _temp_right_expr_type) = (left_expr_type.clone(), right_expr_type.clone());

        match BinOp::get_bin_op(operator).transform_type(left_expr_type, right_expr_type) {
            Some(expr_type) => {
                // eprintln!(
                //     "transformed to {:?} from applying operator {:?} to {:?} and {:?}", 
                //     expr_type.get(),
                //     BinOp::get_bin_op(operator),
                //     temp_left_expr_type,
                //     temp_right_expr_type
                // );
                left_expr_type = expr_type;
            },
            None => {
                //eprintln!("fuck fuck fuck {:?} {:?}", temp_left_expr_type, temp_right_expr_type);
                return Err(ParserError::new(
                    &tokens[start_expr],
                    "Incompatible typing"
                ));
            }
        }

        left_expr_data = expr_bump.alloc(ExprData::BinaryOp(
            BinOp::get_bin_op(operator),
            left_expr_data,
            right_expr_data,
        ));
    }

    *index = peek;
    return Ok(Expr {
        expr_data: left_expr_data,
        expr_type: left_expr_type
    });
}

pub fn parse_expression_set<'pes, 'sfda, 'i>(
    expr_bump: &'pes ExprBump,
    tokens: &'pes [Tkn],
    index: &mut usize,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pes>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>
) -> Result<Expr<'pes, 'sfda>, ParserError<'pes>> {
    let mut peek = *index;

    let mut expr_indexes = vec![];
    let mut exprs: Vec<Expr> = vec![];

    'parse_set: loop {
        let start_expr = peek;
        
        let expr;

        match parse_expression(
            expr_bump, 
            tokens, 
            &mut peek, 
            functions,
            variables, 
            0
        ) {
            Ok(value) => expr = value,
            Err(_err) => {
                // eprintln!("finished parsing set because {:?} from {} to {}", 
                //     err, &tokens[start_expr], &tokens[peek]
                // );
                break 'parse_set;
            }
        };

        if let ExprType::Function { 
            name,
            return_type, 
            left_args, 
            right_args 
        } = expr.expr_type.clone_inner() {
            if exprs.len() != left_args.len() {
                return Err(ParserError::new(
                    &tokens[start_expr],
                    "incorrect number of prefix arguments"
                ));
            }

            let mut left_exprs = vec![];
            let mut right_exprs = vec![];

            for i in 0..exprs.len() {
                exprs[i].expr_type.clone()
                    .match_type(ExprTypeCons::new_temp(left_args[i].clone()))
                    .ok_or(ParserError::new(
                        &tokens[expr_indexes[i]],
                        "could not match type to expected function paramter type"
                    ))?;

                left_exprs.push(exprs[i].expr_data);
            }

            //TODO
            //handle c function syntax vs typical group expressions
            // foo(arg1, arg2) vs. foo arg1 arg2

            'parse_right_args: for arg in right_args {
                let start_expr = peek;
                let expr = parse_expression(
                    expr_bump, 
                    tokens, 
                    &mut peek, 
                    functions, 
                    variables, 
                    0
                )?;

                expr.expr_type.match_type(ExprTypeCons::new_temp(arg))
                    .ok_or(ParserError::new(
                        &tokens[start_expr],
                        "could not match type to expected function paramter type"
                    ))?;

                right_exprs.push(expr.expr_data);
            }

            let expr = Expr {
                expr_data: expr_bump.alloc(ExprData::Function { 
                    name, 
                    left_args: left_exprs, 
                    right_args: right_exprs 
                }),
                expr_type: ExprTypeCons::new_temp(*return_type)
            };

            //eprintln!("exprs contract to {:?}", &expr);
            exprs.clear();
            exprs.push(expr);
            //eprintln!("right after {:?}", &exprs);
            continue 'parse_set;
        }

        //eprintln!("{:?} pushed", &expr);
        exprs.push(expr);
        expr_indexes.push(start_expr);

        //eprintln!("after push {:?} at {}", &exprs, &tokens[start_expr]);
    }

    //eprintln!("{:?}", &exprs);

    *index = peek;

    return match &exprs[..]  {
        [expr] => Ok(expr.clone()),
        _output @ [..] => {
            //eprintln!("{output:?} is {} long", output.len());
            Err(ParserError::new(
                &tokens[peek],
                "could not parse expression set into single expression"
            ))
        }
    };
}

fn parse_group_expression<'pge, 'sfda, 'i>(
    expr_bump: &'pge ExprBump,
    tokens: &'pge [Tkn], 
    index: &mut usize,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pge>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>
) -> Result<Expr<'pge, 'sfda>, ParserError<'pge>> {
    if is_expected_token(tokens, TknType::OpenParen, index) {
        let expr = parse_expression_set(expr_bump, tokens, index, functions, variables)?;
        expect_token(tokens, TknType::CloseParen, index).unwrap();
        return Ok(expr);
    } else if is_expected_token(tokens, TknType::Dollar, index) {
        let expr = parse_expression_set(expr_bump, tokens, index, functions, variables)?;
        return Ok(expr.map(expr_bump, |data| ExprData::AmbiguousArray(data)));
    }
    return Err(ParserError::new(
        &tokens[*index],
        "Unexpected Token.  Was expecting an '(' or a '$'",
    ));
}



fn parse_atom<'pa, 'sfda, 'i>(
    expr_bump: &'pa ExprBump,
    tokens: &'pa [Tkn], 
    index: &mut usize,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pa>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>
) -> Result<Expr<'pa, 'sfda>, ParserError<'pa>> {
    let mut peek = *index;
    let mut curr_token = tokens.get(peek).map(|e| &e.token);

    let op: Option<UnOp>;
    if let Some(TknType::Operation(Op::Plus)) = curr_token {
        op = Some(UnOp::Plus);
        peek += 1;
    } else if let Some(TknType::Operation(Op::Minus)) = curr_token {
        op = Some(UnOp::Minus);
        peek += 1;
    } else if let Some(TknType::Operation(Op::LogicNot)) = curr_token {
        op = Some(UnOp::LogicNot);
        peek += 1;
    } else if let Some(TknType::Operation(Op::BitwiseNegate)) = curr_token {
        op = Some(UnOp::BitwiseNegate);
        peek += 1;
    } else if let Some(TknType::Borrow) = curr_token {
        if let Some(TknType::Keyword(Kwrd::Mutable)) = tokens.get(peek + 1).map(|e| &e.token) {
            op = Some(UnOp::BorrowMutable);
            peek += 2;
        } else {
            op = Some(UnOp::Borrow);
            peek += 1;
        }
    } else {
        op = None;
    }

    curr_token = tokens.get(peek).map(|e| &e.token);

    let expr_data: &ExprData;
    let expr_type;
    if let Some(TknType::Identifier(ident)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Identifier(ident.clone()));
        let functions = functions.borrow();

        if functions.contains_key(ident) {
            //println!("grabbed function {ident} at {}", &tokens[peek]);
            let fun_def = unsafe { functions.get(ident).unwrap_unchecked() };
            let left_args = fun_def.left_args.iter().map(|e| e.param_type.clone()).collect();
            let right_args = fun_def.right_args.iter().map(|e| e.param_type.clone()).collect();
            expr_type = ExprTypeCons::new_temp(ExprType::Function { 
                name: ident.clone(), 
                return_type: Box::new(fun_def.return_type.clone()), 
                left_args, 
                right_args 
            }) 
        } else {
            expr_type = ExprTypeCons::new_stored(tokens, &peek, ident, variables)?;
        }
        peek += 1;
    } else if let Some(TknType::IntegerLiteral(int)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::IntegerLiteral(*int)));
        expr_type = ExprTypeCons::new_temp(if *int >= 0 {
            ExprType::AmbiguousPosInteger
        } else {
            ExprType::AmbiguousNegInteger
        });
        peek += 1;
    } else if let Some(TknType::FloatLiteral(float)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::FloatLiteral(*float)));
        expr_type = ExprTypeCons::new_temp(ExprType::AmbiguousFloat);
        peek += 1;
    } else if let Some(TknType::CharLiteral(chr)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::CharLiteral(*chr)));
        expr_type = ExprTypeCons::new_temp(ExprType::Char);
        peek += 1;
    } else if let Some(TknType::StringLiteral(string)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::StringLiteral(string.clone())));
        expr_type = ExprTypeCons::new_temp(ExprType::StringLiteral);
        peek += 1;
    } else if let Some(TknType::BooleanLiteral(b)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::BooleanLiteral(*b)));
        expr_type = ExprTypeCons::new_temp(ExprType::Bool);
        peek += 1;
    } else if let Some(TknType::OpenParen) = curr_token {
        Expr {expr_data, expr_type} = parse_group_expression(
            expr_bump, 
            tokens, 
            &mut peek, 
            functions, 
            variables
        )?;
    } else if let Some(TknType::OpenSquareBracket) | Some(TknType::Dollar) = curr_token {
        Expr {expr_data, expr_type} = parse_array(
            expr_bump, 
            tokens, 
            &mut peek, 
            functions, 
            variables
        )?;
    }  else {
        return Err(ParserError::new(
            &tokens[peek],
            "Unexpected Token.  Expected an Identifier, Literal, '(', or '$'",
        ));
    }

    *index = peek;
    match op {
        None => return Ok(Expr {expr_data, expr_type}),
        Some(op) => {
            let expr_data = expr_bump.alloc(ExprData::UnaryOp(op, expr_data));
            return Ok(Expr {expr_data, expr_type});
        },
    }
}

fn parse_array<'pa, 'sfda, 'i>(
    expr_bump: &'pa ExprBump,
    tokens: &'pa [Tkn<'pa>],
    index: &mut usize,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pa>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>
) -> Result<Expr<'pa, 'sfda>, ParserError<'pa>> {
    let mut peek = *index;
    
    // let end;
    // let (mut step, mut count) = (peek, 0);
    // loop {
    //     if step >= tokens.len() {
    //         return Err(ParserError::new(&tokens[*index], "Expected a ';' within the same scope."));
    //     } else if let Tkn {token: TknType::OpenCurlyBrace, ..} = tokens[step] {
    //         count += 1;
    //     } else if let Tkn {token: TknType::CloseCurlyBrace, ..} = tokens[step] {
    //         count -= 1;
    //     } else if let Tkn {token: TknType::Semicolon, ..} = tokens[step] && count == 0 {
    //         end = step;
    //         break;
    //     }
    //     step += 1;
    // }

    let expect_closing;
    if tokens::is_expected_token(tokens, TknType::OpenSquareBracket, &mut peek) {
        expect_closing = true;
    } else {
        tokens::expect_token(tokens, TknType::Dollar, &mut peek);
        expect_closing = false;
    }

    let mut elements = vec![];
    let mut array_type = None;

    loop {
        let start_expr = peek;
        let expr_possible = parse_expression(
            expr_bump, 
            tokens, 
            &mut peek, 
            functions,
            variables, 
            0
        );
        if let Ok(expr) = expr_possible {
            let Expr {expr_data, expr_type} = expr;

            if array_type.is_none() {
                array_type = Some(expr_type);
                elements.push(expr_data);
                continue;
            };

            array_type = array_type.map(|prev_type| 
                prev_type.match_type(expr_type).ok_or(ParserError::new(
                    &tokens[start_expr],
                    "Type of expression does not match array type"
                )
            )).transpose()?;

            // array_type = Some(prev_type.match_type(expr_type).ok_or(ParserError::new(
            //     &tokens[start_expr],
            //     "Type of expression does not match expression"
            // ))?);

            elements.push(expr_data);
            continue;
        }
        break;
    }

    let array_type = unsafe { array_type.unwrap_unchecked() };

    if expect_closing {
        tokens::expect_token(tokens, TknType::CloseSquareBracket, &mut peek).ok_or(ParserError::new(
            &tokens[*index],
            "Expected a ']'"
        ))?;
    }

    return Ok(Expr {
        expr_data: expr_bump.alloc(ExprData::Array(elements)),
        expr_type: array_type
    });
}