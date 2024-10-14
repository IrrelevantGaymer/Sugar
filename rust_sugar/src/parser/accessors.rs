use crate::lexer::token::{Kwrd, Tkn, TknType};

use super::{tokens, ParserError};

#[derive(Debug)]
pub struct AccessorDefinition<'da> {
    pub name: String,
    pub body_tokens: &'da [Tkn<'da>],
}

pub struct Accessor {
    pub name: String,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>
}

pub fn define_accessor<'da>(
    tokens: &'da [Tkn<'da>], 
    index: &mut usize
) -> Option<AccessorDefinition<'da>> {
    let mut peek = *index;
    let name: String;

    tokens::expect_token(tokens, TknType::Keyword(Kwrd::Accessor), &mut peek)?;
    if let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) {
        name = ident.clone();
        peek += 1;
    } else {
        return None;
    }

    tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)?;

    let body_tokens;
    let start = peek;
    let end;
    loop {
        if tokens::is_expected_token(tokens, TknType::CloseCurlyBrace, &mut peek) {
            end = peek - 1;
            peek += 1;
            body_tokens = &tokens[start..=end];
            *index += peek;
            return Some( AccessorDefinition {name, body_tokens});
        }
        peek += 1;
    }
}

pub fn parse_accessor<'pa>(
    accessor_definition: &AccessorDefinition<'pa>
) -> Result<Accessor, Vec<ParserError<'pa>>> {
    let mut peek = 0;
    let AccessorDefinition { name, body_tokens } = accessor_definition;
    let mut errors = vec![];

    let mut whitelist = vec![];
    let mut blacklist = vec![];

    let mut whitelist_defined = false;
    let mut blacklist_defined = false;

    'accessor_body: while peek < body_tokens.len() {
        let start_index = peek;
        if tokens::is_expected_token(body_tokens, TknType::Keyword(Kwrd::Whitelist), &mut peek) {
            if whitelist_defined {
                errors.push(ParserError::new(
                    &body_tokens[start_index],
                    "whitelist was already defined for this accessor"
                ));
            }
            let expect_closing_brace;
            if tokens::is_expected_token(body_tokens, TknType::OpenCurlyBrace, &mut peek) {
                expect_closing_brace = true;
            } else if tokens::is_expected_token(body_tokens, TknType::Dollar, &mut peek) {
                expect_closing_brace = false;
            } else {
                errors.push(ParserError::new(
                    &body_tokens[peek],
                    "expected either '$' or '{'"
                ));
                expect_closing_brace = false;
            }

            'whitelist_body: loop {
                if peek > body_tokens.len() {
                    if expect_closing_brace {
                        errors.push(ParserError::new(
                            &body_tokens[peek - 1],
                            "expected '}' after"
                        ));
                    }
                    break 'accessor_body;
                } else if expect_closing_brace && tokens::is_expected_token(
                    body_tokens, 
                    TknType::CloseCurlyBrace, 
                    &mut peek
                ) {
                    tokens::expect_token(&body_tokens, TknType::Comma, &mut peek);
                    break 'whitelist_body;
                } else if tokens::is_token(body_tokens, TknType::Keyword(Kwrd::Blacklist), peek) {
                    break 'whitelist_body;
                } else if let Some(Tkn { 
                    token: TknType::Identifier(ident), .. 
                }) = tokens::get_token(body_tokens, peek) {
                    whitelist.push(ident.clone());
                } else {
                    errors.push(ParserError::new(
                        &body_tokens[peek],
                        "expected either an ident for whitelist, or a '}'"
                    ));
                    peek += 1;
                }
            }
            whitelist_defined = true;
            continue 'accessor_body;
        } else if tokens::is_expected_token(body_tokens, TknType::Keyword(Kwrd::Blacklist), &mut peek) {
            if blacklist_defined {
                errors.push(ParserError::new(
                    &body_tokens[start_index],
                    "blacklist was already defined for this accessor"
                ));
            }
            let expect_closing_brace;
            if tokens::is_expected_token(body_tokens, TknType::OpenCurlyBrace, &mut peek) {
                expect_closing_brace = true;
            } else if tokens::is_expected_token(body_tokens, TknType::Dollar, &mut peek) {
                expect_closing_brace = false;
            } else {
                errors.push(ParserError::new(
                    &body_tokens[peek],
                    "expected either '$' or '{'"
                ));
                expect_closing_brace = false;
            }

            'blacklist_body: loop {
                if peek > body_tokens.len() {
                    if expect_closing_brace {
                        errors.push(ParserError::new(
                            &body_tokens[peek - 1],
                            "expected '}' after"
                        ));
                    }
                    break 'accessor_body;
                } else if expect_closing_brace && tokens::is_expected_token(
                    body_tokens, 
                    TknType::CloseCurlyBrace, 
                    &mut peek
                ) {
                    tokens::expect_token(&body_tokens, TknType::Comma, &mut peek);
                    break 'blacklist_body;
                } else if tokens::is_token(body_tokens, TknType::Keyword(Kwrd::Blacklist), peek) {
                    break 'blacklist_body;
                } else if let Some(Tkn { 
                    token: TknType::Identifier(ident), .. 
                }) = tokens::get_token(body_tokens, peek) {
                    blacklist.push(ident.clone());
                } else {
                    errors.push(ParserError::new(
                        &body_tokens[peek],
                        "expected either an ident for whitelist, or a '}'"
                    ));
                    peek += 1;
                }
            }
            blacklist_defined = true;
            continue 'accessor_body;
        } else {
            errors.push(ParserError::new(
                &body_tokens[peek],
                "expected either keyword \"whitelist\" or keyword \"blacklist\""
            ));
            break 'accessor_body;
        }
    }

    if errors.is_empty() {
        return Ok(Accessor {
            name: name.clone(),
            whitelist,
            blacklist
        });
    }
    return Err(errors);
}

pub fn get_accessor_string(
    token: Option<&Tkn>,
    index: &mut usize,
    accessors: &[&str]
) -> Option<String> {
    match &token {
        Some(Tkn {token: TknType::Identifier(ident), ..}) => {
            for i in accessors {
                if i == ident {
                    *index += 1;
                    return Some(ident.clone());
                }
            }
            return None;
        },
        Some(Tkn {token: TknType::Keyword(Kwrd::Public), ..}) => {
            *index += 1;
            return Some("public".to_string());
        },
        Some(Tkn {token: TknType::Keyword(Kwrd::Private), ..}) => {
            *index += 1;
            return Some("private".to_string());
        },
        Some(Tkn {token: TknType::Keyword(Kwrd::Package), ..}) => {
            *index += 1;
            return Some("package".to_string());
        },
        _ => {
            dbg!(&token);
            return None;
        }
    }
}