use crate::lexer::token::{Tkn, TknType};

pub fn expect_token(
    tokens: &[Tkn], 
    expected: TknType, 
    index: &mut usize
) -> Option<()> {
    let token = tokens.get(*index).map(|e| &e.token)?;
    if let TknType::Either(left, right) = expected {
        if *token == *left || *token == *right {
            *index += 1;
            return Some(());
        }
        return None;
    }

    if *token == expected {
        *index += 1;
        return Some(());
    }
    return None;
}

pub fn expect_tokens(
    tokens: &[Tkn], 
    expected: &[TknType], 
    index: &mut usize
) -> Option<()> {
    let mut peek = *index;
    for i in expected {
        expect_token(tokens, i.clone(), &mut peek)?
    }
    *index = peek;
    return Some(());
}

pub fn is_expected_token(
    tokens: &[Tkn], 
    expected: TknType, 
    index: &mut usize
) -> bool {
    let token = match tokens.get(*index).map(|e| &e.token) {
        Some(tkn) => tkn,
        None => return false,
    };
    if let TknType::Either(left, right) = expected {
        if *token == *left || *token == *right {
            *index += 1;
            return true;
        }
        return false;
    }

    if *token == expected {
        *index += 1;
        return true;
    }
    return false;
}

pub fn is_expected_tokens(
    tokens: &[Tkn], 
    expected: &[TknType], 
    index: &mut usize
) -> bool {
    let mut peek = *index;
    for i in expected {
        if !is_expected_token(tokens, i.clone(), &mut peek) {
            return false;
        }
    }
    *index = peek;
    return true;
}

pub fn is_token(
    tokens: &[Tkn],
    expected: TknType,
    index: usize
) -> bool {
    let token = match tokens.get(index).map(|e| &e.token) {
        Some(tkn) => tkn,
        None => return false,
    };
    if let TknType::Either(left, right) = expected {
        if *token == *left || *token == *right {
            return true;
        }
        return false;
    }

    return *token == expected;
}

pub fn is_tokens(
    tokens: &[Tkn],
    expected: &[TknType],
    index: usize,
) -> bool {
    let mut peek = index;
    for i in expected {
        if !is_expected_token(tokens, i.clone(), &mut peek) {
            return false;
        }
    }
    return true;
}

pub fn get_token<'tkns>(tokens: &'tkns [Tkn], index: usize) -> Option<&'tkns Tkn> {
    if index >= tokens.len() {
        return None;
    }
    return Some(&tokens[index]);
}