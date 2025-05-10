use std::{cell::RefCell, collections::HashMap};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;

use crate::{full_result::{FullResult, OptionToFullResult}, lexer::token::{Kwrd, Op, Tkn, TknType}, parser::{expr::ExprTypeCons, tokens}};

use super::{accessors, expr::{Expr, ExprData, ExprType, VariableData}, stmt::{self, StmtData}, structs::Struct, ExprBump, FnParamBump, ParserError, StmtBump};

#[allow(non_camel_case_types)]
pub enum BuiltInFunction {
    print_string, print_i32,
    read_char, read_i32,
    panic
}

impl BuiltInFunction {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "print_string" => Some(BuiltInFunction::print_string),
            "print_i32" => Some(BuiltInFunction::print_i32),
            "read_char" => Some(BuiltInFunction::read_char),
            "read_i32" => Some(BuiltInFunction::read_i32),
            "panic" => Some(BuiltInFunction::panic),
            _ => None
        }
    }

    pub fn match_args(&self, left_args: &[Expr], right_args: &[Expr]) -> bool {
        let ExprType::Function {
            left_args: built_in_left_args,
            right_args: built_in_right_args,
            ..
        } = self.get_type() else {
            unreachable!("BuiltInFunction should be of type Function");
        };

        return left_args.iter().map(|Expr {expr_type, ..}| expr_type.clone_inner()).eq(built_in_left_args) &&
            right_args.iter().map(|Expr {expr_type, ..}| expr_type.clone_inner()).eq(built_in_right_args);
    }

    pub fn get_type(&self) -> ExprType {
        match self {
            BuiltInFunction::print_string => ExprType::Function { 
                name: String::from("print_string"), 
                return_type: Box::new(ExprType::Void), 
                left_args: vec![], 
                right_args: vec![
                    ExprType::StringLiteral
                ] 
            },
            BuiltInFunction::print_i32 => ExprType::Function { 
                name: String::from("print_i32"), 
                return_type: Box::new(ExprType::Void), 
                left_args: vec![], 
                right_args: vec![
                    ExprType::I32
                ] 
            },
            BuiltInFunction::read_char => ExprType::Function { 
                name: String::from("read_char"), 
                return_type: Box::new(
                    ExprType::AnonymousCustom { fields: Box::new([
                        (String::from("value"), ExprType::Char),
                        (String::from("success"), ExprType::Bool)
                    ]) 
                }), 
                left_args: vec![], 
                right_args: vec![] 
            },
            BuiltInFunction::read_i32 => ExprType::Function { 
                name: String::from("read_i32"), 
                return_type: Box::new(
                    ExprType::AnonymousCustom { fields: Box::new([
                        (String::from("value"), ExprType::I32),
                        (String::from("success"), ExprType::Bool)
                    ]) 
                }), 
                left_args: vec![], 
                right_args: vec![] 
            },
            BuiltInFunction::panic => ExprType::Function { 
                name: String::from("panic"), 
                return_type: Box::new(ExprType::Never), 
                left_args: vec![], 
                right_args: vec![
                    ExprType::StringLiteral
                ] 
            }
        }
    }
}

pub type Fun<'tkns, 'bumps, 'defs> = Function<'tkns, 'bumps, 'defs>;
#[derive(Clone, Debug)]
pub struct Function<'tkns, 'bumps, 'defs> {
    pub location: String,
    pub name: String,
    pub accessibility: String,
    pub mutable: bool,
    pub recursive: bool,
    pub left_args: &'bumps [FnParam<'tkns, 'bumps, 'defs>],
    pub right_args: &'bumps [FnParam<'tkns, 'bumps, 'defs>],
    pub return_type: ExprType,
    pub body: Vec<&'bumps StmtData<'bumps, 'defs>>,
}

pub type FnParam<'tkns, 'bumps, 'defs> = FunctionParamater<'tkns, 'bumps, 'defs>;
#[derive(Clone, Debug)]
pub struct FunctionParamater<'tkns, 'bumps, 'defs> {
    pub tkn: &'tkns Tkn,
    pub param_type: ExprType,
    pub param_name: Option<String>,
    pub param_default: Option<&'bumps ExprData<'bumps, 'defs>>,
}

#[derive(Clone, Debug)]
pub struct FunctionDefinition<'tkns> {
    pub accessibility: Option<&'tkns Tkn>,
    pub name: String,
    pub mutable: bool,
    pub recursive: bool,
    pub arg_tokens: &'tkns [Tkn],
    pub body_tokens: &'tkns [Tkn],
}

pub type FullFnDef<'tkns, 'bumps, 'defs> = FullFunctionDefinition<'tkns, 'bumps, 'defs>;
#[derive(Clone, Debug)]
pub struct FullFunctionDefinition<'tkns, 'bumps, 'defs> {
    pub accessibility: String,
    pub name: String,
    pub mutable: bool,
    pub recursive: bool,
    pub left_args: &'bumps [FnParam<'tkns, 'bumps, 'defs>],
    pub right_args: &'bumps [FnParam<'tkns, 'bumps, 'defs>],
    pub return_type: ExprType
}

impl<'tkns, 'bumps, 'defs> FullFnDef<'tkns, 'bumps, 'defs> {
    pub fn from_partial_fn_def(
        fn_param_bump: &'bumps FnParamBump,
        fn_def: FunctionDefinition<'tkns>, 
        accessors: &[&str],
        struct_names: &[&str]
    ) -> Result<
        (FullFnDef<'tkns, 'bumps, 'defs>, &'tkns [Tkn]), 
        ParserError<'tkns, 'bumps, 'defs>
    > {
        let FunctionDefinition {
            accessibility,
            name,
            mutable,
            recursive,
            arg_tokens,
            body_tokens
        } = fn_def;

        let accessibility = accessors::get_accessor_string(accessibility, &mut 0, &accessors)
            .ok_or_else(|| ParserError::AccessorNotDefined { 
                tkn: accessibility.unwrap() 
            })?;

        let (left_args, right_args, return_type) = define_arguments(
            fn_param_bump,
            arg_tokens,
            &struct_names
        )?;
        
        return Ok((FullFnDef {
            accessibility,
            name: name.clone(),
            mutable: mutable,
            recursive: recursive,
            left_args,
            right_args,
            return_type
        }, body_tokens));
    }
}

#[derive(PartialEq, Debug)]
pub enum FnType {
    Prefix, Infix, Postfix
}

pub type FnParseState = FunctionParseState;
#[derive(PartialEq)]
pub enum FunctionParseState {
    NoDef,              // ~~
    NoDefPrefix,        // prefix ~~
    FullyDefPrefix,     // prefix done
    ArgsDef,            // done ~~
    ParDefInfix,        // done infix ~~
    FullyDefInfix,      // done infix done
    FullyDefPostfix,    // done postfix
}

pub type Enc = Encapsulation;
#[derive(PartialEq)]
pub enum Encapsulation {
    Parenthesis,
    Dollar
}

pub fn define_function<'tkns, 'bumps, 'defs>(
    tokens: &'tkns [Tkn], 
    index: &mut usize,
) -> FullResult<FunctionDefinition<'tkns>, (), ParserError<'tkns, 'bumps, 'defs>> {
    let mut peek = *index;
    let accessibility = match tokens[peek].token {
        TknType::Keyword(Kwrd::Public)
        | TknType::Keyword(Kwrd::Private)
        | TknType::Keyword(Kwrd::Package) 
        | TknType::Identifier(_) => {
            peek += 1;
            Some(&tokens[peek - 1])
        },
        TknType::Keyword(Kwrd::Mutable)
        | TknType::Keyword(Kwrd::Recursive)
        | TknType::Keyword(Kwrd::Function) => {
            None
        },
        _ => {
            return FullResult::SoftErr(());
        },
    };

    let mutable = tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Mutable), &mut peek);
    let recursive = tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Recursive), &mut peek);

    let name;
    tokens::expect_token(tokens, TknType::Keyword(Kwrd::Function), &mut peek).ok_or_soft(())?;
    if let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) {
        peek += 1;
        name = ident.clone();
    } else {
        return FullResult::HardErr(ParserError::ExpectedIdentifier { tkn: &tokens[peek] });
    }

    let arg_tokens: &[Tkn];
    let body_tokens: &[Tkn];
    let mut start = peek;
    let mut end;
    // since anonymous structs use curly braces we check for whenever types are declared for curly braces
    // and because anonymous structs can be nested, we have to keep track of 
    // how far down the rabbit hole we are
    let mut parsing_type_level: usize = 0;
    loop {
        if tokens::is_expected_token(tokens, TknType::OpenCurlyBrace, &mut peek) {
            if parsing_type_level > 0 {
                //Technically this should never happen in correct code
                //However we pass this error down further in the parsing step
                continue;
            }
            peek -= 1;
            end = peek;
            arg_tokens = &tokens[start..end];
            start = peek;
            break;
        } else if tokens::is_expected_tokens(
            tokens, 
            &[TknType::Colon, TknType::OpenCurlyBrace], 
            &mut peek
        ) {
            parsing_type_level += 1;
            continue;
        } else if tokens::is_expected_tokens(
            tokens, 
            &[TknType::Operation(Op::Assign), TknType::OpenCurlyBrace], 
            &mut peek
        ) {
            parsing_type_level += 1; 
            continue;
        } else if tokens::is_expected_token(tokens, TknType::CloseCurlyBrace, &mut peek) {
            if parsing_type_level == 0 {
                //Technically this should never happen in correct code
                //However we pass this error down further in the parsing step
                continue;
            }

            parsing_type_level -= 1;
            continue;
        } 
        peek += 1;
    }
    let mut count = 0;
    loop {
        if tokens::is_expected_token(tokens, TknType::OpenCurlyBrace, &mut peek) {
            count += 1;
            continue;
        } else if tokens::is_expected_token(tokens, TknType::CloseCurlyBrace, &mut peek) {
            if count > 1 {
                count -= 1;
                continue;
            } 

            end = peek - 1;
            body_tokens = &tokens[start..=end];
            *index = peek;
            return FullResult::Ok(FunctionDefinition {
                accessibility,
                name,
                mutable,
                recursive,
                arg_tokens,
                body_tokens,
            });
        }
        peek += 1;
    }
}

pub fn define_arguments<'tkns, 'bumps, 'defs>(
    fn_param_bump: &'bumps FnParamBump,
    tokens: &'tkns [Tkn],
    structs: &[&str],
) -> Result<
    (
        &'bumps [FnParam<'tkns, 'bumps, 'defs>], 
        &'bumps [FnParam<'tkns, 'bumps, 'defs>], 
        ExprType
    ), 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = 0;
    let fn_params_left: &[FnParam];
    let mut fn_params_right: &[FnParam];
    let mut return_type: ExprType = ExprType::Void;
    let mut id: Option<(FnType, &Tkn)> = None;

    let is_expected_prefix = tokens::is_expected_tokens(
        tokens,
        &[
            TknType::OpenParen,
            TknType::Keyword(Kwrd::Prefix),
            TknType::CloseParen,
        ],
        &mut peek,
    ) || tokens::is_expected_tokens(
        tokens,
        &[TknType::Dollar, TknType::Keyword(Kwrd::Prefix)],
        &mut peek,
    ) || tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Prefix), &mut peek);

    if is_expected_prefix {
        id = Some((FnType::Prefix, &tokens[peek - 1]));
    }

    fn_params_right = fn_group_to_params(fn_param_bump, tokens, &mut peek, &structs)?;

    if let Some((FnType::Prefix, id_token)) = id {
        if tokens::is_expected_token(tokens, TknType::Operation(Op::Assign), &mut peek) {
            return_type = super::get_type_token_expr_type(
                tokens,
                &mut peek,
                structs,
            ).ok_or_else(|| ParserError::ExpectedType { tkn: &tokens[peek] })?;
            fn_params_left = &[];
            return Ok((fn_params_left, fn_params_right, return_type));
        } else if peek >= tokens.len() {
            fn_params_left = &[];
            return Ok((fn_params_left, fn_params_right, return_type));
        } else {
            return Err(ParserError::DefinedIncorrectlyPlacedArgument { 
                tkn: &tokens[peek], 
                arg_type: FnType::Postfix,
                fix_defined: id_token,
                fix_type: FnType::Prefix 
            });
        }
    }

    let start_fix = peek;
    let is_expected_infix = tokens::is_expected_tokens(
        tokens,
        &[
            TknType::OpenParen,
            TknType::Keyword(Kwrd::Infix),
            TknType::CloseParen,
        ],
        &mut peek,
    ) || tokens::is_expected_tokens(
        tokens,
        &[TknType::Dollar, TknType::Keyword(Kwrd::Infix)],
        &mut peek,
    ) || tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Infix), &mut peek);

    let is_expected_postfix = tokens::is_expected_tokens(
        tokens,
        &[
            TknType::OpenParen,
            TknType::Keyword(Kwrd::Postfix),
            TknType::CloseParen,
        ],
        &mut peek,
    ) || tokens::is_expected_tokens(
        tokens,
        &[TknType::Dollar, TknType::Keyword(Kwrd::Postfix)],
        &mut peek,
    ) || tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Postfix), &mut peek);

    if is_expected_infix {
        if let Some((_, id_token)) = id {
            return Err (ParserError::ConflictingFunctionFixDefinitions { 
                tkn: &tokens[start_fix], 
                fix_defined: id_token
            });
        }
        fn_params_left = fn_params_right;

        fn_params_right = fn_group_to_params(&fn_param_bump, tokens, &mut peek, structs)?;
        if tokens::is_expected_token(tokens, TknType::Operation(Op::Assign), &mut peek) {
            return_type = super::get_type_token_expr_type(
                tokens,
                &mut peek,
                structs,
            ).ok_or_else(|| ParserError::ExpectedType { tkn: &tokens[peek] })?;
            return Ok((fn_params_left, fn_params_right, return_type));
        } else if peek >= tokens.len() {
            return Ok((fn_params_left, fn_params_right, return_type));
        } else {
            return Err(ParserError::ExpectedTokens { 
                tkn: &tokens[peek],
                received: tokens[peek..].iter().map(|tkn| &tkn.token), 
                expected: &[TknType::Operation(Op::Assign), TknType::Colon, TknType::OpenCurlyBrace] 
            });
        }
    } else if is_expected_postfix {
        if let Some((_, id_token)) = id {
            return Err(ParserError::ConflictingFunctionFixDefinitions { 
                tkn: &tokens[start_fix], 
                fix_defined: id_token
            });
        }
        fn_params_left = fn_params_right;
        fn_params_right = &[];

        if tokens::is_expected_token(tokens, TknType::Operation(Op::Assign), &mut peek) {
            return_type = super::get_type_token_expr_type(
                tokens,
                &mut peek,
                structs,
            ).ok_or_else(|| ParserError::ExpectedType { tkn: &tokens[peek] })?;
            return Ok((fn_params_left, fn_params_right, return_type));
        } else if peek >= tokens.len() {
            return Ok((fn_params_left, fn_params_right, return_type));
        } else {
            return Err(ParserError::DefinedIncorrectlyPlacedArgument{ 
                tkn: &tokens[peek], 
                arg_type: FnType::Prefix,
                fix_defined: &tokens[start_fix],
                fix_type: FnType::Postfix 
            });
        }
    } else if tokens::is_expected_token(tokens, TknType::Operation(Op::Assign), &mut peek) {
        return_type = super::get_type_token_expr_type(
            tokens, 
            &mut peek, 
            structs
        ).ok_or_else(|| ParserError::ExpectedType { tkn: &tokens[peek] })?;
        fn_params_left = &[];
        return Ok((fn_params_left, fn_params_right, return_type));
    } else if peek >= tokens.len() {
        fn_params_left = &[];
        return Ok((fn_params_left, fn_params_right, return_type));
    } else {
        return Err(ParserError::ExpectedEndOfFunctionDefinition { tkn: &tokens[peek - 1] });
    }
}

pub fn parse_function<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    fn_def: FullFnDef<'tkns, 'bumps, 'defs>,
    tokens: &'tkns [Tkn]
) -> Result<Fun<'tkns, 'bumps, 'defs>, ParserError<'tkns, 'bumps, 'defs>> {
    let FullFnDef {
        accessibility, name, mutable, recursive, left_args, right_args, mut return_type
    } = fn_def;

    for arg in left_args {
        let variable_name = arg.param_name.as_ref().unwrap().clone();
        let param_type = arg.param_type.clone();
        
        variables.push(variable_name, VariableData::new(arg.tkn, false, ExprTypeCons::new(expr_bump, param_type)));
    }

    for arg in right_args {
        let variable_name = arg.param_name.as_ref().unwrap().clone();
        let param_type = arg.param_type.clone();
        
        variables.push(variable_name, VariableData::new(arg.tkn, false, ExprTypeCons::new(expr_bump, param_type)));
    }

    let mut peek = 0;
    let mut stmts: Vec<&StmtData> = vec![];
    let body_tokens = &tokens[1..tokens.len()-1];
    {
        let variables = variables.new_frame();
        
        loop {
            if peek >= body_tokens.len() {
                break;
            }
            
            let stmt_possible = stmt::parse_statement(
                expr_bump, 
                stmt_bump, 
                structs,
                functions, 
                &variables, 
                body_tokens, 
                &mut return_type,
                &mut peek
            );
            match stmt_possible {
                FullResult::Ok(mut stmt) => stmts.append(&mut stmt),
                FullResult::SoftErr(_) => break,
                FullResult::HardErr(err) => return Err(err)
            }
        }
}

    return Ok(Function {
        accessibility,
        name,
        location: "".to_string(),
        mutable,
        recursive,
        left_args,
        right_args,
        return_type,
        body: stmts
    });
}

pub fn expect_closing_group(
    tokens: &[Tkn], 
    expect_closing: bool, 
    index: &mut usize
) -> bool {
    if expect_closing && tokens[*index].token == TknType::CloseParen
    {
        *index += 1;
        return true;
    } else if !expect_closing && (
        *index >= tokens.len() ||
        tokens[*index].token == TknType::Dollar || 
        tokens[*index].token == TknType::Operation(Op::Assign) || 
        tokens[*index].token == TknType::OpenCurlyBrace ||
        tokens[*index].token == TknType::Keyword(Kwrd::Infix) ||
        tokens[*index].token == TknType::Keyword(Kwrd::Postfix)
    ) {
        return true;
    }
    return false;
}

pub fn fn_group_to_params<'tkns, 'bumps, 'defs>(
    fn_param_bump: &'bumps FnParamBump,
    tokens: &'tkns [Tkn],  
    index: &mut usize, 
    structs: &[&str]
) -> Result<
    &'bumps [FnParam<'tkns, 'bumps, 'defs>], 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut fn_params: Vec<FnParam> = vec![];

    let expect_closing;
    if tokens::is_expected_token(tokens, TknType::OpenParen, index) {
        expect_closing = true;
    } else if tokens::is_expected_token(tokens, TknType::Dollar, index) {
        expect_closing = false;
    } else {
        *index += 1;
        return Ok(fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter()));
    }

    let mut require_comma = false;
    loop {
        if expect_closing_group(tokens, expect_closing, index) {
            return Ok(fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter()));
        }
        
        if require_comma {
            return Err(ParserError::ExpectedToken {
                tkn: &tokens[*index],
                expected: TknType::Comma
            });
        }
        
        let Some(TknType::Identifier(ident)) = tokens::get_token(tokens, *index).map(|e| &e.token) else {
            return Err(ParserError::ExpectedIdentifier { tkn: &tokens[*index] });
        };
        let tkn = &tokens[*index];

        *index += 1;

        tokens::expect_token(tokens, TknType::Colon, index)
            .expect(format!("Expected Colon at {}", tokens[*index]).as_str());

        let expr_type = super::get_type_token_expr_type(
            tokens, 
            index, 
            &structs
        ).expect(format!("Expected Type at {}", tokens[*index - 1]).as_str());

        let fn_param = FnParam {
            tkn,
            param_type: expr_type,
            param_name: Some(ident.clone()),
            param_default: None,
        };

        fn_params.push(fn_param);

        if !tokens::is_expected_token(tokens, TknType::Comma, index) {
            require_comma = true;
        }
    }
}