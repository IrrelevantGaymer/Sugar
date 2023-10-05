#[allow(dead_code)]

use crate::token::{Tkn, TknType, Kwrd, Op};
use crate::{syntax::{
    Stmt, Expr, Fn, FnParam, 
    OpAssoc, OpPrec, OPERATOR_INFO_MAP, 
    UnOp, BinOp, Lit}, option_result_utils::Resultable};
use if_chain::if_chain;


pub struct Parser<'p> {
    tokens: Vec<Tkn<'p>>,
    index: usize,
    location: &'p str,
}

impl<'p> Parser<'p> {
    pub fn new (location: &'p str, tokens: Vec<Tkn<'p>>) -> Parser<'p> {
        return Parser {
            tokens: tokens,
            index: 0,
            location: location
        };
    }

    fn get_token(&self) -> &TknType {
        return &self.tokens[self.index].token;
    }

    fn get_token_at(&self, index: usize) -> &TknType {
        return &self.tokens[index].token;
    }

    pub fn parse(&mut self) {
        let accessor_names: Vec<(String, &[Tkn])> = vec![];
        let struct_names: Vec<(String, &[Tkn])> = vec![];
        let function_names: Vec<(String, usize, usize, &[Tkn])> = vec![];
        let actions: Vec<Stmt> = vec![];


    }

    pub fn define_accessor(&mut self) -> Option<(String, &[Tkn])> {
        todo!();
    }

    pub fn define_struct(&mut self) -> Option<(String, &[Tkn])> {
        todo!();
    }

    pub fn define_function(&mut self) -> Option<(String, usize, usize, &[Tkn])> {
        let mut peek = self.index;
        
        todo!();
    }

    pub fn parse_function(&mut self) -> Option<Fn> {
        let mut peek = self.index;
        let accessibility: String = match &self.tokens[self.index].token {
            TknType::Keyword(Kwrd::Public) => {
                peek += 1;
                String::from("public")
            },
            TknType::Keyword(Kwrd::Private) => {
                peek += 1;
                String::from("private")
            },
            TknType::Keyword(Kwrd::Package) => {
                peek += 1;
                String::from("package")
            },
            TknType::Identifier(ident) => {
                ident.clone()
            },
            TknType::Keyword(Kwrd::Mutable) 
            | TknType::Keyword(Kwrd::Recursive)
            | TknType::Keyword(Kwrd::Function) => String::from("private"),
            _ => return None
        };

        let mutable = self.is_expected_token(TknType::Keyword(Kwrd::Mutable), &mut peek);
        let recursive = self.is_expected_token(TknType::Keyword(Kwrd::Recursive), &mut peek);
        let name;

        self.expect_token(TknType::Keyword(Kwrd::Function), &mut peek)?;
        if let TknType::Identifier(ident) = self.get_token() {
            name = ident.clone();
        } else {
            return None;
        }

        let arguments: &[FnParam] = &[];
        let stmt: Stmt;
        match self.get_token() {
            TknType::OpenParen => todo!(),
            TknType::Dollar => todo!(),
            TknType::Colon => todo!(),
            TknType::OpenCurlyBrace => {
                peek += 1;
                let mut stmts = vec![];
                while let Some(stmt) = self.parse_statement() {
                    stmts.push(stmt);
                }
                self.expect_token(TknType::CloseCurlyBrace, &mut peek)?;
                stmt = Stmt::Compound(stmts);
            },
            TknType::Operation(Op::Arrow) => {
                peek += 1;
                stmt = self.parse_compound_statement()?;
            },
            TknType::NewLine => todo!(),
            _ => return None
        }

        let location = String::from(self.location);
        
        return Some(Fn {
            location: location,
            accessibility: accessibility,
            mutable: mutable,
            recursive: recursive,
            name: name,
            arguments: arguments,
            body: stmt
        });
    }

    fn parse_accessor(&self) {

    }

    fn parse_struct(&self) {

    }

    fn parse_statement(&self) -> Option<Stmt> {
        todo!();
    }
    
    fn parse_variable_declaration(&self, index: &mut usize) -> Result<Vec<Stmt>, (&Tkn, String)> {
        let mut peek = *index;
        let data_type: Option<String> = match self.get_token_at(peek) {
            TknType::Type(data) => Some(data.to_string()),
            TknType::Keyword(Kwrd::Let) => None,
            TknType::Identifier(data) => Some(data.to_string()),
            _ => return Err((&self.tokens[*index], "Unexpected Token.  Expected a Type, Identifier, or Let".to_string()))
        };

        peek += 1;
        let mut idents = vec![];
        let mut stmts = vec![];
        while let TknType::Identifier(ident) = self.get_token_at(peek) {
            idents.push(ident.clone());
            stmts.push(Stmt::Declare(data_type.clone(), ident.clone()));
            peek += 1;
        }

        if idents.len() == 0 { return Err((
            &self.tokens[*index], 
            "Expected identifier(s) to be declared".to_string()
        )); }

        if_chain!{
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

        self.expect_token(TknType::Either(
            &TknType::Semicolon,
            &TknType::NewLine
        ), &mut peek)
        .to_result_with_error((
            &self.tokens[peek], 
            "Expected Semicolon or Newline".to_string()
        ))?;

        return Ok(stmts);
    }

    fn parse_variable_assignment(&self, index: usize) -> Result<Vec<Stmt>, (&Tkn, String)> {
        let mut peek = index;
        let mut idents: Vec<String> = vec![];

        loop {
            if_chain!{
                if let TknType::Identifier(ident) = self.get_token_at(peek);
                if let TknType::Operation(Op::Assign) = self.get_token_at(peek + 1);
                then {
                    idents.push(ident.clone());
                    peek += 2;
                    continue;
                } else {
                    break;
                }
            }
        }

        let token_index = peek;
        if let Ok(expr) = self.parse_expression(&mut peek, 0) {
            let mut stmts: Vec<Stmt> = vec![];
            for i in 0..idents.len() {
                stmts.push(Stmt::Assign(Expr::Identifier(idents[i].clone()), expr.clone()));
            }
            return Ok(stmts);
        }

        return Err((
            &self.tokens[token_index], 
            "".to_string()
        ));
    }

    fn parse_variable_insertion(&self, index: usize) -> Option<Stmt> {
        let mut peek = index;

        

        return None;
    }

    fn parse_compound_statement(&self) -> Option<Stmt> {
        
        
        todo!();
    }

    pub fn parse_expression(&self, index: &mut usize, min_prec: u32) -> Result<Expr<'p>, (&Tkn, String)> {
        let mut peek = *index;
        let mut left_expr = self.parse_atom(&mut peek)?;
        loop {
            let operator = self.get_token_at(peek);

            if !OPERATOR_INFO_MAP.contains_key(operator) {
                break;
            }

            let (prec, assoc) = OPERATOR_INFO_MAP[operator];
            let prec = prec as u32;

            if prec < min_prec {
                break;
            }

            let next_min_prec = if assoc == OpAssoc::Left { prec + 1 } else { prec };

            peek += 1;

            let right_expr = self.parse_expression(&mut peek, next_min_prec)?;

            left_expr = Expr::BinaryOp(BinOp::get_bin_op(operator), 
                Box::new(left_expr), 
                Box::new(right_expr)
            );
        }

        *index = peek;
        return Ok(left_expr);
    }

    fn parse_atom(&self, index: &mut usize) -> Result<Expr<'p>, (&Tkn, String)> {
        let mut peek = *index;
        let mut curr_token = self.get_token_at(peek);

        let op: Option<UnOp>;
        if curr_token == &TknType::Operation(Op::Plus) {
            op = Some(UnOp::Plus);
            peek += 1;
        } else if curr_token == &TknType::Operation(Op::Minus) {
            op = Some(UnOp::Minus);
            peek += 1;
        } else if curr_token == &TknType::Operation(Op::LogicNot) {
            op = Some(UnOp::LogicNot);
            peek += 1;
        } else if curr_token == &TknType::Operation(Op::BitwiseNegate) {
            op = Some(UnOp::BitwiseNegate);
            peek += 1;
        } else if curr_token == &TknType::Borrow {
            if self.get_token_at(peek + 1)  == &TknType::Keyword(Kwrd::Mutable) {
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
        if let TknType::Identifier(ident) = curr_token {
            expr = Expr::Identifier(ident.clone());
            peek += 1;
        } else if let TknType::IntegerLiteral(int) = curr_token {
            expr = Expr::Literal(Lit::IntegerLiteral(*int));
            peek += 1;
        } else if let TknType::FloatLiteral(float) = curr_token {
            expr = Expr::Literal(Lit::FloatLiteral(*float));
            peek += 1;
        } else if let TknType::CharLiteral(chr) = curr_token {
            expr = Expr::Literal(Lit::CharLiteral(*chr));
            peek += 1;
        } else if let TknType::StringLiteral(string) = curr_token {
            expr = Expr::Literal(Lit::StringLiteral(string.clone()));
            peek += 1;
        } else if let TknType::BooleanLiteral(b) = curr_token {
            expr = Expr::Literal(Lit::BooleanLiteral(*b));
            peek += 1;  
        } else if curr_token == &TknType::OpenParen || curr_token == &TknType::Dollar {
            expr = self.parse_group_expression(&mut peek)?;
        } else {
            return Err((
                &self.tokens[peek], 
                "Unexpected Token.  Expected an Identifier, Literal, '(', or '$'".to_string()
            ));
        }

        *index = peek;
        match op {
            None => return Ok(expr),
            Some(op) => return Ok(Expr::UnaryOp(op, Box::new(expr)))
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
            "Unexpected Token.  Was expecting an '(' or a '$'".to_string()
        ));
    }

    fn expect_token(&self, expected: TknType, index: &mut usize) -> Option<()> {
        if let TknType::Either(left, right) = expected {
            if self.get_token_at(*index) == left 
            || self.get_token_at(*index) == right {
                *index += 1;
                return Some(());
            }
            return None;
        }

        if self.get_token_at(*index) == &expected {
            *index += 1;
            return Some(());
        }
        return None;
    }

    fn is_expected_token(&self, expected: TknType, index: &mut usize) -> bool {
        if let TknType::Either(left, right) = expected {
            if self.get_token_at(*index) == left 
            || self.get_token_at(*index) == right {
                *index += 1;
                return true;
            }
            return false;
        }

        if self.get_token_at(*index) == &expected {
            *index += 1;
            return true;
        }
        return false;
    }
}