mod lexer;
mod parser;
mod token;
mod syntax;
mod string_utils;

use std::env;
use std::fs;

use syntax::{Expr, UnOp, BinOp, Lit};

fn main() {
    let file_path = env::args().nth(1).unwrap();
    println!("{}", file_path);
    let contents = fs::read_to_string(&file_path).unwrap();

    let mut lexer = lexer::Lexer::new(&file_path, &contents);
    let tokens = lexer.tokenize();
    
    dbg!(&tokens);

    let mut parser = parser::Parser::new(&file_path, tokens);

    let expr = parser.parse_expression(&mut 0, 1).unwrap();
    dbg!(&expr);
    dbg!(eval(expr));

}

fn eval(expr: Expr) -> i32 {
    if let Expr::BinaryOp(op, left, right) = expr {
        return match op {
            BinOp::Exponent => eval(*left).pow(eval(*right) as u32),
            BinOp::Multiply => eval(*left) * eval(*right),
            BinOp::IntDivide => eval(*left) / eval(*right),
            BinOp::Modulo => eval(*left) % eval(*right),
            BinOp::Plus => eval(*left) + eval(*right),
            BinOp::Minus => eval(*left) - eval(*right),
            _ => todo!("{:?} not implemented yet", op)
        }
    } else if let Expr::UnaryOp(op, value) = expr {
        return match op {
            UnOp::Plus => eval(*value),
            UnOp::Minus => -eval(*value),
            _ => todo!("{:?} not implemented yet", op)
        }
    } else if let Expr::Literal(Lit::IntegerLiteral(int)) = expr {
        return int as i32;
    } else {
        todo!("{:?}", expr);
    }
}
