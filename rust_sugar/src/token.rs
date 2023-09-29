
pub type Tkn<'t> = Token<'t>;
pub struct Token<'t> {
    token: TokenType<'t>
}

pub type TknType<'t> = TokenType<'t>;
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

pub enum Type {
    Integer(u8),
    UnsignedInteger(u8),

    Character,
    Boolean,
}