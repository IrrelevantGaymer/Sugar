use crate::lexer::token::{Kwrd, Tkn, TknType};

use super::{accessors, expr::ExprType, tokens, ParserError};

use let_unless::let_unless;

#[derive(Clone, Debug)]
pub struct Struct {
    pub accessibility: String,
    pub location: String,
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub accessibility: String,
    pub field_type: ExprType,
    pub field_name: String,
}

#[derive(Debug)]
pub struct StructDefinition<'ds> {
    pub accessibility: Option<&'ds Tkn<'ds>>,
    pub name: String,
    pub body_tokens: &'ds [Tkn<'ds>],
}

pub fn define_struct<'ds>(
    tokens: &'ds [Tkn<'ds>], 
    index: &mut usize
) -> Option<StructDefinition<'ds>> {
    let mut peek = *index;
    let accessibility = match &tokens[peek].token {
        TknType::Keyword(Kwrd::Public)
        | TknType::Keyword(Kwrd::Private)
        | TknType::Keyword(Kwrd::Package) 
        | TknType::Identifier(_) => {
            peek += 1;
            Some(&tokens[peek - 1])
        },
        TknType::Keyword(Kwrd::Struct) => None,
        _ => {
            return None;
        }
    };

    tokens::expect_token(tokens, TknType::Keyword(Kwrd::Struct), &mut peek)?;
    let name;
    if let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) {
        name = ident.clone();
        peek += 1;
    } else {
        return None;
    }

    tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)?;
    let body_tokens: &[Tkn];
    let start = peek;
    let end;
    loop {
        if tokens::is_expected_token(tokens, TknType::CloseCurlyBrace, &mut peek) {
            end = peek - 1;
            body_tokens = &tokens[start..end];
            *index = peek;
            return Some(StructDefinition {accessibility, name, body_tokens});
        } else if tokens::is_expected_token(tokens, TknType::EndOfFile, &mut peek) {
            //dbg!("fuck");
            return None;
        }
        peek += 1;
    }
}

pub fn parse_struct<'t, 'ps>(
    struct_def: &'ps StructDefinition<'t>, 
    accessors: &'ps [&'ps str], 
    structs: &'ps [&'ps str]
) -> Result<Struct, ParserError<'t>> where 't: 'ps {
    let StructDefinition {accessibility, name, body_tokens} = struct_def;
    let mut peek: usize = 0;
    let mut fields: Vec<Field> = vec![];
    loop {
        if peek >= body_tokens.len() {
            break;
        }

        let field_type;
        let field_name;
        let accessible = accessors::get_accessor_string(
            tokens::get_token(body_tokens, peek),  
            &mut peek, 
            accessors
        ).ok_or(ParserError::new(
            &body_tokens[peek],
            "expected an accessor like pub or prv, or a user defined one.", 
        ))?;

        let return_type = super::get_type_token_expr_type(
            tokens::get_token(body_tokens, peek), 
            &mut peek, 
            structs
        );

        field_type = let_unless!(return_type unless ExprType::Void => {
            return Err(ParserError::new(
                &body_tokens[peek],
                "expected a type like i32 or bool, or a user defined one",
            ));
        });
        // .ok_or(ParserError::new(
        //     &body_tokens[peek],
        //     "expected a type like i32 or bool, or a user defined one", 
        // ))?;

        field_name = super::get_ident_token_string(
            tokens::get_token(body_tokens, peek), 
            &mut peek, structs
        ).ok_or(ParserError::new(
            &body_tokens[peek],
            "expected an identifier",
        ))?;

        tokens::expect_token(
            body_tokens, 
            TknType::Comma, 
            &mut peek
        ).ok_or(ParserError::new(
            &body_tokens[peek - 1],
            "expected comma, newline or end of struct definition", 
        ))?;

        fields.push(Field {accessibility: accessible, field_type, field_name});
    }
    return Ok(Struct {
        accessibility: accessors::get_accessor_string(
            *accessibility, 
            &mut 0, 
            accessors
        ).unwrap(), 
        name: name.to_string(),
        location: "".to_string(), fields
    });
}