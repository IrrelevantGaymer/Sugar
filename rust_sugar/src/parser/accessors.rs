use crate::{full_result::{FullResult, OptionToFullResult}, lexer::token::{Kwrd, Tkn, TknType}};

use super::{tokens, ParserError};

#[derive(Debug)]
pub struct AccessorDefinition<'tkns> {
    pub name: String,
    pub body_tokens: &'tkns [Tkn],
}

#[derive(Debug)]
pub struct Accessor {
    pub name: String,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>
}

pub fn define_accessor<'tkns, 'exprs, 'defs>(
    tokens: &'tkns [Tkn], 
    index: &mut usize
) -> FullResult<AccessorDefinition<'tkns>, (), Vec<ParserError<'tkns, 'exprs, 'defs>>> {
    let mut peek = *index;
    let name: String;

    tokens::expect_token(tokens, TknType::Keyword(Kwrd::Accessor), &mut peek).ok_or_soft::<_, Vec<_>>(())?;
    if let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) {
        name = ident.clone();
        peek += 1;
    } else {
        return FullResult::HardErr(vec![
            ParserError::ExpectedIdentifier { tkn: &tokens[peek] }
        ]);
    }

    let mut open_braces = vec![];
    tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)
        .ok_or_else_hard(|| vec![ParserError::ExpectedToken {
            tkn: &tokens[peek],
            expected: TknType::OpenCurlyBrace  
        }])?;
    open_braces.push(&tokens[peek - 1]);

    let body_tokens;
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
                return FullResult::Ok(AccessorDefinition { 
                    name, 
                    body_tokens
                });
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

pub fn parse_accessor<'tkns, 'exprs, 'defs>(
    accessor_definition: &AccessorDefinition<'tkns>
) -> Result<Accessor, Vec<ParserError<'tkns, 'exprs, 'defs>>> {
    let mut peek = 0;
    let AccessorDefinition { name, body_tokens } = accessor_definition;
    let mut errors = vec![];

    let mut whitelist = vec![];
    let mut blacklist = vec![];

    let mut whitelist_defined = None;
    let mut blacklist_defined = None;

    let mut open_brace = None;

    'accessor_body: while peek < body_tokens.len() {
        if tokens::is_expected_token(body_tokens, TknType::Keyword(Kwrd::Enclave), &mut peek) {
            if let Some(defined) = whitelist_defined {
                errors.push(ParserError::AlreadyDefinedWhitelist { 
                    tkn: &body_tokens[peek - 1], 
                    defined_whitelist: defined 
                });
            }
            whitelist_defined = Some(&body_tokens[peek - 1]);

            let expect_closing_brace;
            if tokens::is_expected_token(body_tokens, TknType::OpenCurlyBrace, &mut peek) {
                expect_closing_brace = true;
                open_brace = Some(&body_tokens[peek - 1]);
            } else if tokens::is_expected_token(body_tokens, TknType::Dollar, &mut peek) {
                expect_closing_brace = false;
            } else {
                errors.push(ParserError::ExpectedTokens { 
                    tkn: &body_tokens[peek], 
                    received: body_tokens[peek..].iter().map(|tkn| &tkn.token),
                    expected: &[TknType::Dollar, TknType::OpenCurlyBrace] 
                });
                expect_closing_brace = false;
            }

            'whitelist_body: loop {
                if peek > body_tokens.len() {
                    if expect_closing_brace {
                        errors.push(ParserError::ExpectedClosingBrace { 
                            tkn: &body_tokens[peek - 2], 
                            open_brace: unsafe { open_brace.unwrap_unchecked() } 
                        });
                    }
                    break 'accessor_body;
                } else if expect_closing_brace && tokens::is_expected_token(
                    body_tokens, 
                    TknType::CloseCurlyBrace, 
                    &mut peek
                ) {
                    tokens::expect_token(&body_tokens, TknType::Comma, &mut peek);
                    break 'whitelist_body;
                } else if tokens::is_token(body_tokens, TknType::Keyword(Kwrd::Exclave), peek) {
                    break 'whitelist_body;
                } else if let Some(Tkn { 
                    token: TknType::Identifier(ident), .. 
                }) = tokens::get_token(body_tokens, peek) {
                    whitelist.push(ident.clone());
                } else {
                    errors.push(ParserError::ExpectedEndOfWhitelist { tkn: &body_tokens[peek] });
                    peek += 1;
                }
            }
            continue 'accessor_body;
        } else if tokens::is_expected_token(body_tokens, TknType::Keyword(Kwrd::Exclave), &mut peek) {
            if let Some(defined) = blacklist_defined {
                errors.push(ParserError::AlreadyDefinedBlacklist { 
                    tkn: &body_tokens[peek - 1], 
                    defined_blacklist: defined
                });
            }
            blacklist_defined = Some(&body_tokens[peek - 1]); 

            let expect_closing_brace;
            if tokens::is_expected_token(body_tokens, TknType::OpenCurlyBrace, &mut peek) {
                expect_closing_brace = true;
                open_brace = Some(&body_tokens[peek - 1]);
            } else if tokens::is_expected_token(body_tokens, TknType::Dollar, &mut peek) {
                expect_closing_brace = false;
            } else {
                errors.push(ParserError::ExpectedTokens { 
                    tkn: &body_tokens[peek], 
                    received: body_tokens[peek..].iter().map(|tkn| &tkn.token),
                    expected: &[TknType::Dollar, TknType::OpenCurlyBrace] 
                });
                expect_closing_brace = false;
            }

            'blacklist_body: loop {
                if peek > body_tokens.len() {
                    if expect_closing_brace {
                        errors.push(ParserError::ExpectedClosingBrace { 
                            tkn: &body_tokens[peek - 2], 
                            open_brace: unsafe { open_brace.unwrap_unchecked() } 
                        });
                    }
                    break 'accessor_body;
                } else if expect_closing_brace && tokens::is_expected_token(
                    body_tokens, 
                    TknType::CloseCurlyBrace, 
                    &mut peek
                ) {
                    tokens::expect_token(&body_tokens, TknType::Comma, &mut peek);
                    break 'blacklist_body;
                } else if tokens::is_token(body_tokens, TknType::Keyword(Kwrd::Exclave), peek) {
                    break 'blacklist_body;
                } else if let Some(Tkn { 
                    token: TknType::Identifier(ident), .. 
                }) = tokens::get_token(body_tokens, peek) {
                    blacklist.push(ident.clone());
                } else {
                    errors.push(ParserError::ExpectedEndOfBlacklist { 
                        tkn: &body_tokens[peek] 
                    });
                    peek += 1;
                }
            }
            continue 'accessor_body;
        } else {
            errors.push(ParserError::NoWhitelistOrBlacklist { 
                tkn: &body_tokens[peek] 
            });
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