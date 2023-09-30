mod lexer;
mod string_utils;
mod token;

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
