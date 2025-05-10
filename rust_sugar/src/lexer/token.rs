extern crate num;

use std::fmt::Display;

use crate::parser::expr::ExprType;

pub type Tkn = Token;
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token: TknType,
    pub file_name: String,
    pub line_index: usize,
    pub line_number: usize
}

impl Token {
    pub fn new (
        token: TknType, file_name: String, line_index: usize, line_number: usize
    ) -> Token {
        return Token{
            token, 
            file_name, 
            line_index, 
            line_number
        };
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} token in file {} on line {} index {}", 
            self.token, self.file_name, self.line_number, self.line_index
        )
    }
}

pub type TknType = TokenType;
#[derive(Clone, Debug)]
pub enum TokenType {
    Keyword(Kwrd),
    Type(Type),
    Identifier(String),
    Operation(Op),

    IntegerLiteral { int: i128, len: usize },
    FloatLiteral { float: f64, len: usize },
    CharLiteral(char),
    StringLiteral(String),
    BooleanLiteral(bool),

    Comma,
    Semicolon,

    Dollar,
    OpenParen,
    CloseParen,
    OpenCurlyBrace,
    CloseCurlyBrace,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenAngularBracket,
    CloseAngularBracket,

    ColonColon,
    Colon,
    Dot,
    Borrow,

    /// _
    DiscardSingle,
    /// ..
    DiscardMany,

    Either(Box<TknType>, Box<TknType>),

    EndOfFile,
    Invalid,
}

impl TokenType {
    pub fn len(&self) -> usize {
        match self {
            TokenType::Keyword(keyword)           => keyword.len(),
            TokenType::Type(sgr_type)             => sgr_type.len(),
            TokenType::Identifier(ident)          => ident.len(),
            TokenType::Operation(operator)        => operator.len(),
            TokenType::IntegerLiteral { len, .. } => *len,
            TokenType::FloatLiteral { len, .. }   => *len,
            TokenType::CharLiteral(_)             => 1,
            TokenType::StringLiteral(string)      => string.len() + 2,
            TokenType::BooleanLiteral(true)       => 4,
            TokenType::BooleanLiteral(false)      => 5,
            TokenType::Comma                      => 1,
            TokenType::Semicolon                  => 1,
            TokenType::Dollar                     => 1,
            TokenType::OpenParen                  => 1,
            TokenType::CloseParen                 => 1,
            TokenType::OpenCurlyBrace             => 1,
            TokenType::CloseCurlyBrace            => 1,
            TokenType::OpenSquareBracket          => 1,
            TokenType::CloseSquareBracket         => 1,
            TokenType::OpenAngularBracket         => 1,
            TokenType::CloseAngularBracket        => 1,
            TokenType::ColonColon                 => 2,
            TokenType::Colon                      => 1,
            TokenType::Dot                        => 1,
            TokenType::Borrow                     => 1,
            TokenType::DiscardSingle              => 1,
            TokenType::DiscardMany                => 2,
            TokenType::Either(token_type, _)      => token_type.len(),
            TokenType::EndOfFile                  => 1,
            TokenType::Invalid                    => 1,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Keyword(keyword) => write!(f, "keyword {keyword}"),
            TokenType::Type(sgr_type) => write!(f, "type {sgr_type}"),
            TokenType::Identifier(ident) => write!(f, "identifier {ident}"),
            TokenType::Operation(operator) => write!(f, "operator {operator}"),
            TokenType::IntegerLiteral {int, ..} => write!(f, "integer {int}"),
            TokenType::FloatLiteral {float, ..} => write!(f, "float {float}"),
            TokenType::CharLiteral(chr) => write!(f, "char {chr}"),
            TokenType::StringLiteral(string) => write!(f, "string \"{string}\""),
            TokenType::BooleanLiteral(bool) => write!(f, "keyword {bool}"),
            TokenType::Comma => write!(f, "comma ','"),
            TokenType::Semicolon => write!(f, "semicolon ';'"),
            TokenType::Dollar => write!(f, "dollar '$'"),
            TokenType::OpenParen => write!(f, "open parenthesis '('"),
            TokenType::CloseParen => write!(f, "close parenthesis ')'"),
            TokenType::OpenCurlyBrace => write!(f, "open curly brace '{{'"),
            TokenType::CloseCurlyBrace => write!(f, "close curly brace '}}'"),
            TokenType::OpenSquareBracket => write!(f, "open square bracket '['"),
            TokenType::CloseSquareBracket => write!(f, "close square bracket ']'"),
            TokenType::OpenAngularBracket => write!(f, "open angular bracket '<'"),
            TokenType::CloseAngularBracket => write!(f, "close angular bracket '>'"),
            TokenType::ColonColon => write!(f, "scope resolution operator '::'"),
            TokenType::Colon => write!(f, "colon ':'"),
            TokenType::Dot => write!(f, "dot '.'"),
            TokenType::Borrow => write!(f, "borrow '&'"),
            TokenType::DiscardSingle => write!(f, "single discard '_'"),
            TokenType::DiscardMany => write!(f, "many discard '..'"),
            TokenType::Either(left, ..) => write!(f, "{left}"),
            TokenType::EndOfFile => write!(f, "end of file"),
            TokenType::Invalid => write!(f, "invalid token"),
        }
    }
}

impl Eq for TknType {}

impl PartialEq for TknType {
    fn eq (&self, other: &Self) -> bool {
        if let TknType::Either(left, right) = self {
            if let TknType::Either(other_left, other_right) = other {
                return left == other_left
                    || left == other_right
                    || right == other_left
                    || right == other_right;
            }
            return **left == *other
                || **right == *other;
        }

        if let TknType::Either(other_left, other_right) = other {
            return *self == **other_left
                || *self == **other_right;
        }
        
        return match (self, other) {
            (TknType::Keyword(left), TknType::Keyword(right)) => left == right,
            (TknType::Type(left), TknType::Type(right)) => left == right,
            (TknType::Identifier(left), TknType::Identifier(right)) => left == right,
            (TknType::Operation(left), TknType::Operation(right)) => left == right,
            (
                TknType::IntegerLiteral { int: left, .. }, 
                TknType::IntegerLiteral { int: right, .. }
            ) => left == right,
            (
                TknType::FloatLiteral { float: left, .. }, 
                TknType::FloatLiteral { float: right, .. }
            ) => left == right,
            (TknType::CharLiteral(left), TknType::CharLiteral(right)) => left == right,
            (TknType::StringLiteral(left), TknType::StringLiteral(right)) => left == right,
            (TknType::Semicolon, TknType::Semicolon) => true,
            (TknType::Dollar,TknType::Dollar) => true,
            (TknType::OpenParen,TknType::OpenParen) => true,
            (TknType::CloseParen,TknType::CloseParen) => true,
            (TknType::OpenCurlyBrace,TknType::OpenCurlyBrace) => true,
            (TknType::CloseCurlyBrace,TknType::CloseCurlyBrace) => true,
            (TknType::OpenSquareBracket,TknType::OpenSquareBracket) => true,
            (TknType::CloseSquareBracket,TknType::CloseSquareBracket) => true,
            (TknType::OpenAngularBracket,TknType::OpenAngularBracket) => true,
            (TknType::CloseAngularBracket,TknType::CloseAngularBracket) => true,
            (TknType::ColonColon,TknType::ColonColon) => true,
            (TknType::Colon,TknType::Colon) => true,
            (TknType::Dot,TknType::Dot) => true,
            (TknType::Comma,TknType::Comma) => true,
            (TknType::Borrow,TknType::Borrow) => true,
            (TknType::EndOfFile,TknType::EndOfFile) => true,
            (TknType::Invalid,TknType::Invalid) => true,

            _ =>  false

        }

    }
}

impl std::hash::Hash for TknType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

pub type Kwrd = Keyword;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keyword {
    /// let
    Let,
    /// return
    Return,

    /// for
    For,
    /// while
    While,
    /// loop
    Loop,
    /// if
    If,
    /// else
    Else,

    /// mut
    Mutable,
    /// im
    InteriorMutable,
    /// rec
    Recursive,
    /// oxy
    Oxidize,
    /// unsafe
    Unsafe,
    
    /// fn
    Function,
    /// accessor
    Accessor,
    /// enclave
    Enclave,
    /// exclave
    Exclave,
    /// struct
    Struct,
    
    /// prefix
    Prefix,
    /// infix
    Infix,
    /// postfix
    Postfix,

    /// namespace
    Namespace,
    /// alias
    Alias,

    /// pub
    Public,
    /// prv
    Private,
    /// pkg
    Package,
}

impl Kwrd {
    pub fn len(&self) -> usize {
        match self {
            Keyword::Let             => 3,
            Keyword::Return          => 6,
            Keyword::For             => 3,
            Keyword::While           => 5,
            Keyword::Loop            => 4,
            Keyword::If              => 2,
            Keyword::Else            => 4,
            Keyword::Mutable         => 3,
            Keyword::InteriorMutable => 2,
            Keyword::Recursive       => 3,
            Keyword::Oxidize         => 3,
            Keyword::Unsafe          => 6,
            Keyword::Function        => 2,
            Keyword::Accessor        => 8,
            Keyword::Enclave         => 7,
            Keyword::Exclave         => 7,
            Keyword::Struct          => 6,
            Keyword::Prefix          => 6,
            Keyword::Infix           => 5,
            Keyword::Postfix         => 7,
            Keyword::Namespace       => 9,
            Keyword::Alias           => 5,
            Keyword::Public          => 3,
            Keyword::Private         => 3,
            Keyword::Package         => 3,
        }
    }
}

impl Display for Kwrd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Keyword::Let => "let",
            Keyword::Return => "return",
            Keyword::For => "for",
            Keyword::While => "while",
            Keyword::Loop => "loop",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::Mutable => "mut",
            Keyword::InteriorMutable => "im",
            Keyword::Recursive => "rec",
            Keyword::Oxidize => "oxy",
            Keyword::Unsafe => "unsafe",
            Keyword::Function => "fn",
            Keyword::Accessor => "accessor",
            Keyword::Enclave => "enclave",
            Keyword::Exclave => "exclave",
            Keyword::Struct => "struct",
            Keyword::Prefix => "prefix",
            Keyword::Infix => "infix",
            Keyword::Postfix => "postfix",
            Keyword::Namespace => "namespace",
            Keyword::Alias => "alias",
            Keyword::Public => "pub",
            Keyword::Private => "prv",
            Keyword::Package => "pkg",
        })
    }
}

pub type Op = Operator;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    /// "!..=""
    BangRangeEquals, 
    /// "!.."
    BangRange,
    /// ..=
    RangeEquals, 
    /// ..
    Range, 

    /// ==
    Equals, 
    /// "!="
    NotEquals, 
    /// <=
    LessThanEqualTo, 
    /// >=
    GreaterThanEqualTo, 
    /// <
    LessThan, 
    /// >
    GreaterThan, 

    /// ++=
    ConcatEquals,
    /// +.=
    PlusFloatEquals,
    /// +=
    PlusEquals,
    /// -.=
    MinusFloatEquals,
    /// -=
    MinusEquals,
    /// **.=
    ExponentFloatEquals,
    /// **=
    ExponentEquals,
    /// *.=
    MultiplyFloatEquals,
    /// *=
    MultiplyEquals,
    /// /.=
    DivideFloatEquals,
    /// /=
    DivideEquals, 
    /// %.=
    ModuloFloatEquals,
    /// %=
    ModuloEquals,
    /// &&=
    LogicAndEquals,
    /// ||=
    LogicOrEquals,
    /// &=
    BitwiseAndEquals,
    /// "!="
    BitwiseOrEquals,
    /// ^=
    BitwiseXorEquals,
    /// <<=
    BitwiseShiftLeftEquals,
    /// >>=
    BitwiseShiftRightEquals,

    /// ++
    PlusPlus,
    /// +.
    PlusFloat,
    /// +
    Plus,
    /// -.
    MinusFloat,
    /// -
    Minus,
    /// **.
    ExponentFloat,
    /// **
    Exponent,
    /// *.
    MultiplyFloat,
    /// *
    Multiply,
    /// /.
    DivideFloat,
    /// /
    Divide,
    /// %.
    ModuloFloat,
    /// %
    Modulo,
    /// &&
    LogicAnd,
    /// ||
    LogicOr,
    /// "!
    LogicNot,
    /// &
    BitwiseAnd,
    /// |
    BitwiseOr,
    /// ^
    BitwiseXor,
    /// ~
    BitwiseNegate,
    /// <<
    BitwiseShiftLeft,
    /// >>
    BitwiseShiftRight,
    
    /// =>
    Arrow,

    /// =
    Assign,
}

impl Op {
    pub fn len(&self) -> usize {
        match self {
            Op::BangRangeEquals         => 4,
            Op::BangRange               => 3,
            Op::RangeEquals             => 3,
            Op::Range                   => 2,
            Op::Equals                  => 2,
            Op::NotEquals               => 2,
            Op::LessThanEqualTo         => 2,
            Op::GreaterThanEqualTo      => 2,
            Op::LessThan                => 1,
            Op::GreaterThan             => 1,
            Op::ConcatEquals            => 3,
            Op::PlusEquals              => 2,
            Op::PlusFloatEquals         => 3,
            Op::MinusEquals             => 2,
            Op::MinusFloatEquals        => 3,
            Op::ExponentEquals          => 3,
            Op::ExponentFloatEquals     => 4,
            Op::MultiplyEquals          => 2,
            Op::MultiplyFloatEquals     => 3,
            Op::DivideEquals            => 2,
            Op::DivideFloatEquals       => 3,
            Op::ModuloEquals            => 2,
            Op::ModuloFloatEquals       => 3,
            Op::LogicAndEquals          => 3,
            Op::LogicOrEquals           => 3,
            Op::BitwiseAndEquals        => 2,
            Op::BitwiseOrEquals         => 2,
            Op::BitwiseXorEquals        => 2,
            Op::BitwiseShiftLeftEquals  => 3,
            Op::BitwiseShiftRightEquals => 3,
            Op::PlusPlus                => 2,
            Op::Plus                    => 1,
            Op::PlusFloat               => 2,
            Op::Minus                   => 1,
            Op::MinusFloat              => 2,
            Op::Exponent                => 1,
            Op::ExponentFloat           => 2,
            Op::Multiply                => 1,
            Op::MultiplyFloat           => 2,
            Op::Divide                  => 1,
            Op::DivideFloat             => 2,
            Op::Modulo                  => 1,
            Op::ModuloFloat             => 2,
            Op::LogicAnd                => 2,
            Op::LogicOr                 => 2,
            Op::LogicNot                => 1,
            Op::BitwiseAnd              => 1,
            Op::BitwiseOr               => 1,
            Op::BitwiseXor              => 1,
            Op::BitwiseNegate           => 1,
            Op::BitwiseShiftLeft        => 2,
            Op::BitwiseShiftRight       => 2,
            Op::Arrow                   => 2,
            Op::Assign                  => 1,
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Op::BangRangeEquals => "!..=",
            Op::BangRange => "!..",
            Op::RangeEquals => "..=",
            Op::Range => "..",
            Op::Equals => "==",
            Op::NotEquals => "!=",
            Op::LessThanEqualTo => "<=",
            Op::GreaterThanEqualTo => ">=",
            Op::LessThan => "<",
            Op::GreaterThan => ">",
            Op::ConcatEquals => "++=",
            Op::PlusFloatEquals => "+.=",
            Op::PlusEquals => "+=",
            Op::MinusFloatEquals => "-.=",
            Op::MinusEquals => "-=",
            Op::ExponentFloatEquals => "**.=",
            Op::ExponentEquals => "**=",
            Op::MultiplyFloatEquals => "*.=",
            Op::MultiplyEquals => "*=",
            Op::DivideFloatEquals => "/.=",
            Op::DivideEquals => "/=",
            Op::ModuloFloatEquals => "%.=",
            Op::ModuloEquals => "%=",
            Op::LogicAndEquals => "&&=",
            Op::LogicOrEquals => "||=",
            Op::BitwiseAndEquals => "&=",
            Op::BitwiseOrEquals => "|=",
            Op::BitwiseXorEquals => "^=",
            Op::BitwiseShiftLeftEquals => "<<=",
            Op::BitwiseShiftRightEquals => ">>=",
            Op::PlusPlus => "++",
            Op::PlusFloat => "+.",
            Op::Plus => "+",
            Op::MinusFloat => "-.",
            Op::Minus => "-",
            Op::ExponentFloat => "**.",
            Op::Exponent => "**",
            Op::MultiplyFloat => "*.",
            Op::Multiply => "*",
            Op::DivideFloat => "/.",
            Op::Divide => "/",
            Op::ModuloFloat => "%.",
            Op::Modulo => "%",
            Op::LogicAnd => "&&",
            Op::LogicOr => "||",
            Op::LogicNot => "!",
            Op::BitwiseAnd => "&",
            Op::BitwiseOr => "|",
            Op::BitwiseXor => "^",
            Op::BitwiseNegate => "~",
            Op::BitwiseShiftLeft => "<<",
            Op::BitwiseShiftRight => ">>",
            Op::Arrow => "=>",
            Op::Assign => "=",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128, 
    F32, F64, 

    /// char
    Char,
    /// bool
    Bool,
}

impl Type {
    pub fn len(&self) -> usize {
        match self {
            Type::I8   => 2,
            Type::I16  => 3,
            Type::I32  => 3,
            Type::I64  => 3,
            Type::I128 => 4,
            Type::U8   => 2,
            Type::U16  => 3,
            Type::U32  => 3,
            Type::U64  => 3,
            Type::U128 => 4,
            Type::F32  => 3,
            Type::F64  => 3,
            Type::Char => 4,
            Type::Bool => 4,
        }
    }
    
    pub fn to_expr_type(&self) -> ExprType {
        match self {
            Self::I8 => ExprType::I8, 
            Self::I16 => ExprType::I16, 
            Self::I32 => ExprType::I32, 
            Self::I64 => ExprType::I64, 
            Self::I128 => ExprType::I128,
            Self::U8 => ExprType::U8, 
            Self::U16 => ExprType::U16, 
            Self::U32 => ExprType::U32, 
            Self::U64 => ExprType::U64, 
            Self::U128 => ExprType::U128, 
            Self::F32 => ExprType::F32, 
            Self::F64 => ExprType::F64, 
            Self::Char => ExprType::Char,
            Self::Bool => ExprType::Bool,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I8            => write!(f, "i8"),
            Self::I16           => write!(f, "i16"),
            Self::I32           => write!(f, "i32"),
            Self::I64           => write!(f, "i64"),
            Self::I128          => write!(f, "i128"),
            Self::U8            => write!(f, "u8"),
            Self::U16           => write!(f, "u16"),
            Self::U32           => write!(f, "u32"),
            Self::U64           => write!(f, "u64"),
            Self::U128          => write!(f, "u128"),
            Self::F32           => write!(f, "f32"),
            Self::F64           => write!(f, "f64"),
            Self::Char          => write!(f, "char"),
            Self::Bool          => write!(f, "bool"),
        }
    }
}
