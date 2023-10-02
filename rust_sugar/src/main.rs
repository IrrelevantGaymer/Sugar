mod lexer;
mod parser;
mod token;
mod syntax;
mod string_utils;

use std::env;
use std::fs;

fn main() {
    let file_path = env::args().nth(1).unwrap();
    println!("{}", file_path);
    let contents = fs::read_to_string(&file_path).unwrap();
    let mut lexer = lexer::Lexer::new(&file_path, &contents);
    let tokens = lexer.tokenize();

    dbg!(tokens);

}
