#[allow(dead_code)]

use crate::token::{Tkn, TknType, Kwrd, Op};
use crate::syntax::{
    Stmt, Expr, Fn, FnParam, 
    OpAssoc, OpPrec, OPERATOR_INFO_MAP, 
    UnOp, BinOp, Lit};
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
        let functions: Vec<Fn> = vec![];
        let actions: Vec<Stmt> = vec![];
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
    
    fn parse_variable_declaration(&self, index: usize) -> Option<Vec<Stmt>> {
        let mut peek = index;
        let data_type: Option<String> = match self.get_token_at(peek) {
            TknType::Type(data) => Some(data.to_string()),
            TknType::Keyword(Kwrd::Let) => None,
            TknType::Identifier(data) => Some(data.to_string()),
            _ => return None
        };

        peek += 1;
        let mut idents = vec![];
        let mut stmts = vec![];
        while let TknType::Identifier(ident) = self.get_token_at(peek) {
            idents.push(ident.clone());
            stmts.push(Stmt::Declare(data_type.clone(), ident.clone()));
            peek += 1;
        }

        if idents.len() == 0 { return None; }

        if_chain!{
            if self.is_expected_token(TknType::Operation(Op::Assign), &mut peek);
            if let Some(expr) = self.parse_expression(&mut peek, 0);
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
        ), &mut peek)?;

        return Some(stmts);
    }

    fn parse_variable_assignment(&self, index: usize) -> Option<Vec<Stmt>> {
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

        if let Some(expr) = self.parse_expression(&mut peek, 0) {
            let mut stmts: Vec<Stmt> = vec![];
            for i in 0..idents.len() {
                stmts.push(Stmt::Assign(Expr::Identifier(idents[i].clone()), expr.clone()));
            }
            return Some(stmts);
        }

        return None;
    }

    fn parse_variable_insertion(&self, index: usize) -> Option<Stmt> {
        let mut peek = index;

        

        return None;
    }

    fn parse_compound_statement(&self) -> Option<Stmt> {
        
        
        todo!();
    }

    pub fn parse_expression(&self, index: &mut usize, min_prec: u32) -> Option<Expr<'p>> {
        let mut peek = *index;
        let mut left_expr = self.parse_atom(&mut peek).unwrap();
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

            let right_expr = self.parse_expression(&mut peek, next_min_prec).unwrap();

            left_expr = Expr::BinaryOp(BinOp::get_bin_op(operator), 
                Box::new(left_expr), 
                Box::new(right_expr)
            );
        }

        *index = peek;
        return Some(left_expr);
    }

    fn parse_atom(&self, index: &mut usize) -> Option<Expr<'p>> {
        let peek = *index;
        let mut skip = 0;
        let curr_token = self.get_token_at(peek);

        let op: Option<UnOp>;
        if curr_token == &TknType::Operation(Op::Plus) {
            op = Some(UnOp::Plus);
            skip += 1;
        } else if curr_token == &TknType::Operation(Op::Minus) {
            op = Some(UnOp::Minus);
            skip += 1;
        } else if curr_token == &TknType::Operation(Op::LogicNot) {
            op = Some(UnOp::LogicNot);
            skip += 1;
        } else if curr_token == &TknType::Operation(Op::BitwiseNegate) {
            op = Some(UnOp::BitwiseNegate);
            skip += 1;
        } else if curr_token == &TknType::Borrow {
            if self.get_token_at(peek + 1)  == &TknType::Keyword(Kwrd::Mutable) {
                op = Some(UnOp::BorrowMutable);
                skip += 2;
            } else {
                op = Some(UnOp::Borrow);
                skip += 1;
            }
        } else {
            op = None;
        }

        let expr;
        if let TknType::Identifier(ident) = curr_token {
            expr = Expr::Identifier(ident.clone());
            skip += 1;
        } else if let TknType::IntegerLiteral(int) = curr_token {
            expr = Expr::Literal(Lit::IntegerLiteral(*int));
            skip += 1;
        } else if let TknType::FloatLiteral(float) = curr_token {
            expr = Expr::Literal(Lit::FloatLiteral(*float));
            skip += 1;
        } else if let TknType::CharLiteral(chr) = curr_token {
            expr = Expr::Literal(Lit::CharLiteral(*chr));
            skip += 1;
        } else if let TknType::StringLiteral(string) = curr_token {
            expr = Expr::Literal(Lit::StringLiteral(string.clone()));
            skip += 1;
        } else if curr_token == &TknType::OpenParen {
            expr = self.parse_group_expression(peek).unwrap();
        } else {
            return None;
        }

        *index += skip;
        match op {
            None => return Some(expr),
            Some(op) => return Some(Expr::UnaryOp(op, Box::new(expr)))
        }
    }

    fn parse_group_expression(&self, index: usize) -> Option<Expr<'p>> {
        let mut peek = index;

        self.expect_token(TknType::OpenParen, &mut peek).unwrap();
        let expr = self.parse_expression(&mut peek, 0);
        self.expect_token(TknType::CloseParen, &mut peek).unwrap();
        
        return expr;
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