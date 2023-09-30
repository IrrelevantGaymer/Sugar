use crate::string_utils::StringUtils;
use crate::token::{Kwrd, Op, Type, Tkn, TknType};

const OPERATOR_CHARACTERS: &str = "+-*/%&|^=!~:<>";

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
                    token = TknType::NewLine;
                    new_line = true;
                    self.line_index = 1;
                    self.line_number += 1;
                    self.consume(1);
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
                    token = self.get_multi_character_token().clone();
                }
            }

            //tokens.push(Tkn::new(token, self.file_name.to_string(), index, line));
        }

        todo!();
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
            if !is_invalid_character(chr) {
                break;
            }
            end_index += 1;
        }

        multi_character = self.source_code.slice(start_index..=end_index);
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
                return self.get_integer_literal(multi_character)
                    .or_else(|| self.get_float_literal(multi_character))
                    .or_else(|| self.get_char_literal(multi_character))
                    .or_else(|| self.get_string_literal(multi_character))
                    .or_else(|| self.get_operation(multi_character))
                    .or_else(|| self.get_type(multi_character))
                    .or_else(|| self.get_identifier(multi_character))
                    .unwrap_or_else(|| self.get_invalid(multi_character));
            }
        };
    }

    fn get_integer_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        todo!();
    }

    fn get_float_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        todo!();
    }

    fn get_char_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        todo!();
    }

    fn get_string_literal(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        todo!();
    }

    fn get_operation(&mut self, multi_character: &str) -> Option<TknType<'a>> {
        todo!();
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

        if multi_character.chars().nth(0).unwrap().is_numeric() {
            return None;
        }

        while let Some(chr) = multi_character.chars().nth(break_index) {
            if chr == ':' || chr == '.' {
                break;
            }
            break_index += 1;
        }
        
        let multi_character = multi_character.slice(0..=break_index);
        self.consume(break_index);

        return Some(TknType::Identifier(String::from(multi_character)));
    }

    fn get_invalid(&mut self, multi_character: &str) -> TknType<'a> {
        self.consume(multi_character.len());
        return TknType::Invalid;
    }
}

fn is_invalid_character(chr: char) -> bool {
    return chr == ' ' || chr == '\n' || chr == '\r' || OPERATOR_CHARACTERS.contains(chr);
}
