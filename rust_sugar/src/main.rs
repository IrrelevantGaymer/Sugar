#![allow(incomplete_features)]
#![feature(let_chains)]

use std::env;
use std::fs;

use sugar::lexer::token::Token;
use sugar::parser;
use sugar::lexer;
use sugar::parser::accessors::Accessor;
use sugar::parser::expr::ExprData;
use sugar::parser::expr::Lit;
use sugar::parser::functions::Fun;
use sugar::parser::functions::FunctionParamater;
use sugar::parser::operators::BinOp;
use sugar::parser::operators::UnOp;
use sugar::parser::structs::Field;
use sugar::parser::structs::Struct;
use sugar::parser::ExprBump;
use sugar::parser::FnParamBump;
use sugar::parser::ParserError;
use sugar::parser::StmtBump;
use sugar::string_utils::StringUtils;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let mut files = vec![];
    let mut settings = Settings::default();
    
    let command = match args.get(1).map(|e| e.to_lowercase()).as_deref() {
        Some("lex") => Command::Lex,
        Some("parse") => Command::Parse,
        Some("help") | None => Command::Help,
        Some("build") => Command::Build,
        Some("run") => Command::Run,
        Some(command) => {
            println!("Could not recognize command {}, try help to see commands", command);
            return;
        }
    };

    let mut index = 2;

    while index < args.len() {
        match args[index].as_str() {
            "--" => {
                index += 1;
                break;
            },
            //.. list other feature flags here
            filepath => {
                files.push(filepath);
            }
        }
        index += 1;
    }

    while index  < args.len() {
        match args[index].as_str() {
            "--minimal" => if settings.lexer_settings.message_setting != LexerMessageSetting::Default {
                println!(
                    "cannot apply flag --minimal because lexer message setting is already defined as {:?}", 
                    settings.lexer_settings.message_setting
                );
                return;
            } else {
                settings.lexer_settings.message_setting = LexerMessageSetting::Minimal;
            },
            "--verbose" => if settings.lexer_settings.message_setting != LexerMessageSetting::Default {
                println!(
                    "cannot apply flag --verbose because lexer message setting is already defined as {:?}", 
                    settings.lexer_settings.message_setting
                );
                return;
            } else {
                settings.lexer_settings.message_setting = LexerMessageSetting::Verbose;
            },
            arg => {
                println!("could not recognize flag {}", arg);
                return;
            }
        }
        index += 1;
    }

    if command == Command::Help {
        match args.get(2).map(|e| e.to_lowercase()).as_deref() {
            None => println!("{}", concat!(
                "commands:\n",
                "\tlex - Tokenizes the provided file paths, returning the tokens for debug purposes.\n",
                "\tparse - Parses the provided file paths, returning the AST for debug purposes.\n",
                "\thelp - Prints out a list of commands w/ descriptions.  Also shows flags and command format for specific commands\n",
                "\tbuild - Compiles and builds the provided file paths into an executable.\n",
                "\trun - JIT Compiles and builds the provided file paths, running the program.\n"
            )),
            Some("lex") => println!("{}", concat!(
                "Tokenizes the provided file paths, returning the tokens for debug purposes.\n",
                "Provided flags:\n",
                "\t--minimal - prints the lexed tokens with minimal information\n",
                "\t--verbose - prints the lexed tokens with all their information\n"
            )),
            Some("parse") => println!("{}", concat!(
                "Parses the provided file paths, returning the AST for debug purposes.\n",
                "No provided flags:\n",
            )),
            Some("build") => println!("not implemented yet\n"),
            Some("run") => println!("not implemented yet \n"),
            Some(command) => println!("Could not recognize command {command}\n")
        }
        return;
    }

    if files.len() > 1 {
        println!("currently multiple files are not supported, please provide only 1 file");
        return;
    } else if files.len() == 0 {
        println!("no file is provided.  A file is needed for the command");
        return;
    }

    match command {
        Command::Lex => lex(files, settings),
        Command::Parse => parse(files, settings),
        Command::Build => println!("not implemented yet"),
        Command::Run => println!("not implemented yet"),
        Command::Help => unreachable!()
    }

    return;
}

#[derive(PartialEq)]
pub enum Command {
    Lex, Parse, Help, Build, Run, 
}

pub struct Settings {
    lexer_settings: LexerSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lexer_settings: Default::default()
        }
    }
}

pub struct LexerSettings {
    message_setting: LexerMessageSetting,
}

impl Default for LexerSettings {
    fn default() -> Self {
        LexerSettings { 
            message_setting: Default::default()    
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexerMessageSetting {
    Minimal, Verbose, Default
}

impl Default for LexerMessageSetting {
    fn default() -> Self {
        LexerMessageSetting::Default
    }
}

fn lex(filepaths: Vec<&str>, settings: Settings) {
    let filepath = filepaths.first().unwrap();
    let contents = fs::read_to_string(filepath).unwrap();
    let mut lexer = lexer::tokenize::Lexer::new(&filepath, &contents);
    let tokens = lexer.tokenize();

    match settings.lexer_settings.message_setting {
        LexerMessageSetting::Default |
        LexerMessageSetting::Minimal => {
            print!("[");
            for token in tokens {
                print!("{}, ", token.token);
            }
            println!("]");
        },
        LexerMessageSetting::Verbose => {
            println!("{tokens:#?}");
        }
    }
}

fn parse(filepaths: Vec<&str>, _settings: Settings) {
    let filepath = filepaths.first().unwrap();
    let contents = fs::read_to_string(filepath).unwrap();
    let mut lexer = lexer::tokenize::Lexer::new(&filepath, &contents);
    let tokens = lexer.tokenize();

    let expr_bump = ExprBump::new();
    let stmt_bump = StmtBump::new();
    let fn_param_bump = FnParamBump::new();

    match parser::parse(&expr_bump, &stmt_bump, &fn_param_bump, &tokens) {
        Ok((accessors, structs, functions)) => {
            println!("parsed accessors:\n");
            for Accessor { name, whitelist, blacklist } in accessors {
                print!("{name} whitelists [");
                for white in whitelist {
                    print!("{white}, ");
                }
                print!("] and blacklists [");
                for black in blacklist {
                    print!("{black}, ");
                }
                println!("]");
            }
            print!("\n");

            println!("parsed structs:\n");
            for Struct { accessibility, location: _, name, fields } in structs {
                println!("{name} with {accessibility} accessibility and fields {{");
                for Field { accessibility, field_name, field_type } in fields {
                    println!("\t{field_name} of type {field_type:?} and accessibility {accessibility},");
                }
                println!("}}")
            }
            print!("\n");

            println!("parsed functions:\n");
            for Fun { 
                accessibility, 
                location: _, 
                name, 
                mutable, 
                recursive, 
                left_args, 
                right_args, 
                return_type, 
                body 
            } in functions {
                print!(
                    "{}{}function {name} with accessibility {accessibility} and left args [", 
                    if mutable {"mutable "} else {""}, 
                    if recursive {"recursive "} else {""},
                );
                for FunctionParamater { param_name, param_type, param_default: _ } in left_args {
                    print!("{} of type {param_type:?}, ", param_name.as_ref().unwrap());
                }
                print!("] and right args [");
                for FunctionParamater { param_type, param_name, param_default: _ } in right_args {
                    print!("{} of type {param_type:?}, ", param_name.as_ref().unwrap());
                }
                println!("] and return type {return_type:?} and statements {{");
                for stmt in body {
                    println!("\t{stmt:?},");
                }
                println!("}}\n");
            }
            print!("\n");
        },
        Err(errors) => for ParserError { 
            token: Token { token: _, file_name, line_index, line_number }, 
            msg 
        } in errors {
            println!("error: {msg} in {file_name} on line {line_number}, col {line_index}\n");
            let line = get_line_from_contents(*line_number, contents.as_str());
            let col_offset = num_whitespace(line);
            println!("{}", get_line_from_contents(*line_number, contents.as_str()).trim());
            println!("{}^", " ".repeat(col_offset));
            println!("\nrecommendations for errors not implemented yet\n");
        }
    };
}

fn get_line_from_contents(line_number: usize, contents: &str) -> &str {
    let mut index = 0;
    for _ in 1..line_number {
        while let Some(chr) = contents.chars().nth(index) && chr != 'n' {
            index += 1;
        }
        index += 1;
    }

    let mut len = 0;
    while let Some(chr) = contents.chars().nth(index + len) && chr != '\n' {
        len += 1;
    }

    return contents.substring(index, len);
}

fn num_whitespace(line: &str) -> usize {
    let mut len = 0;

    for chr in line.chars() {
        if chr.is_whitespace() {
            len += 1;
            continue;
        }
        break;
    }

    return len;
}

#[allow(dead_code)]
fn eval<'e>(bump: &'e bumpalo::Bump, expr: &'e ExprData) -> &'e ExprData<'e> {
    if let ExprData::BinaryOp(op, left, right) = expr {
        let left_expr = eval(bump, left);
        let right_expr = eval(bump, right);
        match op {
            BinOp::Exponent => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    if *num2 < 0 {
                        return bump.alloc(ExprData::Literal(Lit::FloatLiteral((*num1 as f64).powf(*num2 as f64))));
                    }
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(num1.pow(*num2 as u32))));
                }
            }
            BinOp::Multiply => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(num1 * num2)));
                }
            }
            BinOp::IntDivide => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(num1 / num2)));
                }
            }
            BinOp::Modulo => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(num1 % num2)));
                }
            }
            BinOp::Plus => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(num1 + num2)));
                }
            }
            BinOp::Minus => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(num1 - num2)));
                }
            }
            BinOp::LogicAnd => {
                if let ExprData::Literal(Lit::BooleanLiteral(b1)) = left_expr
                    && let ExprData::Literal(Lit::BooleanLiteral(b2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(*b1 && *b2)));
                }
            }
            BinOp::LogicOr => {
                if let ExprData::Literal(Lit::BooleanLiteral(b1)) = left_expr
                    && let ExprData::Literal(Lit::BooleanLiteral(b2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(*b1 || *b2)));
                }
            }
            BinOp::LogicXor => {
                if let ExprData::Literal(Lit::BooleanLiteral(b1)) = left_expr
                    && let ExprData::Literal(Lit::BooleanLiteral(b2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(b1 ^ b2)));
                }
            }
            BinOp::Equals => {
                return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(left_expr == right_expr)));
            }
            BinOp::NotEquals => {
                return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(left_expr != right_expr)));
            }
            BinOp::LessThan => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(num1 < num2)));
                }
            }
            BinOp::LessThanEqualTo => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(num1 <= num2)));
                }
            }
            BinOp::GreaterThan => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(num1 > num2)));
                }
            }
            BinOp::GreaterThanEqualTo => {
                if let ExprData::Literal(Lit::IntegerLiteral(num1)) = left_expr
                    && let ExprData::Literal(Lit::IntegerLiteral(num2)) = right_expr
                {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(num1 >= num2)));
                }
            }
            _ => todo!("{:?} not implemented yet", op),
        }
        return bump.alloc(ExprData::BinaryOp(op.clone(), bump.alloc(left_expr), bump.alloc(right_expr)));
    } else if let ExprData::UnaryOp(op, value) = expr {
        let expr = eval(bump, value);
        match op {
            UnOp::Plus => {
                if let ExprData::Literal(Lit::IntegerLiteral(num)) = expr {
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(*num)));
                }
            }
            UnOp::Minus => {
                if let ExprData::Literal(Lit::IntegerLiteral(num)) = expr {
                    return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(-num)));
                }
            }
            UnOp::LogicNot => {
                if let ExprData::Literal(Lit::BooleanLiteral(b)) = expr {
                    return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(!b)));
                }
            }
            _ => todo!("{:?} not implemented yet", op),
        }
        return bump.alloc(ExprData::UnaryOp(op.clone(), bump.alloc(expr)));
    } else if let ExprData::Literal(Lit::IntegerLiteral(int)) = expr {
        return bump.alloc(ExprData::Literal(Lit::IntegerLiteral(*int)));
    } else if let ExprData::Literal(Lit::BooleanLiteral(b)) = expr {
        return bump.alloc(ExprData::Literal(Lit::BooleanLiteral(*b)));
    } else {
        todo!("{:?}", expr);
    }
}