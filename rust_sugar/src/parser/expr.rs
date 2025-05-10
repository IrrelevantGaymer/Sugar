use std::{cell::{Ref, RefCell}, collections::HashMap, fmt::Display};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;

use crate::{lexer::token::{Kwrd, Op, Tkn, TknType}, parser::tokens};

use super::{functions::{BuiltInFunction, FullFnDef}, operators::{BinOp, OpAssoc, UnOp, OPERATOR_INFO_MAP}, stmt::StmtData, structs::{Field, Struct}, tokens::{expect_token, is_expected_token}, ExprBump, ParserError};

pub type Expr<'bumps, 'defs> = Expression<'bumps, 'defs>;
#[derive(Clone, Debug)]
pub struct Expression<'bumps, 'defs> {
    pub line: usize,
    pub expr_data: &'bumps ExprData<'bumps, 'defs>,
    pub expr_type: ExprTypeCons<'bumps>
}

pub type ExprData<'bumps, 'defs> = ExpressionData<'bumps, 'defs>;

#[derive(Clone, Debug)]
pub enum ExpressionData<'bumps, 'defs> {
    Identifier(String),
    Literal(Lit),
    Custom {
        fields: HashMap<&'defs str, &'bumps ExprData<'bumps, 'defs>>
    },
    AnonymousCustom {
        fields: Box<[(String, &'bumps ExprData<'bumps, 'defs>)]>
    },
    CustomField {
        data: &'bumps Expr<'bumps, 'defs>,
        field: &'defs Field
    },
    AnonymousCustomField {
        data: &'bumps Expr<'bumps, 'defs>,
        field_name: String
    },
    Conditional{ 
        conds: Vec<Expr<'bumps, 'defs>>, 
        bodies: Vec<&'bumps StmtData<'bumps, 'defs>> 
    },
    Function {
        name: String,
        left_args: Vec<Expr<'bumps, 'defs>>,
        right_args: Vec<Expr<'bumps, 'defs>>
    },
    BinaryOp(BinOp, Expr<'bumps, 'defs>, Expr<'bumps, 'defs>),
    UnaryOp(UnOp, Expr<'bumps, 'defs>),
    Array(Vec<Expr<'bumps, 'defs>>),
    Index { 
        expr: Expr<'bumps, 'defs>,
        index: Expr<'bumps, 'defs>
    },
    Tuple(Vec<Expr<'bumps, 'defs>>),
    AmbiguousGroup(Expr<'bumps, 'defs>)
}

pub type ExprTypeCons<'bumps> = ExpressionTypeContainer<'bumps>;
#[derive(Clone, Debug)]
pub struct ExpressionTypeContainer<'bumps> {
    stored: Vec<&'bumps RefCell<ExprType>>
}

impl<'bumps> ExprTypeCons<'bumps> {
    pub fn new(expr_bump: &ExprBump, expr_type: ExprType) -> ExprTypeCons {
        ExpressionTypeContainer { 
            stored: vec![
                expr_bump.alloc(RefCell::new(expr_type))
            ] 
        }
    }

    pub fn new_stored(expr_type: &RefCell<ExprType>) -> ExprTypeCons {
        ExpressionTypeContainer { 
            stored: vec![expr_type] 
        }
    }
    
    pub fn grab_variable_type<'tkns, 'defs>(
        tokens: &'tkns [Tkn],
        peek: usize,
        ident: &str, 
        variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>, 
    ) -> Result<ExprTypeCons<'bumps>, ParserError<'tkns, 'bumps, 'defs>> {
        return Ok(variables.get_in_stack(ident).ok_or_else(|| ParserError::VariableDoesNotExist { 
            tkn: &tokens[peek] 
        })?.get().expr_type.clone());
    }
    
    pub fn clone_inner(&self) -> ExprType {
        unsafe {
            self.stored.first().unwrap_unchecked().borrow().clone()
        }
    }

    pub fn get(&self) -> Ref<'_, ExprType> {
        unsafe {
            self.stored.first().unwrap_unchecked().borrow()
        }
    }

    fn combine(self, other: ExprTypeCons<'bumps>) -> ExprTypeCons<'bumps> {
        let mut stored = vec![];
        stored.extend(self.stored.into_iter());
        stored.extend(other.stored.into_iter());

        return ExprTypeCons {
            stored
        }
    }

    unsafe fn update(&self, expr_type: ExprType) {
        for stored in &self.stored {
            *stored.borrow_mut() = expr_type.clone()
        }
    }

    pub fn match_type(&mut self, other: &mut ExprTypeCons<'bumps>) -> Option<ExprTypeCons<'bumps>> {
        let mut left = self.clone_inner();
        let mut right = other.clone_inner();

        if !left.match_type(&mut right) {
            return None;
        }

        unsafe {
            self.update(left);
            other.update(right);
        }

        return Some(self.clone().combine(other.clone()));
    }
}

impl<'etp> PartialEq for ExprTypeCons<'etp> {
    fn eq(&self, other: &Self) -> bool {
        return *self.get() == *other.get();
    }
}

pub type ExprType = ExpressionType;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExpressionType {
    AmbiguousType,
    I8, I16, I32, I64, I128, ISize, AmbiguousNegInteger,
    U8, U16, U32, U64, U128, USize, AmbiguousPosInteger,
    F32, F64, AmbiguousFloat,
    Char, StringLiteral, Bool,
    Ref(Box<ExprType>),
    MutRef(Box<ExprType>),
    Array {
        length: Option<usize>, 
        expr_type: Box<ExprType>
    },
    //TODO separate into PatternGroup and PatternAmbiguousGroup, since actual Tuple types cannot use the DiscardMany type
    Tuple { start: Vec<ExprType>, end: Vec<ExprType> },
    AmbiguousGroup { start: Vec<ExprType>, end: Vec<ExprType> },
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
    DiscardSingle,
    DiscardMany,
    Custom {
        ident: String
    },
    AnonymousCustom {
        fields: Box<[(String, ExprType)]>
    },
    Void,
    Never
}

impl ExprType {
    pub fn match_type(&mut self, other: &mut ExprType) -> bool {
        use ExprType as ET;
        
        match (self, other) {
            (_, ET::Never) => (),
            (ET::Never, _) => (),
            (l @ ET::AmbiguousType, new_type) => *l = (*new_type).clone(),
            (new_type, r @ ET::AmbiguousType) => *r = (*new_type).clone(),
            (ET::DiscardSingle, _) => return true,
            (_, ET::DiscardSingle) => return true,
            (l @ ET::AmbiguousPosInteger, new_type @ (
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | ET::ISize |
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128 | ET::USize
            )) => *l = (*new_type).clone(),
            (
                new_type @ (
                    ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | ET::ISize |
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128 | ET::USize
                ),
                r @ ET::AmbiguousPosInteger, 
            ) => *r = (*new_type).clone(),
            (
                l @ ET::AmbiguousNegInteger, 
                new_type @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | ET::ISize)
            ) => *l = (*new_type).clone(),
            (
                new_type @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | ET::ISize),
                r @ ET::AmbiguousNegInteger, 
            ) => *r = (*new_type).clone(),
            (l @ ET::AmbiguousFloat, new_type @ (ET::F32 | ET::F64)) => *l = (*new_type).clone(),
            (new_type @ (ET::F32 | ET::F64), r @ ET::AmbiguousFloat) => *r = (*new_type).clone(),
            (
                ET::AnonymousCustom { fields: l_fields },
                ET::AnonymousCustom { fields: r_fields }
            ) => {
                if l_fields.len() != r_fields.len() {
                    return false;
                }

                for (
                    (l_field_name, l_field_type), 
                    (r_field_name, r_field_type)
                ) in l_fields.iter_mut().zip(r_fields) {
                    if l_field_name != r_field_name {
                        return false;
                    }
                    if !l_field_type.match_type(r_field_type) {
                        return false;
                    }
                }
            },
            (
                ET::Array {
                    length: ref mut l_length, 
                    expr_type: ref mut l_type
                }, 
                ET::Array {
                    length: ref mut r_length, 
                    expr_type: ref mut r_type
                }
            ) => {
                if l_length.is_some() && r_length.is_some() {
                    return unsafe { l_length.zip(*r_length).map(|(l, r)| l == r).unwrap_unchecked() };
                } else if l_length.is_some() {
                    *r_length = l_length.clone();
                } else if r_length.is_some() {
                    *l_length = r_length.clone();
                }

                return l_type.match_type(r_type.as_mut());
            },
            (
                ET::Tuple { 
                    start: ref mut l_start_types, 
                    end: ref mut l_end_types 
                }, 
                ET::Tuple {
                    start: ref mut r_types,
                    end: r_end
                }
            ) if r_end.is_empty() && l_start_types.len() + l_end_types.len() <= r_types.len() => {
                let (r_start_types, r_end_types) = r_types.split_at_mut(l_start_types.len());

                return l_start_types.iter_mut().zip(r_start_types.iter_mut()).map(|(l, r)| l.match_type(r))
                    .chain(l_end_types.iter_mut().zip(r_end_types.iter_mut().rev()).map(|(l, r)| l.match_type(r)))
                    .all(|e| e);
            },
            (
                ET::Tuple { 
                    start: ref mut l_types, 
                    end: l_end
                }, 
                ET::Tuple {
                    start: ref mut r_start_types,
                    end: ref mut r_end_types 
                }
            ) if l_end.is_empty() && r_start_types.len() + r_end_types.len() <= l_types.len() => {
                let (l_start_types, l_end_types) = l_types.split_at_mut(r_start_types.len());
                
                return r_start_types.iter_mut().zip(l_start_types.iter_mut()).map(|(r, l)| r.match_type(l))
                    .chain(r_end_types.iter_mut().zip(l_end_types.iter_mut().rev()).map(|(r, l)| r.match_type(l)))
                    .all(|e| e);
            },
            ( ET::Tuple {..}, ET::Tuple {..}) => unreachable!("Both Group Types include a Discard Many Type"),
            (
                l @ ET::AmbiguousGroup { .. }, 
                ET::Array { length: r_length, expr_type: r_type}
            ) => {
                {
                    let ET::AmbiguousGroup { start: l_start_types, end: l_end_types } = l else {
                        unreachable!();
                    };

                    if let Some(r_len) = r_length && l_start_types.len() + l_end_types.len() > *r_len {
                        return false;
                    }

                    if !l_start_types.iter_mut().chain(l_end_types.iter_mut())
                        .map(|e| e.match_type(r_type)).all(|e| e) 
                    {
                        return false;
                    }
                }

                *l = ET::Array { length: *r_length, expr_type: r_type.clone() };
                return true;
            },
            (old_type, new_type) if old_type != new_type => return false,
            _ => ()
        };
        return true;
    }


    pub fn size_of(&self, structs: &[Struct]) -> usize {
        const ARCHITECTURE_SIZE: usize = std::mem::size_of::<usize>();

        return match self {
            ExprType::AmbiguousType => panic!("amibiguous type is not real type"),
            ExprType::AmbiguousNegInteger | 
            ExprType::AmbiguousPosInteger => ARCHITECTURE_SIZE,
            ExprType::AmbiguousFloat => 4,
            ExprType::I8 | ExprType::U8 | ExprType::Char | ExprType::Bool => 1,
            ExprType::I16 | ExprType::U16 => 2,
            ExprType::I32 | ExprType::U32 | ExprType::F32 => 4,
            ExprType::I64 | ExprType::U64 | ExprType::F64 => 8,
            ExprType::I128 | ExprType::U128 => 16,
            ExprType::ISize | ExprType::USize => ARCHITECTURE_SIZE,
            ExprType::Ref(_) => ARCHITECTURE_SIZE,
            ExprType::MutRef(_) => ARCHITECTURE_SIZE,
            ExprType::StringLiteral => ARCHITECTURE_SIZE * 2,
            ExprType::Array { length: Some(length), expr_type } => expr_type.size_of(structs) * length,
            ExprType::Array { length: None, .. } => panic!("type does not have constant size"),
            ExprType::Tuple {..} => todo!("not implemented yet; requires padding"),
            ExprType::AmbiguousGroup { .. } => todo!("not implemented yet; requires padding"),
            ExprType::Function { .. } => todo!("not implemented yet"),
            ExprType::FunctionPass { .. } => todo!("not implemented yet"),
            ExprType::DiscardSingle => 0,
            ExprType::DiscardMany => 0,
            ExprType::Custom { ident } => {
                let custom_struct = structs.iter()
                    .find(|custom_struct| custom_struct.name == *ident)
                    .expect(format!("struct {ident} does not exist").as_str());
                let mut size = 0;
                for field in &custom_struct.fields {
                    size += field.field_type.size_of(structs);
                }
                size
            },
            ExprType::AnonymousCustom { fields } => {
                let mut size = 0;
                for (_, field_type) in fields.iter() {
                    size += field_type.size_of(structs);
                }
                size
            },
            ExprType::Void => 0,
            ExprType::Never => 0
        };
    }

    pub fn is_real_type(&self) -> bool {
        return !matches!(self, 
            Self::AmbiguousType | 
            Self::AmbiguousNegInteger | 
            Self::AmbiguousPosInteger | 
            Self::AmbiguousFloat | 
            Self::AmbiguousGroup { .. } |
            Self::DiscardSingle |
            Self::DiscardMany |
            Self::Void
        )
    }
}

impl std::fmt::Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let simple_output = match self {
            ExpressionType::AmbiguousType       => Some("`Ambiguous Type`"),
            ExpressionType::I8                  => Some("i8"),
            ExpressionType::I16                 => Some("i16"),
            ExpressionType::I32                 => Some("i32"),
            ExpressionType::I64                 => Some("i64"),
            ExpressionType::I128                => Some("i128"),
            ExpressionType::ISize               => Some("isize"),
            ExpressionType::AmbiguousNegInteger => Some("`Ambiguous Negative Integer`"),
            ExpressionType::U8                  => Some("u8"),
            ExpressionType::U16                 => Some("u16"),
            ExpressionType::U32                 => Some("u32"),
            ExpressionType::U64                 => Some("u64"),
            ExpressionType::U128                => Some("u128"),
            ExpressionType::USize               => Some("usize"),
            ExpressionType::AmbiguousPosInteger => Some("`Ambiguous Positive Integer`"),
            ExpressionType::F32                 => Some("f32"),
            ExpressionType::F64                 => Some("f64"),
            ExpressionType::AmbiguousFloat      => Some("`Ambiguous Float`"),
            ExpressionType::Char                => Some("char"),
            ExpressionType::StringLiteral       => Some("str"),
            ExpressionType::Bool                => Some("bool"),
            ExpressionType::DiscardSingle       => Some("_"),
            ExpressionType::DiscardMany         => Some(".."),
            ExpressionType::Void                => Some("void"),
            ExpressionType::Never               => Some("!"),
            _                                   => None
        };

        if let Some(out) = simple_output {
            return write!(f, "{out}");
        }
        
        write!(f, "{}", match self {
            ExpressionType::Ref(expression_type) => format!("&{expression_type}"),
            ExpressionType::MutRef(expression_type) => format!("&mut {expression_type}"),
            ExpressionType::Array { length: Some(length), expr_type } => format!("[{expr_type}; {length}]"),
            ExpressionType::Array { length: None, expr_type } => format!("[{expr_type}; ?]"),
            ExpressionType::Tuple { .. } => todo!(),
            ExpressionType::AmbiguousGroup { .. } => todo!(),
            ExpressionType::Function { .. } => todo!(),
            ExpressionType::FunctionPass { .. } => todo!(),
            ExpressionType::Custom { ident } => ident.clone(),
            ExpressionType::AnonymousCustom { fields } => 'str: {
                let mut output = String::new();

                if fields.is_empty() {
                    output += "{}";
                    break 'str output;
                }

                output += "{ ";
                let ((first_field_name, first_field_type), fields) = fields.split_first().unwrap();

                output += first_field_name.as_str();
                output += ": ";
                output += first_field_type.to_string().as_str();

                for (field_name, field_type) in fields {
                    output += ", ";
                    output += field_name.as_str();
                    output += ": ";
                    output += field_type.to_string().as_str();
                }

                output += " }";
                break 'str output;
            },
            ExpressionType::AmbiguousType |
            ExpressionType::I8 |
            ExpressionType::I16 |
            ExpressionType::I32 |
            ExpressionType::I64 |
            ExpressionType::I128 |
            ExpressionType::ISize |
            ExpressionType::AmbiguousNegInteger |
            ExpressionType::U8 |
            ExpressionType::U16 |
            ExpressionType::U32 |
            ExpressionType::U64 |
            ExpressionType::U128 |
            ExpressionType::USize |
            ExpressionType::AmbiguousPosInteger |
            ExpressionType::F32 |
            ExpressionType::F64 |
            ExpressionType::AmbiguousFloat |
            ExpressionType::Char |
            ExpressionType::StringLiteral |
            ExpressionType::Bool |
            ExpressionType::DiscardSingle |
            ExpressionType::DiscardMany | 
            ExpressionType::Void | 
            ExpressionType::Never => unreachable!()
        })
    }
}

#[derive(Debug)]
pub struct VariableData<'tkns, 'bumps> {
    pub tkn: &'tkns Tkn,
    pub mutable: bool,
    pub expr_type: ExprTypeCons<'bumps>
}

impl<'tkns, 'bumps> VariableData<'tkns, 'bumps> {
    pub fn new(
        tkn: &'tkns Tkn, 
        mutable: bool, 
        expr_type: ExprTypeCons<'bumps>
    ) -> Self {
        VariableData {
            tkn,
            mutable,
            expr_type: expr_type
        }
    }

    pub fn get_type(&self) -> Ref<'_, ExprType> {
        self.expr_type.get()
    }
}

impl<'tkns, 'bumps> Display for VariableData<'tkns, 'bumps> {
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

impl Literal {
    pub fn get_type(&self) -> ExprType {
        match self {
            Literal::IntegerLiteral(_) => ExprType::AmbiguousNegInteger,
            Literal::FloatLiteral(_) => ExprType::AmbiguousFloat,
            Literal::CharLiteral(_) => ExprType::Char,
            Literal::StringLiteral(_) => ExprType::StringLiteral,
            Literal::BooleanLiteral(_) => ExprType::Bool,
        }
    }
}

//recursive
fn parse_expression<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    structs: &'defs [Struct],
    tokens: &'tkns [Tkn], 
    index: &mut usize,
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    min_prec: u32,
) -> Result<Expr<'bumps, 'defs>, ParserError<'tkns, 'bumps, 'defs>> {
    let mut peek = *index;
    //let very_start_expr = peek;
    let Expr {
        expr_data: mut left_expr_data, 
        expr_type: mut left_expr_type,
        ..
    } = parse_atom(expr_bump, structs, tokens, &mut peek, line, functions, variables)?;

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
            expr_type: mut right_expr_type,
            ..
        } = parse_expression(
            expr_bump, 
            structs,
            tokens, 
            &mut peek, 
            line,
            functions, 
            variables, 
            next_min_prec
        )?;

        //let (_temp_left_expr_type, _temp_right_expr_type) = (left_expr_type.clone(), right_expr_type.clone());

        match BinOp::get_bin_op(operator).transform_type(
            expr_bump, 
            &mut left_expr_type, 
            &mut right_expr_type
        ) {
            Some(expr_type) => {
                left_expr_type = expr_type;
            },
            None => {
                return Err(ParserError::CouldNotMatchType { 
                    tkns: &tokens[start_expr..peek], 
                    calculated_type: right_expr_type.clone_inner(), 
                    expected_type: left_expr_type.clone_inner() 
                });
            }
        }

        left_expr_data = expr_bump.alloc(ExprData::BinaryOp(
            BinOp::get_bin_op(operator),
            Expr {
                line,
                expr_data: left_expr_data,
                expr_type: left_expr_type.clone(),
            },
            Expr {
                line,
                expr_data: right_expr_data,
                expr_type: right_expr_type
            }
        ));
    }

    *index = peek;
    return Ok(Expr {
        line,
        expr_data: left_expr_data,
        expr_type: left_expr_type
    });
}

pub fn parse_expression_set<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    structs: &'defs [Struct],
    tokens: &'tkns [Tkn],
    index: &mut usize,
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>
) -> Result<Expr<'bumps, 'defs>, ParserError<'tkns, 'bumps, 'defs>> {
    let mut peek = *index;
    let beginning_index = peek;

    let mut expr_start_indices = vec![];
    let mut expr_end_indices = vec![];
    let mut exprs: Vec<Expr> = vec![];

    let last_expr_err;

    'parse_set: loop {
        let start_expr = peek;
        
        let expr;

        match parse_expression(
            expr_bump, 
            structs,
            tokens, 
            &mut peek, 
            line,
            functions,
            variables, 
            0
        ) {
            Ok(value) => expr = value,
            Err(err) => {
                last_expr_err = err;
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
                return Err(ParserError::IncorrectNumberPrefixArguments { 
                    tkn: &tokens[peek], 
                    args: exprs.into_iter().map(|e| e.expr_type.clone_inner()).collect::<Vec<_>>().into_boxed_slice(), 
                    expected_args: left_args.into_boxed_slice(), 
                    function: functions.borrow().get(name.as_str()).unwrap().clone()
                });
            }

            let mut left_exprs = vec![];
            let mut right_exprs = vec![];

            for (i, mut expr) in exprs.drain(..).enumerate() {
                expr.expr_type = expr.expr_type
                    .match_type(&mut ExprTypeCons::new(expr_bump, left_args[i].clone()))
                    .ok_or_else(|| ParserError::CouldNotMatchType {
                        tkns: &tokens[expr_start_indices[i]..expr_end_indices[i]],
                        calculated_type: expr.expr_type.clone_inner(),
                        expected_type: left_args[i].clone()
                    })?;

                left_exprs.push(Expr {
                    line,
                    expr_data: expr.expr_data,
                    expr_type: expr.expr_type
                });
            }

            //TODO
            //handle c function syntax vs typical group expressions
            // foo(arg1, arg2) vs. foo arg1 arg2

            'parse_right_args: for arg in right_args {
                let start_expr = peek;
                let mut expr = parse_expression(
                    expr_bump, 
                    structs,
                    tokens, 
                    &mut peek, 
                    line,
                    functions, 
                    variables, 
                    0
                )?;

                expr.expr_type = expr.expr_type.match_type(&mut ExprTypeCons::new(expr_bump, arg.clone()))
                    .ok_or_else(|| ParserError::CouldNotMatchType { 
                        tkns: &tokens[start_expr..peek], 
                        calculated_type: expr.expr_type.clone_inner(), 
                        expected_type: arg
                    })?;

                right_exprs.push(Expr {
                    line,
                    expr_data: expr.expr_data,
                    expr_type: expr.expr_type
                });
            }

            let expr = Expr {
                line,
                expr_data: expr_bump.alloc(ExprData::Function { 
                    name, 
                    left_args: left_exprs, 
                    right_args: right_exprs 
                }),
                expr_type: ExprTypeCons::new(expr_bump, *return_type)
            };

            exprs.push(expr);
            continue 'parse_set;
        }

        exprs.push(expr);
        expr_start_indices.push(start_expr);
        expr_end_indices.push(peek);
    }

    *index = peek;

    return match &exprs[..]  {
        [expr] => Ok(expr.clone()),
        [] => {
            Err(last_expr_err)
        }
        [..] => {
            Err(ParserError::MultipleExpressions { 
                tkn: &tokens[beginning_index], 
                expr: exprs.into_boxed_slice()
            })
        }
    };
}

fn parse_group_expression<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    structs: &'defs [Struct],
    tokens: &'tkns [Tkn], 
    index: &mut usize,
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>
) -> Result<Expr<'bumps, 'defs>, ParserError<'tkns, 'bumps, 'defs>> {
    if is_expected_token(tokens, TknType::OpenParen, index) {
        let expr = parse_expression_set(expr_bump, structs, tokens, index, line, functions, variables)?;
        expect_token(tokens, TknType::CloseParen, index).unwrap();
        return Ok(expr);
    } else if is_expected_token(tokens, TknType::Dollar, index) {
        let expr = parse_expression_set(expr_bump, structs, tokens, index, line, functions, variables)?;
        //TODO return ambiguous group instead
        return Ok(expr);
    }
    return Err(ParserError::ExpectedTokens { 
        tkn: &tokens[*index], 
        received: tokens[*index..].iter().map(|tkn| &tkn.token),
        expected: &[TknType::OpenParen, TknType::Dollar] 
    });
}



fn parse_atom<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    structs: &'defs [Struct],
    tokens: &'tkns [Tkn], 
    index: &mut usize,
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>
) -> Result<Expr<'bumps, 'defs>, ParserError<'tkns, 'bumps, 'defs>> {
    let mut peek = *index;
    let mut curr_token = tokens.get(peek).map(|e| &e.token);

    let op: Option<UnOp>;
    if let Some(TknType::Operation(Op::PlusFloat)) = curr_token {
        op = Some(UnOp::PlusFloat);
        peek += 1;
    } else if let Some(TknType::Operation(Op::Plus)) = curr_token {
        op = Some(UnOp::Plus);
        peek += 1;
    } else if let Some(TknType::Operation(Op::MinusFloat)) = curr_token {
        op = Some(UnOp::MinusFloat);
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
        } else if let Some(TknType::Keyword(Kwrd::InteriorMutable)) = tokens.get(peek + 1).map(|e| &e.token) {
            op = Some(UnOp::BorrowInteriorMutable);
            peek += 2;
        } else {
            op = Some(UnOp::Borrow);
            peek += 1;
        }
    } else {
        op = None;
    }

    curr_token = tokens.get(peek).map(|e| &e.token);

    let mut expr_data: &ExprData;
    let mut expr_type;
    if let Some(TknType::Identifier(ident)) = curr_token {
        if let Some(fun_def) = functions.borrow().get(ident) {
            expr_data = expr_bump.alloc(ExprData::Identifier(ident.clone()));
            let left_args = fun_def.left_args.iter().map(|e| e.param_type.clone()).collect();
            let right_args = fun_def.right_args.iter().map(|e| e.param_type.clone()).collect();
            expr_type = ExprTypeCons::new(expr_bump, ExprType::Function { 
                name: ident.clone(), 
                return_type: Box::new(fun_def.return_type.clone()), 
                left_args, 
                right_args 
            });
            peek += 1;
        } else if let Some(built_in_fn) = BuiltInFunction::from_name(ident) {
            expr_data = expr_bump.alloc(ExprData::Identifier(ident.clone()));
            expr_type = ExprTypeCons::new(expr_bump, BuiltInFunction::get_type(&built_in_fn));
            peek += 1;
        } else if let Some(custom_struct) = structs.iter()
            .find(|custom_struct| custom_struct.name == *ident) 
        {
            peek += 1;

            tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)
                .ok_or_else(|| ParserError::ExpectedToken { 
                    tkn: &tokens[peek], 
                    expected: TknType::OpenCurlyBrace 
                })?;

            let mut fields: HashMap<&str, &ExprData> = HashMap::new();
            let mut field_indices: HashMap<&str, &Tkn> = HashMap::new();

            let mut needed_comma = false;
            loop {
                let Some(TknType::Identifier(field_name)) = tokens::get_token(tokens, peek).map(|e| &e.token) else {
                    break;
                };
                let field_index = peek;

                if needed_comma {
                    return Err(ParserError::ExpectedToken{ tkn: &tokens[peek], expected: TknType::Comma });
                }

                //TODO add accessor checks

                let field = custom_struct.fields.iter().find(|field| 
                    field.field_name == *field_name
                ).ok_or_else(|| ParserError::FieldDoesNotExist { 
                    tkn: &tokens[peek], 
                    custom_struct
                })?;

                peek += 1;

                if fields.contains_key(field.field_name.as_str()) {
                    return Err(ParserError::AlreadyDefinedField { 
                        tkn: &tokens[peek], 
                        defined_field: field_indices.get(field.field_name.as_str()).unwrap() 
                    });
                }

                if tokens::expect_token(tokens, TknType::Colon, &mut peek).is_some() {
                    let start_expr = peek;
                    let mut expr = parse_expression_set(
                        expr_bump, 
                        structs, 
                        tokens, 
                        &mut peek, 
                        line,
                        functions, 
                        variables
                    )?;
                    
                    expr.expr_type.match_type(
                        &mut ExprTypeCons::new(expr_bump, field.field_type.clone())
                    ).ok_or(ParserError::CouldNotMatchType { 
                        tkns: &tokens[start_expr..peek], 
                        calculated_type: expr.expr_type.clone_inner(), 
                        expected_type: field.field_type.clone()
                    })?;

                    fields.insert(
                        &field.field_name, 
                        expr_bump.alloc(expr.expr_data.clone())
                    );
                    field_indices.insert(
                        &field.field_name,
                        &tokens[field_index]
                    );
                } else {
                    let ident_type = &variables.get_in_stack(&field.field_name)
                        .ok_or_else(|| ParserError::FieldExpressionNotDefined { 
                            tkn: &tokens[peek], 
                            field 
                        })?
                        .get().expr_type;

                    ident_type.clone().match_type(
                        &mut ExprTypeCons::new(expr_bump, field.field_type.clone())
                    ).ok_or_else(|| ParserError::CouldNotMatchType { 
                        tkns: core::slice::from_ref(&tokens[peek]), 
                        calculated_type: ident_type.clone_inner(), 
                        expected_type: field.field_type.clone()
                    })?;

                    fields.insert(
                        &field.field_name, 
                        expr_bump.alloc(
                            ExprData::Identifier(field.field_name.clone())
                        )
                    );
                    field_indices.insert(
                        &field.field_name,
                        &tokens[field_index]
                    );
                }

                needed_comma = tokens::expect_token(tokens, TknType::Comma, &mut peek).is_none();
            }

            //TODO check if all fields are initialized

            tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek)
                .ok_or_else(|| ParserError::ExpectedToken { 
                    tkn: &tokens[peek], 
                    expected: TknType::CloseCurlyBrace 
                })?;

            expr_data = expr_bump.alloc(ExprData::Custom { fields });
            expr_type = ExprTypeCons::new(expr_bump, ExprType::Custom { 
                ident: custom_struct.name.clone() 
            });
        } else {
            expr_data = expr_bump.alloc(ExprData::Identifier(ident.clone()));
            expr_type = ExprTypeCons::grab_variable_type(tokens, peek, ident, variables)?;
            peek += 1;
        }
    } else if let Some(TknType::IntegerLiteral { int, .. }) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::IntegerLiteral(*int)));
        expr_type = ExprTypeCons::new(expr_bump, if *int >= 0 {
            ExprType::AmbiguousPosInteger
        } else {
            ExprType::AmbiguousNegInteger
        });
        peek += 1;
    } else if let Some(TknType::FloatLiteral { float, .. }) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::FloatLiteral(*float)));
        expr_type = ExprTypeCons::new(expr_bump, ExprType::AmbiguousFloat);
        peek += 1;
    } else if let Some(TknType::CharLiteral(chr)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::CharLiteral(*chr)));
        expr_type = ExprTypeCons::new(expr_bump, ExprType::Char);
        peek += 1;
    } else if let Some(TknType::StringLiteral(string)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::StringLiteral(string.clone())));
        expr_type = ExprTypeCons::new(expr_bump, ExprType::StringLiteral);
        peek += 1;
    } else if let Some(TknType::BooleanLiteral(b)) = curr_token {
        expr_data = expr_bump.alloc(ExprData::Literal(Lit::BooleanLiteral(*b)));
        expr_type = ExprTypeCons::new(expr_bump, ExprType::Bool);
        peek += 1;
    } else if let Some(TknType::OpenParen) = curr_token {
        Expr {expr_data, expr_type, ..} = parse_group_expression(
            expr_bump, 
            structs,
            tokens, 
            &mut peek, 
            line,
            functions, 
            variables
        )?;
    } else if let Some(TknType::OpenSquareBracket) = curr_token {
        Expr {expr_data, expr_type, ..} = parse_array(
            expr_bump, 
            structs,
            tokens, 
            &mut peek, 
            line,
            functions, 
            variables
        )?;
    } else if let Some(TknType::Dollar) = curr_token {
        if let Ok(expr) = parse_array(
            expr_bump, 
            structs,
            tokens, 
            &mut peek, 
            line,
            functions, 
            variables
        ) {
            expr_data = expr.expr_data;
            expr_type = expr.expr_type;
        } else if let Ok(expr) = parse_group_expression(
            expr_bump, 
            structs, 
            tokens, 
            &mut peek, 
            line,
            functions, 
            variables
        ) {
            expr_data = expr.expr_data;
            expr_type = expr.expr_type;
        } else {
            return Err(ParserError::InvalidDollarExpression { 
                tkn: &tokens[peek] 
            });
        }
    } else if let Some(TknType::OpenCurlyBrace) = curr_token {
        peek += 1;
        let mut field_datas = vec![];
        let mut field_types = vec![];
        let mut need_comma = false;
        loop {
            if tokens::is_expected_token(tokens, TknType::CloseCurlyBrace, &mut peek) {
                break;
            }

            if need_comma {
                return Err(ParserError::ExpectedTokens { 
                    tkn: &tokens[peek], 
                    received: tokens[*index..].iter().map(|tkn| &tkn.token),
                    expected: &[TknType::Comma, TknType::CloseCurlyBrace] 
                });
            }

            let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) else {
                return Err(ParserError::ExpectedIdentifier { tkn: &tokens[peek] });
            };
            peek += 1;

            tokens::expect_token(tokens, TknType::Colon, &mut peek)
                .ok_or_else(|| ParserError::ExpectedToken { 
                    tkn: &tokens[peek], 
                    expected: TknType::Colon 
                })?;

            let Expr {
                expr_data,
                expr_type,
                ..
            } = parse_expression_set(
                expr_bump, 
                structs, 
                tokens, 
                &mut peek, 
                line, 
                functions, 
                variables
            )?;

            field_datas.push((ident.clone(), expr_data));
            field_types.push((ident.clone(), expr_type.clone_inner()));

            if !tokens::is_expected_token(tokens, TknType::Comma, &mut peek) {
                need_comma = true;
            }
        }
        expr_data = expr_bump.alloc(ExprData::AnonymousCustom { 
            fields: field_datas.into_boxed_slice() 
        });
        expr_type = ExprTypeCons::new(expr_bump, ExprType::AnonymousCustom { 
            fields: field_types.into_boxed_slice() 
        })
    } else {
        return Err(ParserError::InvalidExpressionAtom { tkn: &tokens[peek] });
    }

    loop {
        if tokens::is_expected_token(tokens, TknType::Dot, &mut peek) {
            if let ExprType::Custom { ident } = expr_type.clone_inner() && 
                let Some(TknType::Identifier(field_name)) = tokens::get_token(tokens, peek).map(|e| &e.token) && 
                let Some(custom_struct) = structs.iter().find(|custom_struct| custom_struct.name == *ident) &&
                let Some(field) = custom_struct.fields.iter().find(|field| field.field_name == *field_name)
            {
                expr_data = expr_bump.alloc(ExprData::CustomField { 
                    data: expr_bump.alloc(Expr {line, expr_data, expr_type}), 
                    field
                });
                expr_type = ExprTypeCons::new(expr_bump, field.field_type.clone());

                peek += 1;
                continue;
            } else if let ExprType::AnonymousCustom { fields } = expr_type.clone_inner() &&
                let Some(TknType::Identifier(field_name)) = tokens::get_token(tokens, peek).map(|e| &e.token) &&
                let Some((field_name, field_type)) = fields.iter().find(|(anonymous_field_name, _)| 
                    anonymous_field_name == field_name
                )
            {
                expr_data = expr_bump.alloc(ExprData::AnonymousCustomField { 
                    data: expr_bump.alloc(Expr {line, expr_data, expr_type}), 
                    field_name: field_name.clone()
                });
                expr_type = ExprTypeCons::new(expr_bump, field_type.clone());

                peek += 1;
                continue;
            } else {
                //TODO detect methods
                return Err(ParserError::InvalidDotExpression { 
                    tkn: &tokens[peek - 1], 
                    expr_type: expr_type.clone_inner() 
                });
            }
        }
        break;
    }

    *index = peek;
    match op {
        None => return Ok(Expr {line, expr_data, expr_type}),
        Some(op) => {
            let expr_data = expr_bump.alloc(ExprData::UnaryOp(op, Expr {
                line,
                expr_data,
                expr_type: expr_type.clone()
            }));
            return Ok(Expr {line, expr_data, expr_type});
        },
    }
}

fn parse_array<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    structs: &'defs [Struct],
    tokens: &'tkns [Tkn],
    index: &mut usize,
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>
) -> Result<Expr<'bumps, 'defs>, ParserError<'tkns, 'bumps, 'defs>> {
    let mut peek = *index;
    
    let expect_closing;
    if tokens::is_expected_token(tokens, TknType::OpenSquareBracket, &mut peek) {
        expect_closing = true;
    } else {
        tokens::expect_token(tokens, TknType::Dollar, &mut peek).ok_or_else(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::Dollar 
        })?;
        expect_closing = false;
    }

    let mut elements = vec![];
    let mut array_type = None;

    loop {
        let start_expr = peek;
        let expr_possible = parse_expression(
            expr_bump, 
            structs,
            tokens, 
            &mut peek, 
            line,
            functions, 
            variables, 
            0
        );
        if let Ok(expr) = expr_possible {
            let Expr {expr_data, mut expr_type, ..} = expr;

            if array_type.is_none() {
                array_type = Some(expr_type);
                elements.push(expr_data);
                continue;
            };

            array_type = array_type.map(|mut prev_type| 
                prev_type.match_type(&mut expr_type).ok_or_else(|| ParserError::CouldNotMatchType { 
                    tkns: &tokens[start_expr..peek], 
                    calculated_type: expr_type.clone_inner(), 
                    expected_type: prev_type.clone_inner() 
                })
            ).transpose()?;

            // array_type = Some(prev_type.match_type(expr_type).ok_or_else(|| ParserError::new(
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
        tokens::expect_token(tokens, TknType::CloseSquareBracket, &mut peek)
            .ok_or_else(|| ParserError::ExpectedToken { 
                tkn: &tokens[peek], 
                expected: TknType::CloseAngularBracket 
            })?;
    }

    let length = elements.len();

    return Ok(Expr {
        line,
        expr_data: expr_bump.alloc(ExprData::Array(elements.iter().map(|data| Expr {
            line,
            expr_data: data,
            expr_type: array_type.clone()
        }).collect())),
        expr_type: ExprTypeCons::new(expr_bump, ExprType::Array { 
            length: Some(length), 
            expr_type: Box::new(array_type.clone_inner()) 
        })
    });
}

#[cfg(test)]
mod test {
    use crate::parser::expr::ExprType;

    #[test]
    pub fn test_match_type() {
        assert_ne!(ExprType::StringLiteral, ExprType::Char);
    }
}