use crate::string_utils::StringUtils;
use crate::token::{Kwrd, Op, Type, Tkn, TknType};

const SINGLE_TOKEN_CHARACTERS: &str = "(){}[];?.";
const OPERATOR_CHARACTERS: &str = "+-*/%&|^=!~:<>";
const OPERATORS: &[&str] = &[
    "!..=", "!..", "..=", "..",
    "++=", "+=", "-=", "**=", "*=", "//=", "/=", "%=",
    "&&=", "||=", "^^=", "=!", "&=", "|=", "^=", "=~", "<<=", ">>=",
    "==", "!=", "<=", ">=", "=", "->",
    "++", "+", "--", "-", "**", "*", "//", "/", "%",
    "&&", "||", "^^", "!", "&", "|", "^", "~", "<<", ">>",
    "<", ">",
];

pub struct Lexer<'a> {
    file_name: &'a str,
    source_code: &'a str,
    peek: usize,
    line_index: usize,
    line_number: usize,
}

impl<'a> Lexer<'a> {
    fn consume(&mut self, num: usize) {
        self.peek += num;
    }
    
    pub fn new (file: &'a str, src: &'a str) -> Lexer<'a> {
        return Lexer {
            file_name: file,
            source_code: src,
            peek: 0,
            line_index: 1,
            line_number: 1
        };
    }

    pub fn set_file(&mut self, file: &'a str) {
        self.file_name = file;
    }

    pub fn set_source_code(&mut self, src: &'a str) {
        self.source_code = src;
        self.peek = 0;
        self.line_index = 1;
        self.line_number = 1;
    }
    
    pub fn tokenize(&mut self) -> Vec<Tkn> {
        let mut tokens: Vec<Tkn> = Vec::new();
        let mut new_line: bool = true;

        while let Some(chr) = self.source_code.chars().nth(self.peek) {
            let token: TknType;
            let index = self.line_index;
            let line = self.line_number;

            println!("char: \'{}\'", chr);
            match chr {
                '?' => {
                    token = TknType::Question;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                '$' => {
                    token = TknType::Dollar;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                }
                ':' => {
                    if let Some(':') = self.source_code.chars().nth(self.peek + 1) {
                        token = TknType::ColonColon;
                        new_line = false;
                        self.line_index += 2;
                        self.consume(2);
                    } else {
                        token = TknType::Colon;
                        new_line = false;
                        self.line_index += 1;
                        self.consume(1);
                    }
                },
                ';' => {
                    token = TknType::Semicolon;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                '(' => {
                    token = TknType::OpenParen;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                ')' => {
                    token = TknType::CloseParen;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                '{' => {
                    token = TknType::OpenCurlyBrace;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                '}' => {
                    token = TknType::CloseCurlyBrace;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                '[' => {
                    token = TknType::OpenSquareBracket;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                ']' => {
                    token = TknType::CloseSquareBracket;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(1);
                },
                '\n' | '\r' => {
                    self.line_index = 1;
                    self.line_number += 1;
                    self.consume(1);
                    
                    if !new_line {
                        continue;
                    }
                    
                    token = TknType::NewLine;
                    new_line = true;
                },
                ' ' => {
                    let spaces = self.count_spaces();
                    self.line_index += spaces;
                    self.consume(spaces);

                    if new_line {
                        token = TknType::Spaces(spaces);
                        new_line = false;
                    } else {
                        continue;
                    }
                },
                _ => {
                    token = self.get_multi_character_token();
                    new_line = false;
                }
            }

            tokens.push(Tkn::new(
                token, 
                self.file_name.to_string(), 
                index, 
                line
            ));

            println!("token: \n\t{}\npeek: {}", tokens[tokens.len() - 1], self.peek);
        }

        tokens.push(Tkn::new(
            TknType::EndOfFile, 
            self.file_name.to_string(), 
            self.line_index, 
            self.line_number
        ));

        return tokens;
    }

    fn count_spaces(&mut self) -> usize {
        let mut spaces: usize = 0;
        let mut index: usize = self.peek;

        while let Some(' ') = self.source_code.chars().nth(index) {
            spaces += 1;
            index += 1;
        }

        return spaces;
    }

    fn get_multi_character_token(&mut self) -> TknType<'a>{
        let multi_character: &'a str;
        let start_index: usize = self.peek;
        let mut end_index: usize = self.peek;

        while let Some(chr) = self.source_code.chars().nth(end_index) {
            if is_invalid_character(chr) {
                break;
            }
            end_index += 1;
        }

        multi_character = self.source_code.slice(start_index..end_index);
        println!("multi_character: \"{}\"", multi_character);
        match multi_character {
            "let" => {
                self.consume(3);
                return TknType::Keyword(Kwrd::Let);
            },
            "loop" => {
                self.consume(4);
                return TknType::Keyword(Kwrd::Loop);
            },
            "mut" => {
                self.consume(3);
                return TknType::Keyword(Kwrd::Mutable);
            },
            "oxy" => {
                self.consume(3);
                return TknType::Keyword(Kwrd::Oxidize);
            },
            "unsafe" => {
                self.consume(6);
                return TknType::Keyword(Kwrd::Unsafe);
            },
            "fn" | "fun" | "function" => {
                self.consume(multi_character.len());
                return TknType::Keyword(Kwrd::Function);
            },
            "accessor" => {
                self.consume(7);
                return TknType::Keyword(Kwrd::Accessor);
            },
            "whitelist" => {
                self.consume(9);
                return TknType::Keyword(Kwrd::Whitelist);
            },
            "blacklist" => {
                self.consume(9);
                return TknType::Keyword(Kwrd::Blacklist);
            },
            "struct" | "class" => {
                self.consume(multi_character.len());
                return TknType::Keyword(Kwrd::Struct);
            },
            "namespace" => {
                self.consume(9);
                return TknType::Keyword(Kwrd::Namespace);
            },
            "alias" => {
                self.consume(5);
                return TknType::Keyword(Kwrd::Alias);
            },
            "pub" => {
                self.consume(3);
                return TknType::Keyword(Kwrd::Public);
            },
            "prv" => {
                self.consume(3);
                return TknType::Keyword(Kwrd::Private);
            },
            "pkg" => {
                self.consume(3);
                return TknType::Keyword(Kwrd::Package);
            },  
            _ => {
                return self.get_operation()
                    .or_else(|| self.get_integer_literal(multi_character))
                    .or_else(|| self.get_float_literal(multi_character))
                    .or_else(|| self.get_char_literal(multi_character))
                    .or_else(|| self.get_string_literal(multi_character))
                    .or_else(|| self.get_type(multi_character))
                    .or_else(|| self.get_identifier(multi_character))
                    .unwrap_or_else(|| self.get_invalid(multi_character));
            }
        };
    }

    fn get_operation(&mut self) -> Option<TknType<'a>> {
        if let Some(chr) = self.source_code.chars().nth(self.peek) {
            if !OPERATOR_CHARACTERS.contains(chr) {
                return None;
            }
        } else {
            return None;
        }
        
        for i in 0..OPERATORS.len() {
            if self.source_code.len() - self.peek < OPERATORS[i].len() {
                continue;
            }
            
            let operator = self.source_code.slice(self.peek..self.peek + OPERATORS[i].len());
            println!("token: \"{}\", operator: \"{}\"", operator, OPERATORS[i]);
            if operator == OPERATORS[i] {
                self.consume(   OPERATORS[i].len());
                return match operator {
                    "!..=" => Some(TknType::Operation(Op::BangRangeEquals)), 
                    "!.." => Some(TknType::Operation(Op::BangRange)), 
                    "..=" => Some(TknType::Operation(Op::RangeEquals)), 
                    ".." => Some(TknType::Operation(Op::Range)), 
                    "==" => Some(TknType::Operation(Op::Equals)), 
                    "!=" => Some(TknType::Operation(Op::NotEquals)), 
                    "<=" => Some(TknType::Operation(Op::LessThanEqualTo)),
                    ">=" => Some(TknType::Operation(Op::GreaterThanEqualTo)), 
                    "<" => Some(TknType::Either(
                      &TknType::Operation(Op::LessThan),
                      &TknType::OpenAngularBracket
                    )), 
                    ">" => Some(TknType::Either(
                      &TknType::Operation(Op::GreaterThan),
                      &TknType::CloseAngularBracket
                    )), 
                    "++=" => Some(TknType::Operation(Op::ConcatEquals)), 
                    "+=" => Some(TknType::Operation(Op::PlusEquals)),
                    "-=" => Some(TknType::Operation(Op::MinusEquals)), 
                    "**=" => Some(TknType::Operation(Op::ExponentEquals)), 
                    "*=" => Some(TknType::Operation(Op::MultiplyEquals)), 
                    "//=" => Some(TknType::Operation(Op::FloatDivideEquals)), 
                    "/=" => Some(TknType::Operation(Op::IntDivideEquals)), 
                    "&&=" => Some(TknType::Operation(Op::LogicAndEquals)), 
                    "&=" => Some(TknType::Operation(Op::BitwiseAndEquals)), 
                    "||=" => Some(TknType::Operation(Op::LogicOrEquals)), 
                    "|=" => Some(TknType::Operation(Op::BitwiseOrEquals)), 
                    "^^=" => Some(TknType::Operation(Op::LogicXorEquals)), 
                    "^=" => Some(TknType::Operation(Op::BitwiseXorEquals)), 
                    "=!" => Some(TknType::Operation(Op::EqualsNot)), 
                    "=~" => Some(TknType::Operation(Op::EqualsNegate)), 
                    "++" => Some(TknType::Operation(Op::PlusPlus)), 
                    "--" => Some(TknType::Operation(Op::MinusMinus)), 
                    "+" => Some(TknType::Operation(Op::Plus)), 
                    "-" => Some(TknType::Operation(Op::Minus)), 
                    "**" => Some(TknType::Operation(Op::Exponent)), 
                    "*" => Some(TknType::Operation(Op::Multiply)), 
                    "//" => Some(TknType::Operation(Op::Equals)), 
                    "/" => Some(TknType::Operation(Op::Equals)), 
                    "&&" => Some(TknType::Operation(Op::Equals)), 
                    "||" => Some(TknType::Operation(Op::Equals)), 
                    "^^" => Some(TknType::Operation(Op::Equals)),
                    "&" => Some(TknType::Operation(Op::Equals)), 
                    "|" => Some(TknType::Operation(Op::Equals)), 
                    "^" => Some(TknType::Operation(Op::Equals)), 
                    "=" => Some(TknType::Operation(Op::Equals)),
                    "->" => Some(TknType::Operation(Op::Equals)),
                    _ => unreachable!()
                }
            }
        }
        
        return None;
    }

    fn get_integer_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        match multi_character.parse() {
            Ok(int) => {
                self.consume(multi_character.len());
                return Some(TknType::IntegerLiteral(int));
            },
            Err(_) => return None
        }
    }

    fn get_float_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        return None;
    }

    fn get_char_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        return None;
    }

    fn get_string_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        return None;
    }


    fn get_type(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        match multi_character.chars().nth(0) {
            Some('i') => {
                
            },
            Some('u') => {
                
            },
            Some('f') => {
                
            },
            _ => ()
        }

        match multi_character {
            "char" => {
                self.consume(4);
                return Some(TknType::Type(Type::Character));
            },
            "bool" => {
                self.consume(4);
                return Some(TknType::Type(Type::Boolean));
            },
            _ => (),
        }
        
        return None;
    }

    fn get_identifier(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        let mut break_index: usize = 0;

        if let Some(chr) = multi_character.chars().nth(0) {
            if chr.is_numeric() {
                return None;
            }
        } else {
            return None;
        }

        while let Some(chr) = multi_character.chars().nth(break_index) {
            if chr == ':' || chr == '.' {
                break;
            }
            break_index += 1;
        }
        
        let multi_character = multi_character.slice(0..break_index);
        self.consume(break_index);

        return Some(TknType::Identifier(multi_character.to_string()));
    }

    fn get_invalid(&mut self, multi_character: &str) -> TknType<'a> {
        self.consume(multi_character.len());
        return TknType::Invalid;
    }
}

fn is_invalid_character(chr: char) -> bool {
    return chr == ' ' 
        || chr == '\n' 
        || chr == '\r' 
        || SINGLE_TOKEN_CHARACTERS.contains(chr)
        || OPERATOR_CHARACTERS.contains(chr);
}
