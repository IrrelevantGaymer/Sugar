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

pub struct Lexer<'l> {
    file_name: &'l str,
    source_code: &'l str,
    index: usize,
    line_index: usize,
    line_number: usize,
}

impl<'l> Lexer<'l> {
    fn consume(&mut self, num: usize) {
        self.index += num;
    }
    
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

    pub fn peek(&self) -> Option<char> {
        return self.source_code.chars().nth(self.index);
    }

    pub fn peek_at(&self, index: usize) -> Option<char> {
        return self.source_code.chars().nth(index);
    }

    pub fn peek_after(&self, after: usize) -> Option<char> {
        return self.source_code.chars().nth(self.index + after);
    }
    
    pub fn tokenize(&mut self) -> Vec<Tkn> {
        let mut tokens: Vec<Tkn> = Vec::new();
        let mut new_line: bool = true;

        while let Some(chr) = self.source_code.chars().nth(self.index) {
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
                            /*
                             * do stuff 
                             */
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
                            let mut after = 1;

                            while let Some(chr) = self.peek_after(after) {
                                if is_invalid_character(chr) {
                                    break;
                                }
                                after += 1;
                            }

                            self.consume(after);
                            self.line_index += after;
                            token = TknType::Invalid;
                        }
                    }
                }
                '?' => {
                    token = TknType::Question;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                '$' => {
                    token = TknType::Dollar;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                '.' => {
                    token = TknType::Dot;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                ':' => {
                    if let Some(':') = self.source_code.chars().nth(self.index + 1) {
                        token = TknType::ColonColon;
                        new_line = false;
                        self.consume(2);
                        self.line_index += 2;
                    } else {
                        token = TknType::Colon;
                        new_line = false;
                        self.consume(1);
                        self.line_index += 1;
                    }
                },
                ';' => {
                    token = TknType::Semicolon;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                '(' => {
                    token = TknType::OpenParen;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                ')' => {
                    token = TknType::CloseParen;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                '{' => {
                    token = TknType::OpenCurlyBrace;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                '}' => {
                    token = TknType::CloseCurlyBrace;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                '[' => {
                    token = TknType::OpenSquareBracket;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                ']' => {
                    token = TknType::CloseSquareBracket;
                    new_line = false;
                    self.consume(1);
                    self.line_index += 1;
                },
                '\n' | '\r' => {
                    self.consume(1);
                    self.line_index = 1;
                    self.line_number += 1;
                    
                    if new_line {
                        continue;
                    }
                    
                    token = TknType::NewLine;
                    new_line = true;
                },
                ' ' => {
                    let spaces = self.count_spaces();
                    self.consume(spaces);
                    self.line_index += spaces;

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

        while let Some(' ') = self.source_code.chars().nth(index) {
            spaces += 1;
            index += 1;
        }

        return spaces;
    }

    fn get_multi_character_token(&mut self) -> TknType<'l>{
        let multi_character: &'l str;
        let start_index: usize = self.index;
        let mut end_index: usize = self.index;

        while let Some(chr) = self.source_code.chars().nth(end_index) {
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
            "fn" | "fun" | "function" => {
                self.consume(multi_character.len());
                self.line_index += multi_character.len();
                return TknType::Keyword(Kwrd::Function);
            },
            "accessor" => {
                self.consume(7);
                self.line_index += 7;
                return TknType::Keyword(Kwrd::Accessor);
            },
            "whitelist" => {
                self.consume(9);
                self.line_index += 9;
                return TknType::Keyword(Kwrd::Whitelist);
            },
            "blacklist" => {
                self.consume(9);
                self.line_index += 9;
                return TknType::Keyword(Kwrd::Blacklist);
            },
            "struct" | "class" => {
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
                    .or_else(|| self.get_integer_literal(multi_character))
                    .or_else(|| self.get_float_literal(multi_character))
                    .or_else(|| self.get_char_literal())
                    .or_else(|| self.get_string_literal())
                    .or_else(|| self.get_type(multi_character))
                    .or_else(|| self.get_identifier(multi_character))
                    .unwrap_or_else(|| self.get_invalid(multi_character));
            }
        };
    }

    fn get_operation(&mut self) -> Option<TknType<'l>> {
        if let Some(chr) = self.source_code.chars().nth(self.index) {
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
                    "%=" => Some(TknType::Operation(Op::ModuloEquals)),
                    "&&=" => Some(TknType::Operation(Op::LogicAndEquals)), 
                    "&=" => Some(TknType::Operation(Op::BitwiseAndEquals)), 
                    "||=" => Some(TknType::Operation(Op::LogicOrEquals)), 
                    "|=" => Some(TknType::Operation(Op::BitwiseOrEquals)), 
                    "^^=" => Some(TknType::Operation(Op::LogicXorEquals)), 
                    "^=" => Some(TknType::Operation(Op::BitwiseXorEquals)), 
                    "<<=" => Some(TknType::Operation(Op::BitwiseShiftLeftEquals)),
                    ">>=" => Some(TknType::Operation(Op::BitwiseShiftRightEquals)),
                    "=!" => Some(TknType::Operation(Op::EqualsNot)), 
                    "=~" => Some(TknType::Operation(Op::EqualsNegate)), 
                    "++" => Some(TknType::Operation(Op::PlusPlus)), 
                    "--" => Some(TknType::Operation(Op::MinusMinus)), 
                    "+" => Some(TknType::Operation(Op::Plus)), 
                    "-" => Some(TknType::Operation(Op::Minus)), 
                    "**" => Some(TknType::Operation(Op::Exponent)), 
                    "*" => Some(TknType::Operation(Op::Multiply)), 
                    "//" => Some(TknType::Operation(Op::FloatDivide)), 
                    "/" => Some(TknType::Operation(Op::IntDivide)), 
                    "%" => Some(TknType::Operation(Op::Modulo)),
                    "&&" => Some(TknType::Operation(Op::LogicAnd)), 
                    "||" => Some(TknType::Operation(Op::LogicOr)), 
                    "^^" => Some(TknType::Operation(Op::LogicXor)),
                    "!" => Some(TknType::Operation(Op::LogicNot)),
                    "&" => Some(TknType::Either(
                        &TknType::Operation(Op::BitwiseAnd),
                        &TknType::Borrow
                    )), 
                    "|" => Some(TknType::Operation(Op::BitwiseOr)), 
                    "^" => Some(TknType::Operation(Op::BitwiseXor)),
                    "~" => Some(TknType::Operation(Op::BitwiseNegate)),
                    "<<" => Some(TknType::Operation(Op::BitwiseShiftLeft)),
                    ">>" => Some(TknType::Operation(Op::BitwiseShiftRight)), 
                    "=" => Some(TknType::Operation(Op::Assign)),
                    "->" => Some(TknType::Operation(Op::Insert)),
                    _ => unreachable!()
                }
            }
        }
        
        return None;
    }

    fn get_integer_literal(&mut self, multi_character: &str) -> Option<TknType<'l>> {
        match multi_character.parse() {
            Ok(int) => {
                self.consume(multi_character.len());
                self.line_index += multi_character.len();
                return Some(TknType::IntegerLiteral(int));
            },
            Err(_) => return None
        }
    }

    fn get_float_literal(&mut self, multi_character: &str) -> Option<TknType<'l>> {
        match multi_character.parse() {
            Ok(float) => {
                self.consume(multi_character.len());
                self.line_index += multi_character.len();
                return Some(TknType::FloatLiteral(float));
            },
            Err(_) => return None
        }
    }

    fn get_char_literal(&mut self) -> Option<TknType<'l>> {
        
        match self.source_code.chars().nth(self.index) {
            Some('\'') => (),
            _ => return None
        }

        let mut index = self.index + 1;
        while let Some(chr) = self.source_code.chars().nth(index) {
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

    fn get_string_literal(&mut self) -> Option<TknType<'l>> {
        match self.source_code.chars().nth(self.index) {
            Some('\"') => (),
            _ => return None
        }

        let mut index = self.index + 1;
        while let Some(chr) = self.source_code.chars().nth(index) {
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


    fn get_type(&mut self, multi_character: &str) -> Option<TknType<'l>> {
        match multi_character.chars().nth(0) {
            Some('i') => {
                let size = multi_character.slice(1..);
                match size.parse() {
                    Ok(size) => {
                        self.consume(multi_character.len());
                        self.line_index += multi_character.len();
                        return Some(TknType::Type(Type::Integer(size)))
                    },
                    Err(_) => return None
                }
            },
            Some('u') => {
                let size = multi_character.slice(1..);
                match size.parse() {
                    Ok(size) => {
                        self.consume(multi_character.len());
                        self.line_index += multi_character.len();
                        return Some(TknType::Type(Type::UnsignedInteger(size)))
                    },
                    Err(_) => return None
                }
            },
            Some('f') => {
                let size = multi_character.slice(1..);
                match size.parse() {
                    Ok(size) => {
                        self.consume(multi_character.len());
                        self.line_index += multi_character.len();
                        return Some(TknType::Type(Type::Float(size)))
                    },
                    Err(_) => return None
                }
            },
            _ => ()
        }

        match multi_character {
            "char" => {
                self.consume(4);
                self.line_index += 4;
                return Some(TknType::Type(Type::Character));
            },
            "bool" => {
                self.consume(4);
                self.line_index += 4;
                return Some(TknType::Type(Type::Boolean));
            },
            _ => (),
        }
        
        return None;
    }

    fn get_identifier(&mut self, multi_character: &str) -> Option<TknType<'l>> {
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

    fn get_invalid(&mut self, multi_character: &str) -> TknType<'l> {
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
