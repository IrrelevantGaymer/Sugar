
type Stmt<'s> = Statement<'s>;

#[derive(Clone, Debug)]
pub enum Statement<'s> {
    Compound(&'s [Stmt<'s>]),
    While(Expr<'s>, &'s Stmt<'s>),
    Conditional(&'s [Expr<'s>], &'s [Stmt<'s>]),
    Return(Expr<'s>),
    Function(Fn<'s>),
    Assign(Expr<'s>, Expr<'s>),
    Insert(Expr<'s>, Expr<'s>),
}

type Expr<'e> = Expression<'e>;

#[derive(Clone, Debug)]
pub enum Expression<'e> {
    Identifier(String),
    Literal(Lit),
    Conditional(&'e Expr<'e>, &'e Expr<'e>, &'e Expr<'e>),
    Functional(String, &'e [Expr<'e>]),
    BinaryOp(Op, &'e Expr<'e>, &'e Expr<'e>),
    UnaryOp(Op, &'e Expr<'e>)

}

type Lit = Literal;

#[derive(Clone, Debug)]
pub enum Literal {
    IntegerLiteral(i128),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),
}

pub type Op = Operator;
#[derive(Clone, Debug)]
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
}

type Fn<'f> = Function<'f>;
#[derive(Clone, Debug)]
pub struct Function<'f> {
    mutable: bool,
    recursive: bool,
    name: String,
    arguments: &'f [FnParam<'f>],
    body: &'f Stmt<'f>
}

type FnParam<'fp> = FunctionParamater<'fp>;
#[derive(Clone, Debug)]
pub struct FunctionParamater<'fp> {
    param_type: String,
    param_name: String,
    param_default: Option<Expr<'fp>>
}