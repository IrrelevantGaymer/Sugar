use crate::{full_result::{FullResult, OptionToFullResult}, lexer::token::{Kwrd, Tkn, TknType}};

use super::{accessors, expr::ExprType, tokens, ParserError};

#[derive(Clone, Debug)]
pub struct Struct {
    pub accessibility: String,
    pub location: String,
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub accessibility: String,
    pub field_type: ExprType,
    pub field_name: String,
}

#[derive(Debug)]
pub struct StructDefinition<'tkns> {
    pub accessibility: Option<&'tkns Tkn>,
    pub name: String,
    pub body_tokens: &'tkns [Tkn],
}

pub fn define_struct<'tkns, 'bumps, 'defs>(
    tokens: &'tkns [Tkn], 
    index: &mut usize
) -> FullResult<StructDefinition<'tkns>, (), Vec<ParserError<'tkns, 'bumps, 'defs>>> {
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
            return FullResult::SoftErr(());
        }
    };

    tokens::expect_token(tokens, TknType::Keyword(Kwrd::Struct), &mut peek).ok_or_soft::<_, Vec<_>>(())?;
    let name;
    if let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) {
        name = ident.clone();
        peek += 1;
    } else {
        return FullResult::HardErr(vec![ParserError::ExpectedIdentifier { tkn: &tokens[peek] }]);
    }

    let mut open_braces = vec![];
    tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)
        .ok_or_else_hard(|| vec![ParserError::ExpectedToken {
            tkn: &tokens[peek],
            expected: TknType::OpenCurlyBrace  
        }])?;
    open_braces.push(&tokens[peek - 1]);

    let body_tokens: &[Tkn];
    let start = peek;
    let end;
    let mut count = 0;
    loop {
        if tokens::is_expected_token(tokens, TknType::OpenCurlyBrace, &mut peek) {
            count += 1;
            open_braces.push(&tokens[peek - 1]);
            continue;
        } else if tokens::is_expected_token(tokens, TknType::CloseCurlyBrace, &mut peek) {
            if count == 0 {
                end = peek - 1;
                body_tokens = &tokens[start..end];
                *index = peek;
                return FullResult::Ok(StructDefinition {accessibility, name, body_tokens});
            }
            count -= 1;
            open_braces.pop();
            continue;
        } else if tokens::is_expected_token(tokens, TknType::EndOfFile, &mut peek) {
            let mut errors = vec![];
            for open_brace in open_braces.into_iter().rev() {
                errors.push(ParserError::ExpectedClosingBrace { 
                    tkn: &tokens[peek - 1], 
                    open_brace 
                });
            }
            return FullResult::HardErr(errors);
        }
        peek += 1;
    }
}

pub fn parse_struct<'tkns, 'bumps, 'defs>(
    struct_def: &StructDefinition<'tkns>, 
    accessors: &[&str], 
    structs: &[&str]
) -> Result<Struct, ParserError<'tkns, 'bumps, 'defs>> {
    let StructDefinition {accessibility, name, body_tokens} = struct_def;
    let mut peek: usize = 0;
    let mut fields: Vec<Field> = vec![];

    let mut need_comma = false;
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
        ).ok_or_else(|| ParserError::MissingAccessor { 
            tkn: &body_tokens[peek] 
        })?;

        if need_comma {
            return Err(ParserError::ExpectedEndOfStruct { 
                tkn: &body_tokens[peek - 1] 
            });
        }

        field_name = super::get_ident_token_string(
            tokens::get_token(body_tokens, peek), 
            &mut peek, structs
        ).ok_or_else(|| ParserError::ExpectedIdentifier { 
            tkn: &body_tokens[peek] 
        })?;

        tokens::expect_token(body_tokens, TknType::Colon, &mut peek)
            .ok_or_else(|| ParserError::ExpectedToken { 
                tkn: &body_tokens[peek], 
                expected: TknType::Colon 
            })?;

        field_type = super::get_type_token_expr_type(
            body_tokens, 
            &mut peek, 
            structs
        ).ok_or_else(|| ParserError::ExpectedType { 
            tkn: &body_tokens[peek] 
        })?;

        if tokens::expect_token(
            body_tokens, 
            TknType::Comma, 
            &mut peek
        ).is_none() {
            need_comma = true;
        };

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