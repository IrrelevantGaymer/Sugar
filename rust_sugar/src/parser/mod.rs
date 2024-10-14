use std::{cell::RefCell, collections::HashMap, ops::Deref};

use accessors::{Accessor, AccessorDefinition};
use bumpalo::Bump;
use expr::{ExprType, VariableData};
use functions::{FullFunctionDefinition, Fun, Function, FunctionDefinition};
use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;
use structs::{Struct, StructDefinition};

use crate::lexer::token::{Tkn, TknType};

pub mod accessors;
pub mod expr;
pub mod functions;
pub mod operators;
pub mod stmt;
pub mod structs;
pub mod tokens;

pub type ExprBump = ExpressionBumpAllocator;
pub type StmtBump = StatementBumpAllocator;
pub type FnParamBump = FunctionParameterBumpAllocator;

#[derive(Debug)]
pub struct ParserErrorAccumulator<'pea> {
    errors: Vec<ParserError<'pea>>
}

impl<'pea> ParserErrorAccumulator<'pea> {
    pub fn new() -> Self {
        ParserErrorAccumulator { errors: vec![] }
    }

    pub fn push(&mut self, error: ParserError<'pea>) {
        self.errors.push(error);
    }
}

#[derive(Debug)]
pub struct ParserError<'pe> {
    pub token: &'pe Tkn<'pe>,
    pub msg: &'static str
}

impl<'pe> ParserError<'pe> {
    pub fn new(token: &'pe Tkn, msg: &'static str) -> Self {
        ParserError {token, msg}
    }
}

impl<'pe> Into<ParserErrorAccumulator<'pe>> for ParserError<'pe> {
    fn into(self) -> ParserErrorAccumulator<'pe> {
        ParserErrorAccumulator {errors: vec![self]}
    }
}

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

pub fn parse<'t>(
    expr_bump: &'t ExprBump,
    stmt_bump: &'t StmtBump,
    fn_param_bump: &'t FnParamBump,
    tokens: &'t [Tkn<'t>]
) -> Result<(
    Vec<Accessor>, 
    Vec<Struct>, 
    Vec<Function<'t>>
), Vec<ParserError<'t>>> {
    let mut errors: Vec<ParserError> = vec![];

    let functions = RefCell::new(HashMap::new());
    let variables = StackFrameDictAllocator::<String, VariableData>::new();
    
    let mut accessor_defs: Vec<AccessorDefinition> = vec![];
    let mut struct_defs: Vec<StructDefinition> = vec![];
    let mut function_defs: Vec<FunctionDefinition> = vec![];

    let mut index: usize = 0;
    while index < tokens.len() {
        if let Some(def) = accessors::define_accessor(tokens, &mut index) {
            accessor_defs.push(def);
        } else if let Some(def) = structs::define_struct(tokens, &mut index) {
            struct_defs.push(def);
        } else if let Some(def) = functions::define_function(tokens, &mut index) {
            function_defs.push(def);
        } else if tokens::is_expected_token(tokens, TknType::EndOfFile, &mut index) {
            break;
        } else {
            errors.push(ParserError::new(&tokens[index], "Could not parse token {}"));
            return Err(errors);
        }
    }
    /*
    let mut data_file = OpenOptions::new()
        .append(true)
        .open("test/parse.txt")
        .expect("cannot open file");

    data_file
        .write(format!("defined accessors:\n{accessor_def:?}\n\n").as_bytes())
        .expect("write failed!");
    data_file
        .write(format!("defined structs:\n{struct_def:?}\n\n").as_bytes())
        .expect("write failed!");
    data_file
        .write(format!("defined functions:\n{function_def:?}\n\n").as_bytes())
        .expect("write failed!");
    */

    // dbg!(&accessor_defs);
    // dbg!(&struct_defs);
    // dbg!(&function_defs);

    let mut full_function_defs: Vec<FullFunctionDefinition> = vec![];
    let mut def_functions: Vec<Fun> = vec![];

    let accessors = accessor_defs.iter()
    .map(|AccessorDefinition {name, ..}|
        name.as_str()
    ).collect::<Vec<&str>>();

    let struct_names = struct_defs
        .iter()
        .map(|StructDefinition {name, ..} | {
            name.as_str()
        })
        .collect::<Vec<&str>>();

    let mut structs: Vec<Struct> = vec![];
    let mut full_accessors: Vec<Accessor> = vec![];

    //index = 0;
    for function_def in function_defs {
        let (
            full_function_def, 
            body_tokens
        ) = match FullFunctionDefinition::from_partial_fn_def(
            &fn_param_bump, 
            function_def, 
            &accessors, 
            &struct_names
        ) {
            Ok((full_function_def, body_tokens)) => (full_function_def, body_tokens),
            Err(err) => {
                errors.push(err);
                continue;
            }
        };

        functions.borrow_mut().insert(full_function_def.name.clone(), full_function_def.clone());
                
        if body_tokens.is_empty() {
            full_function_defs.push(full_function_def);
        } else {
            //println!("parsing function {}", &full_function_def.name);
            let function = match functions::parse_function(
                &expr_bump, 
                &stmt_bump, 
                &functions, 
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
            // let function = match functions::parse_function(
            //     &expr_bump, 
            //     &stmt_bump, 
            //     &functions, 
            //     variables.new_frame(), 
            //     full_function_def, 
            //     body_tokens
            // ) {
            //     Ok(function) => {
            //         println!("got function");
            //         function
            //     },
            //     Err(ParserError{token, msg}) => panic!("{msg} at {token}")
            // };
            //println!("parsed function");
            def_functions.push(function);
            //println!("pushed function");
        }
    }

    for i in &struct_defs {
        match structs::parse_struct(
            i, &accessors, &struct_names
        ) {
            Ok(structure) => structs.push(structure),
            Err(err) => errors.push(err),
        };
    }

    for i in &accessor_defs {
        match accessors::parse_accessor(i) {
            Ok(accessor) => full_accessors.push(accessor),
            Err(ref mut err) => errors.append(err)
        }
    }

    //dbg!(def_functions);
    //dbg!(structs);

    if errors.is_empty() {
        return Ok((full_accessors, structs, def_functions));
    }
    return Err(errors);
    /*
    data_file
        .write(format!("function definitions:\n{function_defs:?}\n\n").as_bytes())
        .expect("write failed!");
    data_file
        .write(format!("function declarations:\n{functions:?}\n\n").as_bytes())
        .expect("write failed!");
    */


}

pub fn get_type_token_expr_type(
    token: Option<&Tkn>, 
    index: &mut usize, 
    structs: &[&str]
) -> ExprType {
    if let Some(Tkn {token: TknType::Type(typ), ..}) = token {
        *index += 1;
        return typ.to_expr_type();
    } else if let Some(Tkn {token: TknType::Identifier(typ), ..}) = token {
        for i in structs {
            if typ == i {
                *index += 1;
                return ExprType::Custom {ident: typ.clone()};
            }
        }
    }
    return ExprType::Void;
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