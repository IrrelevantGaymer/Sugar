extern crate num;

use std::fmt::Display;

use crate::parser::expr::ExprType;

pub type Tkn<'t> = Token<'t>;
#[derive(Clone, Debug)]
pub struct Token<'t> {
    pub token: TknType<'t>,
    pub file_name: String,
    pub line_index: usize,
    pub line_number: usize
}

impl<'t> Token<'t> {
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

impl<'t> Display for Token<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} token in file {} on line {} index {}", 
            self.token, self.file_name, self.line_number, self.line_index
        )
    }
}

pub type TknType<'t> = TokenType<'t>;
#[derive(Clone, Debug)]
pub enum TokenType<'t> {
    Keyword(Kwrd),
    Type(Type),
    Identifier(String),
    Operation(Op),

    IntegerLiteral(i128),
    FloatLiteral(f64),
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

    Either(&'t TknType<'t>, &'t TknType<'t>),

    EndOfFile,
    Invalid,
}

impl<'t> Display for TokenType<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'t> Eq for TknType<'t> {}

impl<'t> PartialEq for TknType<'t> {
    fn eq (&self, other: &Self) -> bool {
        if let TknType::Either(left, right) = self {
            if let TknType::Either(other_left, other_right) = other {
                return left == other_left
                    || left == other_right
                    || right == other_left
                    || right == other_right;
            }
            return *left == other
                || *right == other;
        }

        if let TknType::Either(other_left, other_right) = other {
            return self == *other_left
                || self == *other_right;
        }
        
        return match (self, other) {
            (TknType::Keyword(left), TknType::Keyword(right)) => left == right,
            (TknType::Type(left), TknType::Type(right)) => left == right,
            (TknType::Identifier(left), TknType::Identifier(right)) => left == right,
            (TknType::Operation(left), TknType::Operation(right)) => left == right,
            (TknType::IntegerLiteral(left), TknType::IntegerLiteral(right)) => left == right,
            (TknType::FloatLiteral(left), TknType::FloatLiteral(right)) => left == right,
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

impl<'t> std::hash::Hash for TknType<'t> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

pub type Kwrd = Keyword;
#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Let,
    Return,

    For,
    While,
    Loop,
    If,
    Else,

    Mutable,
    Recursive,
    Oxidize,
    Unsafe,
    
    Function,
    Accessor,
    Whitelist,
    Blacklist,
    Struct,
    
    Prefix,
    Infix,
    Postfix,

    Namespace,
    Alias,

    Public,
    Private,
    Package,
}

pub type Op = Operator;
#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    BangRangeEquals, // !..=
    BangRange, // !..
    RangeEquals, // ..=
    Range, // ..

    Equals, // ==
    NotEquals, // !=
    LessThanEqualTo, // <=
    GreaterThanEqualTo, // >=
    LessThan, // <
    GreaterThan, // >

    ConcatEquals, // ++=
    PlusEquals, // +=
    MinusEquals, // -=
    ExponentEquals, // **=
    MultiplyEquals, // *=
    IntDivideEquals, // //=
    FloatDivideEquals, // /=
    ModuloEquals, // %=
    LogicAndEquals, // &&=
    LogicOrEquals, // ||=
    LogicXorEquals, // ^^=
    EqualsNot, // =!
    BitwiseAndEquals, // &=
    BitwiseOrEquals, // |=
    BitwiseXorEquals, // ^=
    EqualsNegate, // =~
    BitwiseShiftLeftEquals, // <<=
    BitwiseShiftRightEquals, // >>=

    PlusPlus, // ++
    Plus, // +
    MinusMinus, // --
    Minus, // -
    Exponent, // **
    Multiply, // *
    IntDivide, // //
    FloatDivide, // /
    Modulo, // %
    LogicAnd, // &&
    LogicOr, // ||
    LogicXor, // ^^
    LogicNot, // !
    BitwiseAnd, // &
    BitwiseOr, // |
    BitwiseXor, // ^
    BitwiseNegate, // ~
    BitwiseShiftLeft, // <<
    BitwiseShiftRight, // >>
    
    Arrow, // =>

    Assign, // =
    Insert, // ->
}


#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128, 
    F32, F64, 

    Char,
    Bool,
}

impl Type {
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
