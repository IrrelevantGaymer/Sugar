use std::fmt::Display;

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
#[derive(Clone, Debug, PartialEq)]
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
