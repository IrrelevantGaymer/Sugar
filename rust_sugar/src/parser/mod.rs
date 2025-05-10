use std::{cell::RefCell, collections::HashMap, ops::Deref};

use accessors::{Accessor, AccessorDefinition};
use bumpalo::Bump;
use expr::{ExprType, VariableData};
use functions::{define_function, FullFunctionDefinition, Fun, FunctionDefinition};
use once_cell::sync::OnceCell;
use parser_error::ParserError;
use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;
use structs::{Struct, StructDefinition};

use crate::{full_result::FullResult, lexer::token::{Tkn, TknType}};

pub mod accessors;
pub mod expr;
pub mod functions;
pub mod operators;
pub mod parser_error;
pub mod patterns;
pub mod stmt;
pub mod structs;
pub mod tokens;

pub type ExprBump = ExpressionBumpAllocator;
pub type StmtBump = StatementBumpAllocator;
pub type FnParamBump = FunctionParameterBumpAllocator;

pub struct ExpressionBumpAllocator(Bump);

impl ExprBump {
    pub fn new() -> Self {
        ExpressionBumpAllocator(Bump::new())
    }
}

impl Deref for ExprBump {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct StatementBumpAllocator(Bump);

impl StmtBump {
    pub fn new() -> Self {
        StatementBumpAllocator(Bump::new())
    }
}

impl Deref for StmtBump {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FunctionParameterBumpAllocator(Bump);

impl FunctionParameterBumpAllocator {
    pub fn new() -> Self {
        FunctionParameterBumpAllocator(Bump::new())
    }
}

impl Deref for FnParamBump {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    fn_param_bump: &'bumps FnParamBump,
    accessors: &'defs OnceCell<Box<[Accessor]>>,
    structs: &'defs OnceCell<Box<[Struct]>>,
    functions: &'defs OnceCell<Box<[Fun<'tkns, 'bumps, 'defs>]>>,
    tokens: &'tkns [Tkn]
) -> Result<(), Vec<ParserError<'tkns, 'bumps, 'defs>>> {
    let mut errors: Vec<ParserError> = vec![];

    let function_data = RefCell::new(HashMap::new());
    let variables = StackFrameDictAllocator::<String, VariableData>::new();
    
    let mut accessor_defs: Vec<AccessorDefinition> = vec![];
    let mut struct_defs: Vec<StructDefinition> = vec![];
    let mut function_defs: Vec<FunctionDefinition> = vec![];

    let mut index: usize = 0;
    while index < tokens.len() {
        match accessors::define_accessor(tokens, &mut index) {
            FullResult::Ok(def) => {
                accessor_defs.push(def);
                continue;
            },
            FullResult::SoftErr(()) => (),
            FullResult::HardErr(error) => return Err(error)
        }
        match structs::define_struct(tokens, &mut index) {
            FullResult::Ok(def) => {
                struct_defs.push(def);
                continue;
            },
            FullResult::SoftErr(()) => (),
            FullResult::HardErr(error) => return Err(error)
        }
        match define_function(tokens, &mut index) {
            FullResult::Ok(def) => {
                function_defs.push(def);
                continue;
            },
            FullResult::SoftErr(()) => (),
            FullResult::HardErr(error) => return Err(vec![error])
        }

        if tokens::is_expected_token(tokens, TknType::EndOfFile, &mut index) {
            break;
        } else {
            errors.push(ParserError::InvalidBlock{ tkn: &tokens[index] });
            eprintln!("accessors: {accessor_defs:#?}, structs: {struct_defs:#?}, functions: {function_defs:#?}");
            return Err(errors);
        }
    }

    let accessor_names = accessor_defs.iter()
        .map(|AccessorDefinition {name, ..}|
            name.as_str()
        ).collect::<Vec<&str>>();

    let struct_names = struct_defs
        .iter()
        .map(|StructDefinition {name, ..} | {
            name.as_str()
        })
        .collect::<Vec<&str>>();

    let mut accessor_buffer = vec![];
    for i in &accessor_defs {
        match accessors::parse_accessor(i) {
            Ok(accessor) => accessor_buffer.push(accessor),
            Err(ref mut err) => errors.append(err)
        }
    }
    accessors.set(accessor_buffer.into_boxed_slice()).unwrap();

    let mut struct_buffer = vec![];
    for struct_def in &struct_defs {
        match structs::parse_struct(
            struct_def, &accessor_names, &struct_names
        ) {
            Ok(structure) => struct_buffer.push(structure),
            Err(err) => errors.push(err),
        };
    }
    structs.set(struct_buffer.into_boxed_slice()).unwrap();

    let mut full_function_defs = vec![];
    for function_def in function_defs {
        let (
            full_function_def, 
            body_tokens
        ) = match FullFunctionDefinition::from_partial_fn_def(
            &fn_param_bump, 
            function_def, 
            &accessor_names, 
            &struct_names
        ) {
            Ok((full_function_def, body_tokens)) => (full_function_def, body_tokens),
            Err(err) => {
                errors.push(err);
                continue;
            }
        };

        function_data.borrow_mut().insert(full_function_def.name.clone(), full_function_def.clone());
        full_function_defs.push((full_function_def, body_tokens));
    }

    let mut function_buffer = vec![];
    for (full_function_def, body_tokens) in full_function_defs {
        if body_tokens.is_empty() {
            continue;
        } else {
            let function = match functions::parse_function(
                &expr_bump, 
                &stmt_bump, 
                structs.get().unwrap(),
                &function_data, 
                variables.new_frame(), 
                full_function_def, 
                body_tokens
            ) {
                Ok(function) => function,
                Err(err) => {
                    errors.push(err);
                    continue;
                }
            };
            function_buffer.push(function);
        }
    }

    functions.set(function_buffer.into_boxed_slice()).unwrap();
    
    if errors.is_empty() {
        return Ok(());
    }
    return Err(errors);
}

pub fn get_type(
    tokens: &[Tkn],
    index: &mut usize,
    structs: &[Struct]
) -> Option<ExprType> {
    if let Some(Tkn {token: TknType::Type(typ), ..}) = tokens.get(*index) {
        *index += 1;
        return Some(typ.to_expr_type());
    } else if let Some(Tkn {token: TknType::Identifier(typ), ..}) = tokens.get(*index) {
        for i in structs {
            if *typ == i.name {
                *index += 1;
                return Some(ExprType::Custom {ident: typ.clone()});
            }
        }
    }
    return None;
}

pub fn get_type_token_expr_type(
    tokens: &[Tkn], 
    index: &mut usize, 
    structs: &[&str]
) -> Option<ExprType> {
    let mut peek = *index;
    if let Some(TknType::Type(typ)) = tokens.get(peek).map(|e| &e.token) {
        peek += 1;
        *index = peek;
        return Some(typ.to_expr_type());
    } else if let Some(TknType::Identifier(typ)) = tokens.get(peek).map(|e| &e.token) {
        for i in structs {
            if typ == *i {
                peek += 1;
                *index = peek;
                return Some(ExprType::Custom {ident: typ.clone()});
            }
        }
    } else if let Some(TknType::OpenCurlyBrace) = tokens.get(peek).map(|e| &e.token) {
        peek += 1;
        let mut fields = vec![];
        let mut need_comma = false;
        loop {
            if tokens::is_expected_token(tokens, TknType::CloseCurlyBrace, &mut peek) {
                return Some(ExprType::AnonymousCustom { fields: fields.into_boxed_slice() });
            }

            if need_comma {
                eprintln!("comma");
                return None;
            }

            let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) else {
                return None;
            };
            peek += 1;

            tokens::expect_token(tokens, TknType::Colon, &mut peek)?;

            let field_type = get_type_token_expr_type(tokens, &mut peek, structs)?;

            fields.push((ident.clone(), field_type));

            if !tokens::is_expected_token(tokens, TknType::Comma, &mut peek) {
                need_comma = true;
            }
        }
    }
    return None;
}

pub fn get_ident_token_string(
    token: Option<&Tkn>, 
    index: &mut usize, 
    structs: &[&str]
) -> Option<String> {
    if let Some(Tkn {token: TknType::Identifier(ident), ..}) = token {
        for i in structs {
            if ident == i {
                return None;
            }
        }
        *index += 1;
        return Some(ident.clone());
    }
    return None;
}