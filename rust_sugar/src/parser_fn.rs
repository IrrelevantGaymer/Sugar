use crate::{
    token::{TknType, Tkn, Kwrd}, 
    syntax::{FnParam},
    parser::{
        is_expected_token,
        is_expected_tokens,
        expect_token,
        expect_tokens,
        get_token_from_tokens_at
    }
};

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

pub fn get_type_token_string(
    token: Option<&TknType>, 
    index: &mut usize, 
    structs: &[&str]
) -> Option<String> {
    if let Some(TknType::Type(typ)) = token {
        *index += 1;
        return Some(typ.to_string());
    } else if let Some(TknType::Identifier(typ)) = token {
        for i in structs {
            if typ == i {
                *index += 1;
                return Some(typ.clone());
            }
        }
    }
    return None;
}

pub fn get_ident_token_string(
    token: Option<&TknType>, 
    index: &mut usize, 
    structs: &[&str]
) -> Option<String> {
    if let Some(TknType::Identifier(ident)) = token {
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

pub fn get_accessor_string(
    token: Option<&TknType>,
    index: &mut usize,
    accessors: &[&str]
) -> Option<String> {
    match &token {
        Some(TknType::Identifier(ident)) => {
            for i in accessors {
                if i == ident {
                    *index += 1;
                    return Some(ident.clone());
                }
            }
            return None;
        },
        Some(TknType::Keyword(Kwrd::Public)) => {
            *index += 1;
            return Some("public".to_string());
        },
        Some(TknType::Keyword(Kwrd::Private)) => {
            *index += 1;
            return Some("private".to_string());
        },
        Some(TknType::Keyword(Kwrd::Package)) => {
            *index += 1;
            return Some("package".to_string());
        },
        _ => {
            dbg!(&token);
            return None;
        }
    }
}

pub fn expect_closing_group(
    tokens: &[Tkn], 
    encapsulation: &Encapsulation, 
    index: &mut usize
) -> bool {
    if (encapsulation == &Encapsulation::Parenthesis 
        && tokens[*index].token == TknType::CloseParen)
    {
        *index += 1;
        return true;
    }
    if (encapsulation == &Encapsulation::Dollar 
        && (tokens[*index].token == TknType::Dollar
        || tokens[*index].token == TknType::Colon 
        || tokens[*index].token == TknType::OpenCurlyBrace)) 
    {
        return true;
    }
    return false;
}

pub fn fn_group_to_params<'fp>(
    tokens: &[Tkn],  
    index: &mut usize, 
    structs: &[&str]
) -> Vec<FnParam<'fp>> {
    let mut fn_param: FnParam = FnParam::default();
    let mut fn_params: Vec<FnParam> = vec![];
    let mut just_types: Option<ArgDef> = None;
    let require_commas: bool;

    let encapsulation: Encapsulation;
    if is_expected_token(tokens, TknType::OpenParen, index) {
        encapsulation = Encapsulation::Parenthesis;
    } else if is_expected_token(tokens, TknType::Dollar, index) {
        encapsulation = Encapsulation::Dollar;
    } else {
        *index += 1;
        return fn_params;
    }

    if let Some(typ) = get_type_token_string(
        get_token_from_tokens_at(tokens, *index), 
        index, &structs) 
    {
        fn_param.param_type = Some(typ);
    } else if let Some(TknType::Identifier(ident)) = get_token_from_tokens_at(tokens, *index) {
        *index += 1;
        fn_param.param_name = Some(ident.clone());
        fn_params.push(fn_param.clone());
        fn_param = FnParam::default();
        just_types = Some(ArgDef::JustIdents);
    } else if encapsulation == Enc::Parenthesis 
        && is_expected_token(tokens, TknType::CloseParen, index) 
    {
        return fn_params;
    } else {
        panic!("unexpected token {:?}, expected type, identifier or )", get_token_from_tokens_at(tokens, *index));
    }

    if encapsulation == Enc::Parenthesis 
        && is_expected_token(tokens, TknType::CloseParen, index) 
    {
        if just_types == None { just_types = Some(ArgDef::JustTypes); }
        fn_params.push(fn_param.clone());
        fn_param = FnParam::default();
        return fn_params;
    }

    if is_expected_token(tokens, TknType::Comma, index) {
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
        if let Some(typ) = get_type_token_string(
            get_token_from_tokens_at(tokens, *index), 
            index, &structs) 
        {
            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();

            fn_param.param_type = Some(typ.to_string());
            just_types = Some(ArgDef::JustTypes);
        } else if let Some(TknType::Identifier(ident)) = get_token_from_tokens_at(tokens, *index) {
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
                let comma = is_expected_token(
                    tokens, 
                    TknType::Comma, 
                    index
                );
                if expect_closing_group(
                    tokens,  
                    &encapsulation, 
                    index
                ) {
                    
                    return fn_params;
                }
                if !comma {panic!("expected comma at {}", *index);}
            } else if is_expected_token(tokens, TknType::Comma, index)  { 
                panic!("invalid comma at {}", *index);
            }

            if let Some(typ) = get_type_token_string(
                get_token_from_tokens_at(tokens, *index), 
                index, &structs)
            {
                fn_param.param_type = Some(typ);
            } else if expect_closing_group(
                tokens, 
                &encapsulation, 
                index
            ) {
                return fn_params;
            } else {
                panic!("unexpected token {}, was expecting type", tokens[*index]);
            }

            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();
        }
    } else if just_types == Some(ArgDef::JustIdents) {
        loop {
            if require_commas {
                let comma = is_expected_token(tokens, TknType::Comma, index);
                if expect_closing_group(
                    tokens, 
                    &encapsulation, 
                    index
                ) {
                    return fn_params;
                }
                if !comma {panic!("expected comma at {}", *index);}
            } else if is_expected_token(tokens, TknType::Comma, index) { 
                panic!("invalid comma at {}", *index);
            }
            
            if let Some(ident) = get_ident_token_string(
                get_token_from_tokens_at(tokens, *index), 
                index, &structs) 
            {
                fn_param.param_name = Some(ident.clone());
            } else if expect_closing_group(
                tokens,  
                &encapsulation, 
                index
            ) {
                return fn_params;
            } else {
                panic!("unexpected token {:?}, expected identifier, not type", get_token_from_tokens_at(tokens, *index - 1));
            }

            fn_params.push(fn_param.clone());
            fn_param = FnParam::default();
        }
    } else if just_types == Some(ArgDef::Both) {
        loop {
            if require_commas {
                let comma = is_expected_token(tokens, TknType::Comma, index);
                if expect_closing_group(
                    tokens, 
                    &encapsulation, 
                    index
                ) {
                    return fn_params;
                }
                if !comma {panic!("expected comma at {}", *index);}
            } else if is_expected_token(tokens, TknType::Comma, index) { 
                panic!("invalid comma at {}", *index);
            }

            if let Some(typ) = get_type_token_string(
                get_token_from_tokens_at(tokens, *index), 
                index, &structs)
            {
                fn_param.param_type = Some(typ);
            } else if expect_closing_group(
                tokens, 
                &encapsulation, 
                index
            ) {
                return fn_params;
            } else {
                panic!("unexpected token {}, was expecting type", tokens[*index]);
            }

            if let Some(ident) = get_ident_token_string(
                get_token_from_tokens_at(tokens, *index), 
                index, &structs)
            {
                fn_param.param_name = Some(ident);
            } else if expect_closing_group(
                tokens, 
                &encapsulation, 
                index
            ) {
                return fn_params;
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