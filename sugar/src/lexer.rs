use crate::string_utils::StringUtils;
use crate::token::{Kwrd, Op, Tkn, TknType};
use std::{iter::Peekable, str::Chars};

const OPERATOR_CHARACTERS: &str = "+-*/%&|^=!~:<>";

pub struct Lexer<'a> {
    file_name: &'a str,
    source_code: &'a str,
    peek: usize,
    line_index: usize,
    line_number: usize,
}

impl<'a> Lexer<'a> {
    fn consume(&mut self, num: Option<usize>) {
        let num = num.unwrap_or(1);
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
        let mut new_line: bool = true;

        while let Some(chr) = self.source_code.chars().nth(self.peek) {
            let token: TknType;
            match chr {
                '?' => {
                    token = TknType::Question;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                '$' => {
                    token = TknType::Dollar;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                '(' => {
                    token = TknType::OpenParen;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                ')' => {
                    token = TknType::CloseParen;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                '{' => {
                    token = TknType::OpenCurlyBrace;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                '}' => {
                    token = TknType::CloseCurlyBrace;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                '[' => {
                    token = TknType::OpenSquareBracket;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                ']' => {
                    token = TknType::CloseSquareBracket;
                    new_line = false;
                    self.line_index += 1;
                    self.consume(None);
                },
                '\n' | '\r' => {
                    token = TknType::NewLine;
                    new_line = true;
                    self.line_index = 1;
                    self.line_number += 1;
                    self.consume(None);
                },
                ' ' => {
                    let spaces = self.count_spaces();
                    self.line_index += spaces;
                    self.consume(Some(spaces));

                    if new_line {
                        token = TknType::Spaces(spaces);
                        new_line = false;
                    } else {
                        continue;
                    }
                },
                _ => ()
            }
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

    fn get_multi_character_token(&mut self) {
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

        let token = match multi_character {
            "let" => TknType::Keyword(Kwrd::Let),
            "loop" => TknType::Keyword(Kwrd::Loop),
            "mut" => TknType::Keyword(Kwrd::Mutable),
            "oxy" => TknType::Keyword(Kwrd::Oxidize),
            "unsafe" => TknType::Keyword(Kwrd::Unsafe),
            "fn" | "fun" | "function" => TknType::Keyword(Kwrd::Function),
            "accessor" => TknType::Keyword(Kwrd::Accessor),
            "whitelist" => TknType::Keyword(Kwrd::Whitelist),
            "blacklist" => TknType::Keyword(Kwrd::Blacklist),
            "struct" | "class" => TknType::Keyword(Kwrd::Struct),
            "namespace" => TknType::Keyword(Kwrd::Namespace),
            "alias" => TknType::Keyword(Kwrd::Alias),
            "pub" => TknType::Keyword(Kwrd::Public),
            "prv" => TknType::Keyword(Kwrd::Private),
            "pkg" => TknType::Keyword(Kwrd::Package),  
            _ => self.get_integer_literal(multi_character)
                .or_else(|| self.get_float_literal(multi_character))
                .or_else(|| self.get_char_literal(multi_character))
                .or_else(|| self.get_string_literal(multi_character))
                .or_else(|| self.get_type(multi_character))
                .or_else(|| self.get_identifier(multi_character))
                .unwrap_or(TknType::Invalid)
        };
    }

    fn get_integer_literal<'b>(&mut self, multi_character: &'b str) -> Option<TknType<'b>> {
        todo!();
    }

    fn get_float_literal<'b>(&mut self, multi_character: &'b str) -> Option<TknType<'b>> {
        todo!();
    }

    fn get_char_literal<'b>(&mut self, multi_character: &'b str) -> Option<TknType<'b>> {
        todo!();
    }

    fn get_string_literal<'b>(&mut self, multi_character: &'b str) -> Option<TknType<'b>> {
        todo!();
    }

    fn get_type<'b>(&mut self, multi_character: &'b str) -> Option<TknType<'b>> {
        todo!();
    }

    fn get_identifier<'b>(&mut self, multi_character: &'b str) -> Option<TknType<'b>> {
        todo!();
    }
}

fn is_invalid_character(chr: char) -> bool {
    return chr == ' ' || chr == '\n' || chr == '\r' || OPERATOR_CHARACTERS.contains(chr);
}
