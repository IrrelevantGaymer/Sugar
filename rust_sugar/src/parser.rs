use crate::token;
use crate::token::{Tkn, TknType, Kwrd};
use crate::syntax;
use crate::syntax::{Stmt, Expr, Fn, FnParam};
use if_chain::if_chain;


pub struct Parser<'p> {
    tokens: &'p [Tkn<'p>],
    index: usize,
    location: String,
}

impl<'p> Parser<'p> {

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
            TknType::Operation(token::Op::Arrow) => {
                peek += 1;
                stmt = self.parse_compound_statement()?;
            },
            TknType::NewLine => todo!(),
            _ => return None
        }

        return Some(Fn {
            location: self.location.clone(),
            accessibility: accessibility,
            mutable: mutable,
            recursive: recursive,
            name: name,
            arguments: arguments,
            body: stmt
        });
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
            if self.is_expected_token(TknType::Operation(token::Op::Assign), &mut peek);
            if let Some(expr) = self.parse_expression();
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
                if let TknType::Operation(token::Op::Assign) = self.get_token_at(peek + 1);
                then {
                    idents.push(ident.clone());
                    peek += 2;
                    continue;
                } else {
                    break;
                }
            }
        }

        if let Some(expr) = self.parse_expression() {
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

    fn parse_expression(&self) -> Option<Expr> {
        todo!();
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