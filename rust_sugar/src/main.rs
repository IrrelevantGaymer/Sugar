mod lexer;
mod option_result_utils;
mod parser;
mod parser_fn;
mod string_utils;
mod syntax;
mod token;

use std::env;
use std::fs;

use syntax::{BinOp, Expr, Lit, UnOp};

use if_chain::if_chain;

fn main() {
    let file_path = env::args().nth(1).unwrap();
    println!("{}", file_path);
    let contents = fs::read_to_string(&file_path).unwrap();

    let mut lexer = lexer::Lexer::new(&file_path, &contents);
    let tokens = lexer.tokenize();

    //dbg!(&tokens);

    let mut parser = parser::Parser::new(&file_path, tokens);
    parser.parse();

    //let expr = parser.parse_expression(&mut 0, 1).unwrap();
    //dbg!(&expr);
    //dbg!(eval(expr));
}

fn eval(expr: Expr) -> Expr {
    if let Expr::BinaryOp(op, left, right) = expr {
        let left_expr = eval(*left);
        let right_expr = eval(*right);
        match op {
            BinOp::Exponent => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        if num2 < 0 {
                            return Expr::Literal(Lit::FloatLiteral((num1 as f64).powf(num2 as f64)));
                        }
                        return Expr::Literal(Lit::IntegerLiteral(num1.pow(num2 as u32)));
                    }
                }
            }
            BinOp::Multiply => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::IntegerLiteral(num1 * num2));
                    }
                }
            }
            BinOp::IntDivide => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::IntegerLiteral(num1 / num2));
                    }
                }
            }
            BinOp::Modulo => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::IntegerLiteral(num1 % num2));
                    }
                }
            }
            BinOp::Plus => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::IntegerLiteral(num1 + num2));
                    }
                }
            }
            BinOp::Minus => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::IntegerLiteral(num1 - num2));
                    }
                }
            }
            BinOp::LogicAnd => {
                if_chain! {
                    if let Expr::Literal(Lit::BooleanLiteral(b1)) = left_expr;
                    if let Expr::Literal(Lit::BooleanLiteral(b2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::BooleanLiteral(b1 && b2));
                    }
                }
            }
            BinOp::LogicOr => {
                if_chain! {
                    if let Expr::Literal(Lit::BooleanLiteral(b1)) = left_expr;
                    if let Expr::Literal(Lit::BooleanLiteral(b2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::BooleanLiteral(b1 || b2));
                    }
                }
            }
            BinOp::LogicXor => {
                if_chain! {
                    if let Expr::Literal(Lit::BooleanLiteral(b1)) = left_expr;
                    if let Expr::Literal(Lit::BooleanLiteral(b2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::BooleanLiteral(b1 ^ b2));
                    }
                }
            }
            BinOp::Equals => {
                return Expr::Literal(Lit::BooleanLiteral(left_expr == right_expr));
            }
            BinOp::NotEquals => {
                return Expr::Literal(Lit::BooleanLiteral(left_expr != right_expr));
            }
            BinOp::LessThan => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::BooleanLiteral(num1 < num2));
                    }
                }
            }
            BinOp::LessThanEqualTo => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::BooleanLiteral(num1 <= num2));
                    }
                }
            }
            BinOp::GreaterThan => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::BooleanLiteral(num1 > num2));
                    }
                }
            }
            BinOp::GreaterThanEqualTo => {
                if_chain! {
                    if let Expr::Literal(Lit::IntegerLiteral(num1)) = left_expr;
                    if let Expr::Literal(Lit::IntegerLiteral(num2)) = right_expr;
                    then {
                        return Expr::Literal(Lit::BooleanLiteral(num1 >= num2));
                    }
                }
            }
            _ => todo!("{:?} not implemented yet", op),
        }
        return Expr::BinaryOp(op, Box::new(left_expr), Box::new(right_expr));
    } else if let Expr::UnaryOp(op, value) = expr {
        let expr = eval(*value);
        match op {
            UnOp::Plus => {
                if let Expr::Literal(Lit::IntegerLiteral(num)) = expr {
                    return Expr::Literal(Lit::IntegerLiteral(num));
                }
            }
            UnOp::Minus => {
                if let Expr::Literal(Lit::IntegerLiteral(num)) = expr {
                    return Expr::Literal(Lit::IntegerLiteral(-num));
                }
            }
            UnOp::LogicNot => {
                if let Expr::Literal(Lit::BooleanLiteral(b)) = expr {
                    return Expr::Literal(Lit::BooleanLiteral(!b));
                }
            }
            _ => todo!("{:?} not implemented yet", op),
        }
        return Expr::UnaryOp(op, Box::new(expr));
    } else if let Expr::Literal(Lit::IntegerLiteral(int)) = expr {
        return Expr::Literal(Lit::IntegerLiteral(int));
    } else if let Expr::Literal(Lit::BooleanLiteral(b)) = expr {
        return Expr::Literal(Lit::BooleanLiteral(b));
    } else {
        todo!("{:?}", expr);
    }
}
