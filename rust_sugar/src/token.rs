
pub type Tkn<'t> = Token<'t>;
#[derive(Clone)]
pub struct Token<'t> {
    token: TokenType<'t>,
    file_name: String,
    line_index: usize,
    line_number: usize
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

pub type TknType<'t> = TokenType<'t>;
#[derive(Clone)]
pub enum TokenType<'t> {
    Keyword(Keyword),
    Type(Type),
    Identifier(String),
    Operator(Operator),

    IntegerLiteral(i128),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),

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

    Dot,
    Borrow,

    Either(&'t TokenType<'t>, &'t TokenType<'t>),

    EndOfFile,
    Invalid,
}

pub type Kwrd = Keyword;
#[derive(Clone)]
pub enum Keyword {
    Let,

    Loop,
    Mutable,
    Oxidize,
    Unsafe,

    Function,
    Accessor,
    Whitelist,
    Blacklist,
    Struct,

    Namespace,
    Alias,

    Public,
    Private,
    Package,
}

pub type Op = Operator;
#[derive(Clone)]
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
    
    Assign, // =
    Insert, // ->
}

#[derive(Clone)]
pub enum Type {
    Integer(u8),
    UnsignedInteger(u8),
    Float(u8),

    Character,
    Boolean,
}