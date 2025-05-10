#![allow(incomplete_features)]
#![feature(let_chains)]

use std::env;
use std::fs;

use once_cell::sync::OnceCell;
use sugar::interpreter;
use sugar::{
    lexer::{
        self, 
        //token::Token
    }, 
    parser::{
        self, 
        accessors::Accessor, 
        functions::{Fun, FnParam},
        structs::{Field, Struct},
        ExprBump,
        StmtBump,
        FnParamBump,
        //ParserError,
    }
};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let mut files = vec![];
    let mut settings = Settings::default();
    
    let command = match args.get(1).map(|e| e.to_lowercase()).as_deref() {
        Some("lex") => Command::Lex,
        Some("parse") => Command::Parse,
        Some("help") | None => Command::Help,
        Some("interpret") => Command::Interpret,
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
            "--minimal" => if settings.message_settings != MessageSetting::Default {
                println!(
                    "cannot apply flag --minimal because lexer message setting is already defined as {:?}", 
                    settings.message_settings
                );
                return;
            } else {
                settings.message_settings = MessageSetting::Minimal;
            },
            "--verbose" => if settings.message_settings != MessageSetting::Default {
                println!(
                    "cannot apply flag --verbose because lexer message setting is already defined as {:?}", 
                    settings.message_settings
                );
                return;
            } else {
                settings.message_settings = MessageSetting::Verbose;
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
                "\tinterpret - Uses the built-in interpreter to run the provided file paths\n",
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
                "Provided flags:\n",
                "\t--minimal - prints the lexed tokens with minimal information\n",
                "\t--verbose - prints the lexed tokens with all their information\n"
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
        Command::Interpret => interpret(files, settings),
        Command::Build => println!("not implemented yet"),
        Command::Run => println!("not implemented yet"),
        Command::Help => unreachable!()
    }

    return;
}

#[derive(PartialEq)]
pub enum Command {
    Lex, Parse, Help, Interpret, Build, Run, 
}

pub struct Settings {
    message_settings: MessageSetting,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            message_settings: Default::default()
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MessageSetting {
    Minimal, Verbose, Default
}

impl Default for MessageSetting {
    fn default() -> Self {
        MessageSetting::Default
    }
}

fn lex(filepaths: Vec<&str>, settings: Settings) {
    let filepath = filepaths.first().unwrap();
    let contents = fs::read_to_string(filepath).unwrap();
    let mut lexer = lexer::tokenize::Lexer::new(&filepath, &contents);
    let tokens = lexer.tokenize();

    match settings.message_settings {
        MessageSetting::Default |
        MessageSetting::Minimal => {
            print!("[");
            for token in tokens {
                print!("{}, ", token.token);
            }
            println!("]");
        },
        MessageSetting::Verbose => {
            println!("{tokens:#?}");
        }
    }
}

fn parse(filepaths: Vec<&str>, settings: Settings) {
    let filepath = filepaths.first().unwrap();
    let contents = fs::read_to_string(filepath).unwrap();
    let mut lexer = lexer::tokenize::Lexer::new(&filepath, &contents);
    let tokens = lexer.tokenize();

    let expr_bump = ExprBump::new();
    let stmt_bump = StmtBump::new();
    let fn_param_bump = FnParamBump::new();

    let accessors = OnceCell::new();
    let structs = OnceCell::new();
    let functions = OnceCell::new();

    match settings.message_settings {
        MessageSetting::Default |
        MessageSetting::Minimal => {
            match parser::parse(
                &expr_bump, &stmt_bump, &fn_param_bump, 
                &accessors, &structs, &functions, 
                &tokens
            ) {
                Ok(()) => {
                    println!("parsed accessors:\n");
                    for Accessor { ref name, ref whitelist, ref blacklist } in accessors.get().unwrap() {
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
                    for Struct { accessibility, location: _, name, fields } in structs.get().unwrap() {
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
                    } in functions.get().unwrap() {
                        print!(
                            "{}{}function {name} with accessibility {accessibility} and left args [", 
                            if *mutable {"mutable "} else {""}, 
                            if *recursive {"recursive "} else {""},
                        );
                        for FnParam { param_name, param_type, .. } in *left_args {
                            print!("{} of type {param_type:?}, ", param_name.as_ref().unwrap());
                        }
                        print!("] and right args [");
                        for FnParam { param_type, param_name, .. } in *right_args {
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
                Err(errors) => for parser_error in errors {
                    parser_error.write(&mut std::io::stdout(), contents.as_str()).unwrap();
                }
            };
        },
        MessageSetting::Verbose => {
            match parser::parse(
                &expr_bump, &stmt_bump, &fn_param_bump,
                &accessors, &structs, &functions,
                &tokens
            ) {
                Ok(()) => {
                    println!("accessors:");
                    for accessor in accessors.get().unwrap() {
                        println!("{accessor:#?}");
                    }
                    println!("");
                    
                    println!("accessors:");
                    for dbg_struct in structs.get().unwrap() {
                        println!("{dbg_struct:#?}");
                    }
                    println!("");

                    println!("accessors:");
                    for function in functions.get().unwrap() {
                        println!("{function:#?}");
                    }
                    println!("")
                },
                Err(errors) => for parser_error in errors {
                    parser_error.write(&mut std::io::stdout(), contents.as_str()).unwrap();
                }
            }
        }
    }
}

fn interpret(filepaths: Vec<&str>, settings: Settings) {
    let filepath = filepaths.first().unwrap();
    let contents = fs::read_to_string(filepath).unwrap();
    let mut lexer = lexer::tokenize::Lexer::new(&filepath, &contents);
    let tokens = lexer.tokenize();

    let expr_bump = ExprBump::new();
    let stmt_bump = StmtBump::new();
    let fn_param_bump = FnParamBump::new();

    let accessors = OnceCell::new();
    let structs = OnceCell::new();
    let functions = OnceCell::new();

    let parsed = parser::parse(&expr_bump, &stmt_bump, &fn_param_bump, &accessors, &structs, &functions, &tokens);

    match parsed {
        Err(errors) => {
            match settings.message_settings {
                MessageSetting::Default |
                MessageSetting::Minimal => {
                    for parser_error in errors {
                        parser_error.write(&mut std::io::stdout(), contents.as_str()).unwrap();
                    }
                    return;
                },
                MessageSetting::Verbose => {
                    for parser_error in errors {
                        parser_error.write(&mut std::io::stdout(), contents.as_str()).unwrap();
                    }
                    return;
                }
            }
        },
        Ok(()) => ()
    };

    let mut interpreter = interpreter::Interpreter::new((accessors.get().unwrap(), structs.get().unwrap(), functions.get().unwrap()));
    //println!("starting");
    interpreter.interpret(&expr_bump);
}