#[allow(dead_code)]
use std::fs::OpenOptions;
use std::io::Write;

use crate::parser_fn::get_accessor_string;
use crate::syntax::Field;
use crate::token::{Kwrd, Op, Tkn, TknType};
use crate::{
    option_result_utils::Resultable,
    parser_fn::{
        expect_closing_group, fn_group_to_params, get_ident_token_string, get_type_token_string,
        ArgDef, Enc, FnParseState, FnType,
    },
    syntax::{
        BinOp, Expr, Fn, FnParam, Lit, OpAssoc, OpPrec, Stmt, Struct, UnOp, OPERATOR_INFO_MAP,
    },
};
use if_chain::if_chain;
use num::complex::ComplexFloat;

pub struct Parser<'p> {
    tokens: Vec<Tkn<'p>>,
    location: &'p str,
}

impl<'p> Parser<'p> {
    pub fn new(location: &'p str, tokens: Vec<Tkn<'p>>) -> Parser<'p> {
        return Parser {
            tokens: tokens,
            location: location,
        };
    }

    fn get_token_at(&self, index: usize) -> Option<&TknType> {
        if index >= self.tokens.len() {
            return None;
        }
        return Some(&self.tokens[index].token);
    }

    pub fn parse(&self) {
        let mut accessor_def: Vec<(String, &[Tkn])> = vec![];
        let mut struct_def: Vec<(Option<&TknType>, String, &[Tkn])> = vec![];
        let mut function_def: Vec<(Option<&TknType>, String, bool, bool, &[Tkn], &[Tkn])> = vec![];
        let mut actions: Vec<Stmt> = vec![];

        let mut index: usize = 0;
        while index < self.tokens.len() {
            if let Some(def) = self.define_accessor(&mut index) {
                accessor_def.push(def);
            } else if let Some(def) = self.define_struct(&mut index) {
                struct_def.push(def);
            } else if let Some(def) = self.define_function(&mut index) {
                function_def.push(def);
            } else if self.is_expected_token(TknType::NewLine, &mut index) {
                index += 1;
            } else if let Some(TknType::Spaces(_)) = self.get_token_at(index) {
                index += 1;
            } else if self.is_expected_token(TknType::EndOfFile, &mut index) {
                break;
            } else {
                panic!("Could not parse token {}", &self.tokens[index]);
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

        dbg!(&accessor_def);
        dbg!(&struct_def);
        dbg!(&function_def);

        let mut function_defs: Vec<(
            String,
            String,
            bool,
            bool,
            Vec<String>,
            Vec<String>,
            Option<String>,
        )> = vec![];
        let mut functions: Vec<(
            String,
            String,
            bool,
            bool,
            Vec<FnParam>,
            Vec<FnParam>,
            Option<String>,
            &[Tkn],
        )> = vec![];

        let accessors = accessor_def.iter()
        .map(|(name, _)|
            name.as_str()
        ).collect::<Vec<&str>>();

        let struct_names = &struct_def
            .iter()
            .map(|(_, struct_name, _) | {
                struct_name.as_str()
            })
            .collect::<Vec<&str>>();

        let mut structs: Vec<Struct> = vec![];

        index = 0;
        for function in function_def {
            let (accessibility, name, mutable, recursive, fn_arg_tkns, fn_body_tkns) = function;
            let accessibility = match get_accessor_string(accessibility, &mut 0, &accessors) {
                Some(accessor) => accessor,
                None => continue
            };
            match Self::define_arguments(
                fn_arg_tkns,
                &struct_names
            ) {
                Ok((left_args, right_args, return_type)) => {
                    if fn_body_tkns.is_empty() {
                        function_defs.push((
                            accessibility,
                            name,
                            mutable,
                            recursive,
                            left_args
                                .iter()
                                .map(|param| param.param_type.as_ref().unwrap().clone())
                                .collect(),
                            right_args
                                .iter()
                                .map(|param| param.param_type.as_ref().unwrap().clone())
                                .collect(),
                            return_type,
                        ));
                    } else {
                        functions.push((
                            accessibility,
                            name,
                            mutable,
                            recursive,
                            left_args,
                            right_args,
                            return_type,
                            fn_body_tkns,
                        ));
                    }
                }
                Err(err) => println!("{}", err),
            }
        }

        for i in &struct_def {
            match Self::parse_struct(
                i, &accessors, &struct_names
            ) {
                Ok(structure) => structs.push(structure),
                Err(e) => {dbg!(e);},
            };
        }

        dbg!(functions);
        dbg!(structs);
        /*
        data_file
            .write(format!("function definitions:\n{function_defs:?}\n\n").as_bytes())
            .expect("write failed!");
        data_file
            .write(format!("function declarations:\n{functions:?}\n\n").as_bytes())
            .expect("write failed!");
        */


    }

    pub fn define_accessor(&self, index: &mut usize) -> Option<(String, &[Tkn])> {
        let mut peek = *index;
        let name: String;

        self.expect_token(TknType::Keyword(Kwrd::Accessor), &mut peek)?;
        if let Some(TknType::Identifier(ident)) = self.get_token_at(peek) {
            name = ident.clone();
            peek += 1;
        } else {
            return None;
        }

        self.expect_token(TknType::OpenCurlyBrace, &mut peek)?;

        let body_tokens;
        let start = peek;
        let end;
        loop {
            if self.is_expected_token(TknType::CloseCurlyBrace, &mut peek) {
                end = peek - 1;
                peek += 1;
                body_tokens = &self.tokens[start..=end];
                *index += peek;
                return Some((name, body_tokens));
            }
            peek += 1;
        }
    }

    pub fn define_struct(&self, index: &mut usize) -> Option<(Option<&TknType>, String, &[Tkn])> {
        let mut peek = *index;
        let accessibility = match &self.tokens[peek].token {
            TknType::Keyword(Kwrd::Public)
            | TknType::Keyword(Kwrd::Private)
            | TknType::Keyword(Kwrd::Package) 
            | TknType::Identifier(_) => {
                peek += 1;
                Some(&self.tokens[peek - 1].token)
            },
            TknType::Keyword(Kwrd::Struct) => None,
            _ => {
                return None;
            }
        };

        self.expect_token(TknType::Keyword(Kwrd::Struct), &mut peek)?;
        let name;
        if let Some(TknType::Identifier(ident)) = self.get_token_at(peek) {
            name = ident.clone();
            peek += 1;
        } else {
            return None;
        }

        self.expect_token(TknType::OpenCurlyBrace, &mut peek)?;
        let body_tokens: &[Tkn];
        let start = peek;
        let end;
        loop {
            if self.is_expected_token(TknType::CloseCurlyBrace, &mut peek) {
                end = peek - 1;
                peek += 1;
                body_tokens = &self.tokens[start..end];
                *index = peek;
                return Some((accessibility, name, body_tokens));
            } else if self.is_expected_token(TknType::EndOfFile, &mut peek) {
                dbg!("fuck");
                return None;
            }
            peek += 1;
        }
    }

    pub fn define_function(
        &self,
        index: &mut usize,
    ) -> Option<(Option<&TknType>, String, bool, bool, &[Tkn], &[Tkn])> {
        let mut peek = *index;
        let accessibility = match &self.tokens[peek].token {
            TknType::Keyword(Kwrd::Public)
            | TknType::Keyword(Kwrd::Private)
            | TknType::Keyword(Kwrd::Package) 
            | TknType::Identifier(_) => {
                peek += 1;
                Some(&self.tokens[peek - 1].token)
            },
            TknType::Keyword(Kwrd::Mutable)
            | TknType::Keyword(Kwrd::Recursive)
            | TknType::Keyword(Kwrd::Function) => None,
            _ => return None,
        };

        let mutable = self.is_expected_token(TknType::Keyword(Kwrd::Mutable), &mut peek);
        let recursive = self.is_expected_token(TknType::Keyword(Kwrd::Recursive), &mut peek);

        let name;
        self.expect_token(TknType::Keyword(Kwrd::Function), &mut peek)?;
        if let Some(TknType::Identifier(ident)) = self.get_token_at(peek) {
            peek += 1;
            name = ident.clone();
        } else {
            return None;
        }

        let argument_tokens: &[Tkn];
        let body_tokens: &[Tkn];
        let mut start = peek;
        let mut end;
        loop {
            if self.is_expected_token(TknType::Semicolon, &mut peek) {
                end = peek - 1;
                argument_tokens = &self.tokens[start..=end];
                body_tokens = &self.tokens[peek..=peek];
                *index = peek + 1;
                return Some((
                    accessibility,
                    name,
                    mutable,
                    recursive,
                    argument_tokens,
                    body_tokens,
                ));
            } else if self.is_expected_token(TknType::OpenCurlyBrace, &mut peek) {
                end = peek - 1;
                argument_tokens = &self.tokens[start..=end];
                start = peek;
                peek += 1;
                break;
            } else if self.is_expected_token(TknType::Operation(Op::Arrow), &mut peek) {
                end = peek;
                argument_tokens = &self.tokens[start..=end];
                start = peek + 1;
                peek += 1;
                loop {
                    if self.is_expected_token(TknType::Semicolon, &mut peek) {
                        end = peek - 1;
                        body_tokens = &self.tokens[start..=end];
                        *index = peek + 1;
                        return Some((
                            accessibility,
                            name,
                            mutable,
                            recursive,
                            argument_tokens,
                            body_tokens,
                        ));
                    }
                    peek += 1;
                }
            }
            peek += 1;
        }
        let mut count = 0;
        loop {
            if self.is_expected_token(TknType::OpenCurlyBrace, &mut peek) {
                count += 1;
            } else if self.is_expected_token(TknType::CloseCurlyBrace, &mut peek) {
                if count > 0 {
                    count -= 1;
                } else {
                    end = peek - 1;
                    body_tokens = &self.tokens[start..=end];
                    *index = peek + 1;
                    return Some((
                        accessibility,
                        name,
                        mutable,
                        recursive,
                        argument_tokens,
                        body_tokens,
                    ));
                }
            }
            peek += 1;
        }
    }

    fn define_arguments<'t>(
        tokens: &'t [Tkn],
        structs: &[&str],
    ) -> Result<(Vec<FnParam<'t>>, Vec<FnParam<'t>>, Option<String>), String> {
        let mut peek = 0;
        let mut fn_params_1: Vec<FnParam>;
        let mut fn_params_2: Vec<FnParam> = vec![];
        let mut return_type: Option<String> = None;
        let mut id: Option<FnType> = None;

        if is_expected_tokens(
            tokens,
            &[
                TknType::OpenParen,
                TknType::Keyword(Kwrd::Prefix),
                TknType::CloseParen,
            ],
            &mut peek,
        ) || is_expected_tokens(
            tokens,
            &[TknType::Dollar, TknType::Keyword(Kwrd::Prefix)],
            &mut peek,
        ) || is_expected_token(tokens, TknType::Keyword(Kwrd::Prefix), &mut peek)
        {
            id = Some(FnType::Prefix);
        }
        fn_params_2 = fn_group_to_params(tokens, &mut peek, &structs);

        if let Some(FnType::Prefix) = id {
            if is_expected_token(tokens, TknType::Colon, &mut peek) {
                return_type = get_type_token_string(
                    get_token_from_tokens_at(tokens, peek),
                    &mut peek,
                    structs,
                );
                fn_params_1 = vec![];
                return Ok((fn_params_1, fn_params_2, return_type));
            } else if peek >= tokens.len() {
                fn_params_1 = vec![];
                return Ok((fn_params_1, fn_params_2, return_type));
            } else {
                return Err(format!("{} expected end of argument definition", tokens[peek]));
            }
        }

        if is_expected_tokens(
            tokens,
            &[
                TknType::OpenParen,
                TknType::Keyword(Kwrd::Infix),
                TknType::CloseParen,
            ],
            &mut peek,
        ) || is_expected_tokens(
            tokens,
            &[TknType::Dollar, TknType::Keyword(Kwrd::Infix)],
            &mut peek,
        ) || is_expected_token(tokens, TknType::Keyword(Kwrd::Infix), &mut peek)
        {
            if id != None {
                return Err (format!(
                    "function cannot be defined as infix, 
                    because it is already defined as prefix {}", tokens[peek - 1]
                ));
            }
            id = Some(FnType::Infix);
            fn_params_1 = fn_params_2;
            fn_params_2 = vec![];

            fn_params_2 = fn_group_to_params(tokens, &mut peek, structs);
            if is_expected_token(tokens, TknType::Colon, &mut peek) {
                return_type = get_type_token_string(
                    get_token_from_tokens_at(tokens, peek),
                    &mut peek,
                    structs,
                );
                return Ok((fn_params_1, fn_params_2, return_type));
            } else if peek >= tokens.len() {
                return Ok((fn_params_1, fn_params_2, return_type));
            } else {
                return Err(format!(
                    "expected a colon, semicolon, or beginning of function body: {}", 
                    tokens[peek]
                ));
            }
        } else if is_expected_tokens(
            tokens,
            &[
                TknType::OpenParen,
                TknType::Keyword(Kwrd::Postfix),
                TknType::CloseParen,
            ],
            &mut peek,
        ) || is_expected_tokens(
            tokens,
            &[TknType::Dollar, TknType::Keyword(Kwrd::Postfix)],
            &mut peek,
        ) || is_expected_token(tokens, TknType::Keyword(Kwrd::Postfix), &mut peek)
        {
            if id != None {
                return Err(format!(
                    "function cannot be defined as postfix, 
                    because it is already defined as prefix {}", tokens[peek - 1]
                ));
            }
            id = Some(FnType::Postfix);
            fn_params_1 = fn_params_2;
            fn_params_2 = vec![];

            if is_expected_token(tokens, TknType::Colon, &mut peek) {
                return_type = get_type_token_string(
                    get_token_from_tokens_at(tokens, peek),
                    &mut peek,
                    structs,
                );
                return Ok((fn_params_1, fn_params_2, return_type));
            } else if peek >= tokens.len() {
                return Ok((fn_params_1, fn_params_2, return_type));
            } else {
                return Err(format!("{} expected end of argument definition", tokens[peek]));
            }
        } else if is_expected_token(tokens, TknType::Colon, &mut peek) {
            return_type =
                get_type_token_string(get_token_from_tokens_at(tokens, peek), &mut peek, structs);
            fn_params_1 = vec![];
            return Ok((fn_params_1, fn_params_2, return_type));
        } else if peek >= tokens.len() {
            fn_params_1 = vec![];
            return Ok((fn_params_1, fn_params_2, return_type));
        } else {
            return Err(format!("{} expected end of argument definition", tokens[peek]));
        }
    }

    pub fn parse_function(&mut self) -> Option<Fn> {
        todo!();
    }

    fn parse_accessor(&self) {}

    fn parse_struct(
        struct_def: &(Option<&TknType>, String, &[Tkn]), 
        accessors: &[&str], structs: &[&str]
    ) -> Result<Struct, String> {
        let (accessibility, name, tokens) = struct_def;
        let mut peek: usize = 0;
        let mut fields: Vec<Field> = vec![];
        loop {
            if peek >= tokens.len() {
                break;
            }

            while let Some(TknType::NewLine | TknType::Spaces(_)) 
                = get_token_from_tokens_at(tokens, peek) 
            {
                peek += 1;
            }

            let field_type;
            let field_name;
            let accessible = get_accessor_string(
                get_token_from_tokens_at(tokens, peek),  
                &mut peek, 
                &accessors
            ).ok_or(format!(
                "expected another field: {}", 
                tokens[peek]
            ))?;
            dbg!(&accessible, &tokens[peek - 1]);

            field_type = get_type_token_string(
                get_token_from_tokens_at(tokens, peek), 
                &mut peek, structs
            ).ok_or(format!(
                "expected type: {}", 
                tokens[peek]
            ))?;
            dbg!(&field_type, &tokens[peek - 1]);

            field_name = get_ident_token_string(
                get_token_from_tokens_at(tokens, peek), 
                &mut peek, structs
            ).ok_or(format!(
                "expected identifier: {}",
                tokens[peek]
            ))?;
            dbg!(&field_name, &tokens[peek - 1]);

            expect_token(tokens, TknType::Comma, &mut peek)
                .or_else(|| expect_token(tokens, TknType::NewLine, &mut peek))
                .or_else(|| {
                    if peek < tokens.len() {Some(())} 
                    else {None}
                }).ok_or(format!(
                    "expected comma, newline or end of struct definition: {}", 
                    tokens[peek]
                ))?;

            while let Some(TknType::NewLine | TknType::Spaces(_)) 
                = get_token_from_tokens_at(tokens, peek) 
            {
                peek += 1;
            }

            fields.push(Field {accessibility: accessible, field_type, field_name});
        }
        return Ok(Struct {
            accessibility: get_accessor_string(
                *accessibility, 
                &mut 0, 
                accessors
            ).unwrap(), 
            name: name.to_string(),
            location: "".to_string(), fields
        });
    }

    fn parse_statement(&self) -> Option<Stmt> {
        todo!();
    }

    fn parse_variable_declaration(&self, index: &mut usize) -> Result<Vec<Stmt>, (&Tkn, String)> {
        let mut peek = *index;
        let data_type: Option<String> = match self.get_token_at(peek) {
            Some(TknType::Type(data)) => Some(data.to_string()),
            Some(TknType::Keyword(Kwrd::Let)) => None,
            Some(TknType::Identifier(data)) => Some(data.to_string()),
            _ => {
                return Err((
                    &self.tokens[*index],
                    "Unexpected Token.  Expected a Type, Identifier, or Let".to_string(),
                ))
            }
        };

        peek += 1;
        let mut idents = vec![];
        let mut stmts = vec![];
        while let Some(TknType::Identifier(ident)) = self.get_token_at(peek) {
            idents.push(ident.clone());
            stmts.push(Stmt::Declare(data_type.clone(), ident.clone()));
            peek += 1;
        }

        if idents.len() == 0 {
            return Err((
                &self.tokens[*index],
                "Expected identifier(s) to be declared".to_string(),
            ));
        }

        if_chain! {
            if self.is_expected_token(TknType::Operation(Op::Assign), &mut peek);
            if let Ok(expr) = self.parse_expression(&mut peek, 0);
            then {
                for i in 0..idents.len() {
                    stmts.push(Stmt::Assign(
                        Expr::Identifier(idents[i].clone()),
                        expr.clone()
                    ));
                }
            }
        }

        self.expect_token(
            TknType::Either(&TknType::Semicolon, &TknType::NewLine),
            &mut peek,
        )
        .to_result_with_error((
            &self.tokens[peek],
            "Expected Semicolon or Newline".to_string(),
        ))?;

        return Ok(stmts);
    }

    fn parse_variable_assignment(&self, index: usize) -> Result<Vec<Stmt>, (&Tkn, String)> {
        let mut peek = index;
        let mut idents: Vec<String> = vec![];

        loop {
            if_chain! {
                if let Some(TknType::Identifier(ident)) = self.get_token_at(peek);
                if let Some(TknType::Operation(Op::Assign)) = self.get_token_at(peek + 1);
                then {
                    idents.push(ident.clone());
                    peek += 2;
                    continue;
                } else {
                    break;
                }
            }
        }

        let expr = self.parse_expression(&mut peek, 0)?;
        let mut stmts: Vec<Stmt> = vec![];
        for i in 0..idents.len() {
            stmts.push(Stmt::Assign(
                Expr::Identifier(idents[i].clone()),
                expr.clone(),
            ));
        }
        return Ok(stmts);
    }

    fn parse_variable_insertion(&self, index: usize) -> Option<Stmt> {
        let mut peek = index;

        return None;
    }

    fn parse_compound_statement(&self) -> Option<Stmt> {
        todo!();
    }

    pub fn parse_expression(
        &self,
        index: &mut usize,
        min_prec: u32,
    ) -> Result<Expr<'p>, (&Tkn, String)> {
        let mut peek = *index;
        let mut left_expr = self.parse_atom(&mut peek)?;
        loop {
            let mut operator = self.get_token_at(peek).unwrap();

            if let TknType::Either(left, right) = operator {
                if OPERATOR_INFO_MAP.contains_key(*left) {
                    operator = *left;
                } else if OPERATOR_INFO_MAP.contains_key(*right) {
                    operator = *right;
                } else {
                    break;
                }
            } else {
                if !OPERATOR_INFO_MAP.contains_key(operator) {
                    break;
                }
            }

            let (prec, assoc) = OPERATOR_INFO_MAP[operator];
            let prec = prec as u32;

            if prec < min_prec {
                break;
            }

            let next_min_prec = if assoc == OpAssoc::Left {
                prec + 1
            } else {
                prec
            };

            peek += 1;

            let right_expr = self.parse_expression(&mut peek, next_min_prec)?;

            left_expr = Expr::BinaryOp(
                BinOp::get_bin_op(operator),
                Box::new(left_expr),
                Box::new(right_expr),
            );
        }

        *index = peek;
        return Ok(left_expr);
    }

    fn parse_atom(&self, index: &mut usize) -> Result<Expr<'p>, (&Tkn, String)> {
        let mut peek = *index;
        let mut curr_token = self.get_token_at(peek);

        let op: Option<UnOp>;
        if let Some(TknType::Operation(Op::Plus)) = curr_token {
            op = Some(UnOp::Plus);
            peek += 1;
        } else if let Some(TknType::Operation(Op::Minus)) = curr_token {
            op = Some(UnOp::Minus);
            peek += 1;
        } else if let Some(TknType::Operation(Op::LogicNot)) = curr_token {
            op = Some(UnOp::LogicNot);
            peek += 1;
        } else if let Some(TknType::Operation(Op::BitwiseNegate)) = curr_token {
            op = Some(UnOp::BitwiseNegate);
            peek += 1;
        } else if let Some(TknType::Borrow) = curr_token {
            if let Some(TknType::Keyword(Kwrd::Mutable)) = self.get_token_at(peek + 1) {
                op = Some(UnOp::BorrowMutable);
                peek += 2;
            } else {
                op = Some(UnOp::Borrow);
                peek += 1;
            }
        } else {
            op = None;
        }

        curr_token = self.get_token_at(peek);

        let expr;
        if let Some(TknType::Identifier(ident)) = curr_token {
            expr = Expr::Identifier(ident.clone());
            peek += 1;
        } else if let Some(TknType::IntegerLiteral(int)) = curr_token {
            expr = Expr::Literal(Lit::IntegerLiteral(*int));
            peek += 1;
        } else if let Some(TknType::FloatLiteral(float)) = curr_token {
            expr = Expr::Literal(Lit::FloatLiteral(*float));
            peek += 1;
        } else if let Some(TknType::CharLiteral(chr)) = curr_token {
            expr = Expr::Literal(Lit::CharLiteral(*chr));
            peek += 1;
        } else if let Some(TknType::StringLiteral(string)) = curr_token {
            expr = Expr::Literal(Lit::StringLiteral(string.clone()));
            peek += 1;
        } else if let Some(TknType::BooleanLiteral(b)) = curr_token {
            expr = Expr::Literal(Lit::BooleanLiteral(*b));
            peek += 1;
        } else if let Some(TknType::OpenParen) | Some(TknType::Dollar) = curr_token {
            expr = self.parse_group_expression(&mut peek)?;
        } else {
            return Err((
                &self.tokens[peek],
                "Unexpected Token.  Expected an Identifier, Literal, '(', or '$'".to_string(),
            ));
        }

        *index = peek;
        match op {
            None => return Ok(expr),
            Some(op) => return Ok(Expr::UnaryOp(op, Box::new(expr))),
        }
    }

    fn parse_group_expression(&self, index: &mut usize) -> Result<Expr<'p>, (&Tkn, String)> {
        if self.is_expected_token(TknType::OpenParen, index) {
            let expr = self.parse_expression(index, 0).unwrap();
            self.expect_token(TknType::CloseParen, index).unwrap();
            return Ok(expr);
        } else if self.is_expected_token(TknType::Dollar, index) {
            let expr = self.parse_expression(index, 0).unwrap();
            return Ok(expr);
        }
        return Err((
            &self.tokens[*index],
            "Unexpected Token.  Was expecting an '(' or a '$'".to_string(),
        ));
    }

    fn expect_token(&self, expected: TknType, index: &mut usize) -> Option<()> {
        let token = self.get_token_at(*index)?;
        if let TknType::Either(left, right) = expected {
            if token == left || token == right {
                *index += 1;
                return Some(());
            }
            return None;
        }

        if token == &expected {
            *index += 1;
            return Some(());
        }
        return None;
    }

    fn expect_tokens(&self, expected: &[TknType], index: &mut usize) -> Option<()> {
        let mut peek = *index;
        for i in expected {
            self.expect_token(i.clone(), &mut peek)?
        }
        *index = peek;
        return Some(());
    }

    fn is_expected_token(&self, expected: TknType, index: &mut usize) -> bool {
        let token = match self.get_token_at(*index) {
            Some(tkn) => tkn,
            None => return false,
        };
        if let TknType::Either(left, right) = expected {
            if token == left || token == right {
                *index += 1;
                return true;
            }
            return false;
        }

        if token == &expected {
            *index += 1;
            return true;
        }
        return false;
    }

    fn is_expected_tokens(&self, expected: &[TknType], index: &mut usize) -> bool {
        let mut peek = *index;
        for i in expected {
            if !self.is_expected_token(i.clone(), &mut peek) {
                return false;
            }
        }
        *index = peek;
        return true;
    }
}

pub fn get_token_from_tokens_at<'t>(tokens: &'t [Tkn], index: usize) -> Option<&'t TknType<'t>> {
    if index >= tokens.len() {
        return None;
    }
    return Some(&tokens[index].token);
}

pub fn expect_token(tokens: &[Tkn], expected: TknType, index: &mut usize) -> Option<()> {
    let token = get_token_from_tokens_at(tokens, *index)?;
    if let TknType::Either(left, right) = expected {
        if token == left || token == right {
            *index += 1;
            return Some(());
        }
        return None;
    }
    if token == &expected {
        *index += 1;
        return Some(());
    }
    return None;
}

pub fn expect_tokens(tokens: &[Tkn], expected: &[TknType], index: &mut usize) -> Option<()> {
    let mut peek = *index;
    for i in expected {
        expect_token(tokens, i.clone(), &mut peek)?
    }
    *index = peek;
    return Some(());
}

pub fn is_expected_token(tokens: &[Tkn], expected: TknType, index: &mut usize) -> bool {
    let token = match get_token_from_tokens_at(tokens, *index) {
        Some(tkn) => tkn,
        None => return false,
    };
    if let TknType::Either(left, right) = expected {
        if token == left || token == right {
            *index += 1;
            return true;
        }
        return false;
    }

    if token == &expected {
        *index += 1;
        return true;
    }
    return false;
}

pub fn is_expected_tokens(tokens: &[Tkn], expected: &[TknType], index: &mut usize) -> bool {
    let mut peek = *index;
    for i in expected {
        if !is_expected_token(tokens, i.clone(), &mut peek) {
            return false;
        }
    }
    *index = peek;
    return true;
}
