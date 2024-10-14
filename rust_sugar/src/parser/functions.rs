use std::{cell::RefCell, collections::HashMap};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;

use crate::{lexer::token::{Kwrd, Op, Tkn, TknType}, parser::{expr::ExprTypeCons, tokens}};

use super::{accessors, expr::{ExprData, ExprType, VariableData}, stmt::{self, Stmt}, ExprBump, FnParamBump, ParserError, StmtBump};

pub type Fun<'f> = Function<'f>;
#[derive(Clone, Debug)]
pub struct Function<'f> {
    pub location: String,
    pub name: String,
    pub accessibility: String,
    pub mutable: bool,
    pub recursive: bool,
    pub left_args: &'f [FnParam<'f>],
    pub right_args: &'f [FnParam<'f>],
    pub return_type: ExprType,
    pub body: Vec<&'f Stmt<'f>>,
}

pub type FnParam<'fp> = FunctionParamater<'fp>;
#[derive(Clone, Debug)]
pub struct FunctionParamater<'fp> {
    pub param_type: ExprType,
    pub param_name: Option<String>,
    pub param_default: Option<&'fp ExprData<'fp>>,
}

impl<'fp> FunctionParamater<'fp> {
    pub fn default() -> FunctionParamater<'fp> {
        return FunctionParamater {
            param_type: ExprType::Void,
            param_name: None,
            param_default: None,
        };
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDefinition<'df> {
    pub accessibility: Option<&'df Tkn<'df>>,
    pub name: String,
    pub mutable: bool,
    pub recursive: bool,
    pub arg_tokens: &'df [Tkn<'df>],
    pub body_tokens: &'df [Tkn<'df>],
}

#[derive(Clone, Debug)]
pub struct FullFunctionDefinition<'dff> {
    pub accessibility: String,
    pub name: String,
    pub mutable: bool,
    pub recursive: bool,
    pub left_args: &'dff [FnParam<'dff>],
    pub right_args: &'dff [FnParam<'dff>],
    pub return_type: ExprType
}

impl<'dff> FullFunctionDefinition<'dff> {
    pub fn from_partial_fn_def<'fpfd>(
        fn_param_bump: &'fpfd FnParamBump,
        fn_def: FunctionDefinition<'fpfd>, 
        accessors: &[&str],
        struct_names: &[&str]
    ) -> Result<(FullFunctionDefinition<'fpfd>, &'fpfd [Tkn<'fpfd>]), ParserError<'fpfd>> {
        let FunctionDefinition {
            accessibility,
            name,
            mutable,
            recursive,
            arg_tokens,
            body_tokens
        } = fn_def;

        let accessibility = match accessors::get_accessor_string(accessibility, &mut 0, &accessors) {
            Some(accessor) => accessor,
            None => return Err(ParserError::new(accessibility.unwrap(), "Accessor is not defined"))
        };

        let (left_args, right_args, return_type) = define_arguments(
            fn_param_bump,
            arg_tokens,
            &struct_names
        )?;
        
        return Ok((FullFunctionDefinition {
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

#[derive(PartialEq)]
pub enum FnType {
    Prefix, Infix, Postfix
}

pub type ArgDef = ArgumentDefinitionType;
#[derive(PartialEq)]
pub enum ArgumentDefinitionType {
    JustTypes, JustIdents, Both
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

pub fn define_function<'df>(
    tokens: &'df [Tkn<'df>], 
    index: &mut usize,
) -> Option<FunctionDefinition<'df>> {
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
            return None;
        },
    };

    let mutable = tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Mutable), &mut peek);
    let recursive = tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Recursive), &mut peek);

    let name;
    tokens::expect_token(tokens, TknType::Keyword(Kwrd::Function), &mut peek)?;
    if let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) {
        peek += 1;
        name = ident.clone();
    } else {
        return None;
    }

    let arg_tokens: &[Tkn];
    let body_tokens: &[Tkn];
    let mut start = peek;
    let mut end;
    loop {
        if tokens::is_expected_token(tokens, TknType::Semicolon, &mut peek) {
            end = peek - 1;
            arg_tokens = &tokens[start..end];
            body_tokens = &[];
            *index = peek;
            return Some(FunctionDefinition {
                accessibility,
                name,
                mutable,
                recursive,
                arg_tokens,
                body_tokens,
            });
        } else if tokens::is_expected_token(tokens, TknType::OpenCurlyBrace, &mut peek) {
            peek -= 1;
            end = peek;
            arg_tokens = &tokens[start..end];
            start = peek;
            break;
        } else if tokens::is_expected_token(tokens, TknType::Operation(Op::Arrow), &mut peek) {
            end = peek - 1;
            arg_tokens = &tokens[start..end];
            start = peek;
            //TODO: Add support for compound statements
            loop {
                if tokens::is_expected_token(tokens, TknType::Semicolon, &mut peek) {
                    end = peek - 1;
                    body_tokens = &tokens[start..=end];
                    *index = peek;
                    return Some(FunctionDefinition {
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
            } else {
                end = peek - 1;
                body_tokens = &tokens[start..=end];
                *index = peek;
                return Some(FunctionDefinition {
                    accessibility,
                    name,
                    mutable,
                    recursive,
                    arg_tokens,
                    body_tokens,
                });
            }
        }
        peek += 1;
    }
}

pub fn define_arguments<'da>(
    fn_param_bump: &'da FnParamBump,
    tokens: &'da [Tkn],
    structs: &[&str],
) -> Result<(&'da [FnParam<'da>], &'da [FnParam<'da>], ExprType), ParserError<'da>> {
    let mut peek = 0;
    let fn_params_1: &[FnParam];
    let mut fn_params_2: &[FnParam];
    let mut return_type: ExprType = ExprType::Void;
    let mut id: Option<FnType> = None;

    if tokens::is_expected_tokens(
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
    ) || tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Prefix), &mut peek)
    {
        id = Some(FnType::Prefix);
    }
    fn_params_2 = fn_group_to_params(fn_param_bump, tokens, &mut peek, &structs);

    if let Some(FnType::Prefix) = id {
        if tokens::is_expected_token(tokens, TknType::Colon, &mut peek) {
            return_type = super::get_type_token_expr_type(
                tokens::get_token(tokens, peek),
                &mut peek,
                structs,
            );
            fn_params_1 = &[];
            return Ok((fn_params_1, fn_params_2, return_type));
        } else if peek >= tokens.len() {
            fn_params_1 = &[];
            return Ok((fn_params_1, fn_params_2, return_type));
        } else {
            return Err(ParserError::new(&tokens[peek], "Expected end of argument definition"));
        }
    }

    if tokens::is_expected_tokens(
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
    ) || tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Infix), &mut peek)
    {
        if id != None {
            return Err (ParserError::new(&tokens[peek - 1],
                "function cannot be defined as infix, 
                because it is already defined as prefix"
            ));
        }
        //id = Some(FnType::Infix);
        fn_params_1 = fn_params_2;
        //fn_params_2 = vec![];

        fn_params_2 = fn_group_to_params(&fn_param_bump, tokens, &mut peek, structs);
        if tokens::is_expected_token(tokens, TknType::Colon, &mut peek) {
            return_type = super::get_type_token_expr_type(
                tokens::get_token(tokens, peek),
                &mut peek,
                structs,
            );
            return Ok((fn_params_1, fn_params_2, return_type));
        } else if peek >= tokens.len() {
            return Ok((fn_params_1, fn_params_2, return_type));
        } else {
            return Err(ParserError::new(
                &tokens[peek],
                "expected a colon, semicolon, or beginning of function body"
            ));
        }
    } else if tokens::is_expected_tokens(
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
    ) || tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Postfix), &mut peek)
    {
        if id != None {
            return Err(ParserError::new(
                &tokens[peek - 1],
                "function cannot be defined as postfix, 
                because it is already defined as prefix"
            ));
        }
        //id = Some(FnType::Postfix);
        fn_params_1 = fn_params_2;
        fn_params_2 = &[];

        if tokens::is_expected_token(tokens, TknType::Colon, &mut peek) {
            return_type = super::get_type_token_expr_type(
                tokens::get_token(tokens, peek),
                &mut peek,
                structs,
            );
            return Ok((fn_params_1, fn_params_2, return_type));
        } else if peek >= tokens.len() {
            return Ok((fn_params_1, fn_params_2, return_type));
        } else {
            return Err(ParserError::new(&tokens[peek], "expected end of argument definition"));
        }
    } else if tokens::is_expected_token(tokens, TknType::Colon, &mut peek) {
        return_type = super::get_type_token_expr_type(
            tokens::get_token(tokens, peek), 
            &mut peek, 
            structs
        );
        fn_params_1 = &[];
        return Ok((fn_params_1, fn_params_2, return_type));
    } else if peek >= tokens.len() {
        fn_params_1 = &[];
        return Ok((fn_params_1, fn_params_2, return_type));
    } else {
        return Err(ParserError::new(&tokens[peek], "expected end of argument definition"));
    }
}

pub fn parse_function<'pf, 'hm, 'i>(
    expr_bump: &'pf ExprBump,
    stmt_bump: &'pf StmtBump,
    functions: &'hm RefCell<HashMap<String, FullFunctionDefinition<'pf>>>,
    variables: StackFrameDictAllocator<'i, String, VariableData<'i>>,
    fn_def: FullFunctionDefinition<'pf>,
    tokens: &'pf [Tkn<'pf>]
) -> Result<Fun<'pf>, ParserError<'pf>> where 'pf: 'hm {
    let FullFunctionDefinition {
        accessibility, name, mutable, recursive, left_args, right_args, return_type
    } = fn_def;

    for arg in left_args {
        let variable_name = arg.param_name.as_ref().unwrap().clone();
        let param_type = arg.param_type.clone();
        
        variables.push(variable_name, VariableData::new(false, ExprTypeCons::new_temp(param_type)));
    }

    for arg in right_args {
        let variable_name = arg.param_name.as_ref().unwrap().clone();
        let param_type = arg.param_type.clone();
        
        variables.push(variable_name, VariableData::new(false, ExprTypeCons::new_temp(param_type)));
    }

    let mut peek = 0;
    let mut stmts: Vec<&Stmt> = vec![];
    let body_tokens = &tokens[1..tokens.len()];
    {
        let variables = variables.new_frame();
        
        loop {
            let stmt_possible = stmt::parse_statement(
                expr_bump, 
                stmt_bump, 
                functions, 
                &variables, 
                body_tokens, 
                &mut peek
            );
            match stmt_possible {
                Ok(mut stmt) => {
                    stmts.append(&mut stmt);
                },
                Err(_) => {
                    break;
                }
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
    encapsulation: &Encapsulation, 
    index: &mut usize
) -> bool {
    if encapsulation == &Encapsulation::Parenthesis 
        && tokens[*index].token == TknType::CloseParen
    {
        *index += 1;
        return true;
    }
    if encapsulation == &Encapsulation::Dollar 
        && (tokens[*index].token == TknType::Dollar
        || tokens[*index].token == TknType::Colon 
        || tokens[*index].token == TknType::OpenCurlyBrace)
    {
        return true;
    }
    return false;
}

pub fn fn_group_to_params<'fp>(
    fn_param_bump: &'fp FnParamBump,
    tokens: &'fp [Tkn<'fp>],  
    index: &mut usize, 
    structs: &[&str]
) -> &'fp [FnParam<'fp>] {
    let mut fn_param: FnParam = FnParam::default();
    let mut fn_params: Vec<FnParam> = vec![];
    let mut just_types: Option<ArgDef> = None;
    let require_commas: bool;

    let encapsulation: Encapsulation;
    if tokens::is_expected_token(tokens, TknType::OpenParen, index) {
        encapsulation = Encapsulation::Parenthesis;
    } else if tokens::is_expected_token(tokens, TknType::Dollar, index) {
        encapsulation = Encapsulation::Dollar;
    } else {
        *index += 1;
        return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
    }

    let expr_type = super::get_type_token_expr_type(
        tokens::get_token(tokens, *index), 
        index, &structs
    );
    if expr_type != ExprType::Void {
        fn_param.param_type = expr_type;
    } else if let Some(Tkn {
        token: TknType::Identifier(ident), ..
    }) = tokens::get_token(tokens, *index) {
        *index += 1;
        fn_param.param_name = Some(ident.clone());
        fn_params.push(fn_param.clone());
        fn_param = FnParam::default();
        just_types = Some(ArgDef::JustIdents);
    } else if encapsulation == Enc::Parenthesis 
        && tokens::is_expected_token(tokens, TknType::CloseParen, index) 
    {
        return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
    } else {
        panic!("unexpected token {:?}, expected type, identifier or )", tokens::get_token(tokens, *index));
    }

    if encapsulation == Enc::Parenthesis 
        && tokens::is_expected_token(tokens, TknType::CloseParen, index) 
    {
        if just_types == None {
            //just_types = Some(ArgDef::JustTypes); 
        }
        fn_params.push(fn_param.clone());
        //fn_param = FnParam::default();
        return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
    }

    if tokens::is_expected_token(tokens, TknType::Comma, index) {
        if just_types == None { 
            just_types = Some(ArgDef::JustTypes); 
            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();
        }
        require_commas = true;
    } else {
        require_commas = false;
    }

    if just_types == None {
        let expr_type = super::get_type_token_expr_type(
            tokens::get_token(tokens, *index), 
            index, &structs
        );
        if expr_type != ExprType::Void {
            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();

            fn_param.param_type = expr_type;
            just_types = Some(ArgDef::JustTypes);
        } else if let Some(Tkn {
            token: TknType::Identifier(ident), ..
        }) = tokens::get_token(tokens, *index) {
            *index += 1;
            fn_param.param_name = Some(ident.clone());

            //check for default value

            just_types = Some(ArgDef::Both);
        }

        fn_params.push(fn_param.clone());
        fn_param = FnParam::default();
    }
    
    if just_types == Some(ArgDef::JustTypes) {
        loop {
            if require_commas {
                let comma = tokens::is_expected_token(
                    tokens, 
                    TknType::Comma, 
                    index
                );
                if expect_closing_group(
                    tokens,  
                    &encapsulation, 
                    index
                ) {
                    
                    return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
                }
                if !comma {panic!("expected comma at {}", *index);}
            } else if tokens::is_expected_token(tokens, TknType::Comma, index)  { 
                panic!("invalid comma at {}", *index);
            }
            let expr_type = super::get_type_token_expr_type(
                tokens::get_token(tokens, *index), 
                index, &structs
            );
            if expr_type != ExprType::Void {
                fn_param.param_type = expr_type;
            } else if expect_closing_group(
                tokens, 
                &encapsulation, 
                index
            ) {
                return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
            } else {
                panic!("unexpected token {}, was expecting type", tokens[*index]);
            }

            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();
        }
    } else if just_types == Some(ArgDef::JustIdents) {
        loop {
            if require_commas {
                let comma = tokens::is_expected_token(tokens, TknType::Comma, index);
                if expect_closing_group(
                    tokens, 
                    &encapsulation, 
                    index
                ) {
                    return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
                }
                if !comma {panic!("expected comma at {}", *index);}
            } else if tokens::is_expected_token(tokens, TknType::Comma, index) { 
                panic!("invalid comma at {}", *index);
            }
            
            if let Some(ident) = super::get_ident_token_string(
                tokens::get_token(tokens, *index), 
                index, &structs) 
            {
                fn_param.param_name = Some(ident.clone());
            } else if expect_closing_group(
                tokens,  
                &encapsulation, 
                index
            ) {
                return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
            } else {
                panic!("unexpected token {:?}, expected identifier, not type", tokens::get_token(tokens, *index - 1));
            }

            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();
        }
    } else if just_types == Some(ArgDef::Both) {
        loop {
            if require_commas {
                let comma = tokens::is_expected_token(tokens, TknType::Comma, index);
                if expect_closing_group(
                    tokens, 
                    &encapsulation, 
                    index
                ) {
                    return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
                }
                if !comma {panic!("expected comma at {}", *index);}
            } else if tokens::is_expected_token(tokens, TknType::Comma, index) { 
                panic!("invalid comma at {}", *index);
            }

            let expr_type = super::get_type_token_expr_type(
                tokens::get_token(tokens, *index), 
                index, &structs
            );
            if expr_type != ExprType::Void {
                fn_param.param_type = expr_type;
            } else if expect_closing_group(
                tokens, 
                &encapsulation, 
                index
            ) {
                return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
            } else {
                panic!("unexpected token {}, was expecting type", tokens[*index]);
            }

            if let Some(ident) = super::get_ident_token_string(
                tokens::get_token(tokens, *index), 
                index, &structs)
            {
                fn_param.param_name = Some(ident);
            } else if expect_closing_group(
                tokens, 
                &encapsulation, 
                index
            ) {
                return fn_param_bump.alloc_slice_fill_iter(fn_params.into_iter());
            } else {
                panic!("unexpected token {}, expected identifier, not type", tokens[*index]);
            }

            //check for default value

            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();
        } 
    }
    unreachable!();
}