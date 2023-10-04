extern crate num;

use std::fmt::Display;

use num_derive::FromPrimitive;

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

    Semicolon,
    NewLine,
    Spaces(usize),

    Question,
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
            (TknType::NewLine, TknType::NewLine) => true,
            (TknType::Spaces(left), TknType::Spaces(right)) => left == right,
            (TknType::Question, TknType::Question) => true,
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

    Loop,
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
    Integer(u8),
    UnsignedInteger(u8),
    Float(u8),

    Character,
    Boolean,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(size) => write!(f, "i{}", size),
            Self::UnsignedInteger(size) => write!(f, "u{}", size),
            Self::Float(size) => write!(f, "f{}", size),
            Self::Character => write!(f, "char"),
            Self::Boolean => write!(f, "bool")
        }
    }
}
