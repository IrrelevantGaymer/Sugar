use crate::string_utils::StringUtils;

use super::token::{Kwrd, Op, Type, Tkn, TknType};

const SINGLE_TOKEN_CHARACTERS: &str = "(){}[];?.,";
const OPERATOR_CHARACTERS: &str = "+-*/%&|^=!~:<>.";
const OPERATORS: &[&str] = &[
    "!..=", "!..", "..=", "..",
    "++=", "+.=", "+=", "-.=", "-=", "**.=", "**=", "*.=", "*=", "/.=", "/=", "%.=", "%=",
    "&&=", "||=", "^^=", "=!", "&=", "|=", "^=", "=~", "<<=", ">>=",
    "==", "!=", "<=", ">=", "=",
    "++", "+.", "+", "-.", "-", "**.", "**", "*.", "*", "/.", "/", "%.", "%",
    "&&", "||", "^^", "!", "&", "|", "^", "~", "<<", ">>",
    "<", ">",
];

pub struct Lexer<'l> {
    file_name: &'l str,
    source_code: &'l str,
    index: usize,
    line_index: usize,
    line_number: usize,
}

impl<'l> Lexer<'l> {
    
    pub fn new (file: &'l str, src: &'l str) -> Lexer<'l> {
        return Lexer {
            file_name: file,
            source_code: src,
            index: 0,
            line_index: 1,
            line_number: 1
        };
    }

    pub fn set_file(&mut self, file: &'l str) {
        self.file_name = file;
    }
    
    pub fn set_source_code(&mut self, src: &'l str) {
        self.source_code = src;
        self.index = 0;
        self.line_index = 1;
        self.line_number = 1;
    }

    fn consume(&mut self, num: usize) {
        self.index += num;
    }
    
    fn peek(&self) -> Option<char> {
        return self.source_code.chars().nth(self.index);
    }
    
    fn peek_at(&self, index: usize) -> Option<char> {
        return self.source_code.chars().nth(index);
    }

    fn peek_after(&self, after: usize) -> Option<char> {
        return self.source_code.chars().nth(self.index + after);
    }
    
    pub fn tokenize(&mut self) -> Vec<Tkn> {
        let mut tokens: Vec<Tkn> = Vec::new();

        while let Some(chr) = self.peek() {
            let token: TknType;
            let index = self.line_index;
            let line = self.line_number;

            match chr {
                '/' => {
                    match self.peek_after(1) {
                        Some('/') => {
                            let mut after = 2;
                            while let Some(chr) = self.peek_after(after) {
                                if chr == '\n' || chr == '\r' {
                                    break;
                                }
                                after += 1;
                            }
                            println!("comment: {}", self.source_code.slice(self.index..self.index+after));
                            self.consume(after);
                            self.line_index += after;
                            continue;
                        },
                        Some(',') => {
                            let mut after = 2;
                            let mut count = 1;
                            
                            while 
                                let (Some(chr1), Some(chr2)) = 
                                (self.peek_after(after), self.peek_after(after + 1)) 
                            {
                                if chr1 == '/' && chr2 == ',' {
                                    count += 1;
                                    after += 2;
                                } else if chr1 == ',' && chr2 == '/' {
                                    count -= 1;
                                    after += 2;
                                } else if chr1 == '\n' || chr1 == '\r' {
                                    after += 1;
                                    self.line_number += 1;
                                } else {
                                    after += 1;
                                }

                                if count == 0 {
                                    break;
                                }
                            }
                            println!("comment: {}", self.source_code.slice(self.index..self.index+after));
                            self.consume(after);
                            self.line_index += after;
                            continue;
                        },
                        _ => {
                            token = self.get_multi_character_token();
                        }
                    }
                },
                ',' => {
                    token = TknType::Comma;
                    self.consume(1);
                    self.line_index += 1;
                }
                '$' => {
                    token = TknType::Dollar;
                    self.consume(1);
                    self.line_index += 1;
                },
                '.' => {
                    token = TknType::Dot;
                    self.consume(1);
                    self.line_index += 1;
                },
                ':' => {
                    if let Some(':') = self.peek_after(1) {
                        token = TknType::ColonColon;
                        self.consume(2);
                        self.line_index += 2;
                    } else {
                        token = TknType::Colon;
                        self.consume(1);
                        self.line_index += 1;
                    }
                },
                ';' => {
                    token = TknType::Semicolon;
                    self.consume(1);
                    self.line_index += 1;
                },
                '(' => {
                    token = TknType::OpenParen;
                    self.consume(1);
                    self.line_index += 1;
                },
                ')' => {
                    token = TknType::CloseParen;
                    self.consume(1);
                    self.line_index += 1;
                },
                '{' => {
                    token = TknType::OpenCurlyBrace;
                    self.consume(1);
                    self.line_index += 1;
                },
                '}' => {
                    token = TknType::CloseCurlyBrace;
                    self.consume(1);
                    self.line_index += 1;
                },
                '[' => {
                    token = TknType::OpenSquareBracket;
                    self.consume(1);
                    self.line_index += 1;
                },
                ']' => {
                    token = TknType::CloseSquareBracket;
                    self.consume(1);
                    self.line_index += 1;
                },
                '\n' | '\r' => {
                    self.consume(1);
                    self.line_index = 1;
                    self.line_number += 1;
                    
                    continue;
                },
                ' ' => {
                    let spaces = self.count_spaces();
                    self.consume(spaces);
                    self.line_index += spaces;

                    continue;
                },
                _ => {
                    token = self.get_multi_character_token();
                }
            }
            
            tokens.push(Tkn::new(
                token, 
                self.file_name.to_string(), 
                index, 
                line
            ));
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
        let mut index: usize = self.index;

        while let Some(' ') = self.peek_at(index) {
            spaces += 1;
            index += 1;
        }

        return spaces;
    }

    fn get_multi_character_token(&mut self) -> TknType {
        let mut multi_character: &'l str;
        let start_index: usize = self.index;
        let mut end_index: usize = self.index;

        // Before trying to parse a float literal, we must check
        // that it is instead an integer literal, or else
        // integers will get confused as floats
        while let Some(chr) = self.peek_at(end_index) {
            if !chr.is_alphanumeric() {
                break;
            }
            end_index += 1;
        }

        multi_character = self.source_code.slice(start_index..end_index);
        if let Some(token) = self.get_integer_literal(multi_character) {
            return token;
        }

        // To avoid conflicts regarding operators featuring a . (such as "/." and "..")
        // and floating point numbers (i.e. 6.28), we check for floating point numbers
        // first.  If we don't, 6.28 will be parsed as
        // { IntegerLiteral(6), Dot, IntegerLiteral(28) } 
        // instead of { FloatLiteral(6.28) }
        let mut contains_dot = false;
        while let Some(chr) = self.peek_at(end_index) {
            if !chr.is_alphanumeric() && chr != '.' {
                break;
            }
            if chr == '.' {
                if contains_dot {
                    break;
                }
                contains_dot = true;
            }
            end_index += 1;
        }

        multi_character = self.source_code.slice(start_index..end_index);
        if let Some(token) = self.get_float_literal(multi_character) {
            return token;
        }

        // We have to reset end index back to its initial state to recheck
        // other possible multicharacter tokens in the event that it is not a float
        end_index = self.index;
        while let Some(chr) = self.peek_at(end_index) {
            if is_invalid_character(chr) {
                break;
            }
            end_index += 1;
        }

        multi_character = self.source_code.slice(start_index..end_index);
        match multi_character {
            "let" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::Let);
            },
            "return" => {
                self.consume(6);
                self.line_index += 6;
                return TknType::Keyword(Kwrd::Return);
            },
            "if" => {
                self.consume(2);
                self.line_index += 2;
                return TknType::Keyword(Kwrd::If);
            },
            "else" => {
                self.consume(4);
                self.line_index += 4;
                return TknType::Keyword(Kwrd::Else);
            },
            "for" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::For);
            },
            "while" => {
                self.consume(5);
                self.line_index += 5;
                return TknType::Keyword(Kwrd::While);
            },
            "loop" => {
                self.consume(4);
                self.line_index += 4;
                return TknType::Keyword(Kwrd::Loop);
            },
            "mut" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::Mutable);
            },
            "im" => {
                self.consume(2);
                self.line_index += 2;
                return TknType::Keyword(Kwrd::InteriorMutable);
            },
            "rec" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::Recursive);
            },
            "oxy" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::Oxidize);
            },
            "unsafe" => {
                self.consume(6);
                self.line_index += 6;
                return TknType::Keyword(Kwrd::Unsafe);
            },
            "fn" => {
                self.consume(multi_character.len());
                self.line_index += multi_character.len();
                return TknType::Keyword(Kwrd::Function);
            },
            "accessor" => {
                self.consume(8);
                self.line_index += 8;
                return TknType::Keyword(Kwrd::Accessor);
            },
            "enclave" => {
                self.consume(7);
                self.line_index += 7;
                return TknType::Keyword(Kwrd::Enclave);
            },
            "exclave" => {
                self.consume(7);
                self.line_index += 7;
                return TknType::Keyword(Kwrd::Exclave);
            },
            "struct" => {
                self.consume(multi_character.len());
                self.line_index += multi_character.len();
                return TknType::Keyword(Kwrd::Struct);
            },
            "namespace" => {
                self.consume(9);
                self.line_index += 9;
                return TknType::Keyword(Kwrd::Namespace);
            },
            "alias" => {
                self.consume(5);
                self.line_index += 5;
                return TknType::Keyword(Kwrd::Alias);
            },
            "true" => {
                self.consume(4);
                self.line_index += 4;
                return TknType::BooleanLiteral(true);
            },
            "false" => {
                self.consume(5);
                self.line_index += 5;
                return TknType::BooleanLiteral(false);
            },
            "pub" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::Public);
            },
            "prv" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::Private);
            },
            "pkg" => {
                self.consume(3);
                self.line_index += 3;
                return TknType::Keyword(Kwrd::Package);
            },
            "prefix" => {
                self.consume(6);
                self.line_index += 6;
                return TknType::Keyword(Kwrd::Prefix);
            },
            "infix" => {
                self.consume(5);
                self.line_index += 5;
                return TknType::Keyword(Kwrd::Infix);
            },
            "postfix" => {
                self.consume(7);
                self.line_index += 7;
                return TknType::Keyword(Kwrd::Postfix);
            },
            _ => {
                return self.get_operation()
                    .or_else(|| self.get_char_literal())
                    .or_else(|| self.get_string_literal())
                    .or_else(|| self.get_type(multi_character))
                    .or_else(|| self.get_identifier(multi_character))
                    .unwrap_or_else(|| self.get_invalid(multi_character));
            }
        };
    }

    fn get_operation(&mut self) -> Option<TknType> {
        if let Some(chr) = self.peek() {
            if !OPERATOR_CHARACTERS.contains(chr) {
                return None;
            }
        } else {
            return None;
        }
        
        for i in 0..OPERATORS.len() {
            if self.source_code.len() - self.index < OPERATORS[i].len() {
                continue;
            }
            
            let operator = self.source_code.slice(self.index..self.index + OPERATORS[i].len());
            if operator == OPERATORS[i] {
                self.consume(OPERATORS[i].len());
                self.line_index += OPERATORS[i].len();
                return match operator {
                    "!..=" => Some(TknType::Operation(Op::BangRangeEquals)), 
                    "!.." => Some(TknType::Operation(Op::BangRange)), 
                    "..=" => Some(TknType::Operation(Op::RangeEquals)), 
                    ".." => Some(TknType::Either(
                        Box::new(TknType::DiscardMany),
                        Box::new(TknType::Operation(Op::Range))
                    )), 
                    "==" => Some(TknType::Operation(Op::Equals)), 
                    "!=" => Some(TknType::Operation(Op::NotEquals)), 
                    "<=" => Some(TknType::Operation(Op::LessThanEqualTo)),
                    ">=" => Some(TknType::Operation(Op::GreaterThanEqualTo)), 
                    "<" => Some(TknType::Either(
                      Box::new(TknType::Operation(Op::LessThan)),
                      Box::new(TknType::OpenAngularBracket)
                    )), 
                    ">" => Some(TknType::Either(
                      Box::new(TknType::Operation(Op::GreaterThan)),
                      Box::new(TknType::CloseAngularBracket)
                    )), 
                    "++=" => Some(TknType::Operation(Op::ConcatEquals)), 
                    "+.=" => Some(TknType::Operation(Op::PlusFloatEquals)),
                    "+=" => Some(TknType::Operation(Op::PlusEquals)),
                    "-.=" => Some(TknType::Operation(Op::MinusFloatEquals)),
                    "-=" => Some(TknType::Operation(Op::MinusEquals)), 
                    "**.=" => Some(TknType::Operation(Op::ExponentFloatEquals)),
                    "**=" => Some(TknType::Operation(Op::ExponentEquals)), 
                    "*.=" => Some(TknType::Operation(Op::MultiplyFloatEquals)),
                    "*=" => Some(TknType::Operation(Op::MultiplyEquals)), 
                    "/.=" => Some(TknType::Operation(Op::DivideFloatEquals)), 
                    "/=" => Some(TknType::Operation(Op::DivideEquals)),
                    "%.=" => Some(TknType::Operation(Op::ModuloFloatEquals)),
                    "%=" => Some(TknType::Operation(Op::ModuloEquals)),
                    "+." => Some(TknType::Operation(Op::PlusFloat)),
                    "+" => Some(TknType::Operation(Op::Plus)), 
                    "-." => Some(TknType::Operation(Op::MinusFloat)),
                    "-" => Some(TknType::Operation(Op::Minus)),
                    "**." => Some(TknType::Operation(Op::ExponentFloat)), 
                    "**" => Some(TknType::Operation(Op::Exponent)), 
                    "*." => Some(TknType::Operation(Op::MultiplyFloat)),
                    "*" => Some(TknType::Operation(Op::Multiply)), 
                    "/." => Some(TknType::Operation(Op::DivideFloat)), 
                    "/" => Some(TknType::Operation(Op::Divide)), 
                    "%." => Some(TknType::Operation(Op::ModuloFloat)),
                    "%" => Some(TknType::Operation(Op::Modulo)),
                    "&&" => Some(TknType::Operation(Op::LogicAnd)), 
                    "||" => Some(TknType::Operation(Op::LogicOr)), 
                    "!" => Some(TknType::Operation(Op::LogicNot)),
                    "&" => Some(TknType::Either(
                        Box::new(TknType::Operation(Op::BitwiseAnd)),
                        Box::new(TknType::Borrow)
                    )), 
                    "|" => Some(TknType::Operation(Op::BitwiseOr)), 
                    "^" => Some(TknType::Operation(Op::BitwiseXor)),
                    "~" => Some(TknType::Operation(Op::BitwiseNegate)),
                    "<<" => Some(TknType::Operation(Op::BitwiseShiftLeft)),
                    ">>" => Some(TknType::Operation(Op::BitwiseShiftRight)), 
                    "=>" => Some(TknType::Operation(Op::Arrow)),
                    "=" => Some(TknType::Operation(Op::Assign)),
                    _ => unreachable!()
                }
            }
        }
        
        return None;
    }

    fn get_integer_literal(&mut self, multi_character: &str) -> Option<TknType> {
        match multi_character.parse() {
            Ok(int) => {
                self.consume(multi_character.len());
                self.line_index += multi_character.len();
                return Some(TknType::IntegerLiteral { 
                    int, 
                    len: multi_character.len() 
                });
            },
            Err(_) => return None
        }
    }

    fn get_float_literal(&mut self, multi_character: &str) -> Option<TknType> {
        match multi_character.parse() {
            Ok(float) => {
                self.consume(multi_character.len());
                self.line_index += multi_character.len();
                return Some(TknType::FloatLiteral {
                    float,
                    len: multi_character.len()
                });
            },
            Err(_) => return None
        }
    }

    fn get_char_literal(&mut self) -> Option<TknType> {
        
        match self.peek() {
            Some('\'') => (),
            _ => return None
        }

        let mut index = self.index + 1;
        while let Some(chr) = self.peek_at(index) {
            if chr == '\\' {
                index += 2;
            } else if chr == '\'' {
                break;
            } else {
                index += 1;
            }
        }

        let len = index - self.index + 1;
        let multi_character = self.source_code.slice(self.index + 1..index);
        match multi_character.parse() {
            Ok(chr) => {
                self.consume(len);
                self.line_index += len;
                return Some(TknType::CharLiteral(chr));
            },
            Err(_) => return None
        }
    }

    fn get_string_literal(&mut self) -> Option<TknType> {
        match self.peek() {
            Some('\"') => (),
            _ => return None
        }

        let mut index = self.index + 1;
        while let Some(chr) = self.peek_at(index) {
            if chr == '\\' {
                index += 2;
                continue;
            } else if chr == '\"' {
                let len = index - self.index + 1;
                let multi_character = self.source_code.slice(self.index + 1..index);
                self.consume(len);
                self.line_index += len;
                return Some(TknType::StringLiteral(multi_character.to_string()));
            } else {
                index += 1;
                continue;
            }
        }

        return None;
    }


    fn get_type(&mut self, multi_character: &str) -> Option<TknType> {
        match multi_character {
            "i8" => {
                self.consume(2);
                self.line_index += 2;
                return Some(TknType::Type(Type::I8));
            },
            "i16" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::I16));
            },
            "i32" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::I32));
            },
            "i64" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::I64));
            },
            "i128" => {
                self.consume(4);
                self.line_index += 4;
                return Some(TknType::Type(Type::I128));
            },
            "u8" => {
                self.consume(2);
                self.line_index += 2;
                return Some(TknType::Type(Type::U8));
            },
            "u16" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::U16));
            },
            "u32" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::U32));
            },
            "u64" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::U64));
            },
            "u128" => {
                self.consume(4);
                self.line_index += 4;
                return Some(TknType::Type(Type::U128));
            },
            "f32" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::F32));
            },
            "f64" => {
                self.consume(3);
                self.line_index += 3;
                return Some(TknType::Type(Type::F64));
            },
            "char" => {
                self.consume(4);
                self.line_index += 4;
                return Some(TknType::Type(Type::Char));
            },
            "bool" => {
                self.consume(4);
                self.line_index += 4;
                return Some(TknType::Type(Type::Bool));
            },
            _ => (),
        }
        
        return None;
    }

    fn get_identifier(&mut self, multi_character: &str) -> Option<TknType> {
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
        self.line_index += break_index;

        return Some(TknType::Identifier(multi_character.to_string()));
    }

    fn get_invalid(&mut self, multi_character: &str) -> TknType {
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
