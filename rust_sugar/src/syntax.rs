use once_cell::sync::Lazy;

use crate::token;
use crate::token::TknType;

use std::collections::HashMap;

pub type Stmt<'s> = Statement<'s>;

#[derive(Clone, Debug)]
pub enum Statement<'s> {
    Compound(Vec<Stmt<'s>>),
    While(Expr<'s>, &'s Stmt<'s>),
    Conditional(Vec<Expr<'s>>, Vec<Stmt<'s>>),
    Return(Expr<'s>),
    Declare(Option<String>, String),
    Assign(Expr<'s>, Expr<'s>),
    Insert(Expr<'s>, Expr<'s>),
}

pub type Expr<'e> = Expression<'e>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'e> {
    Identifier(String),
    Literal(Lit),
    Conditional(&'e Expr<'e>, &'e Expr<'e>, &'e Expr<'e>),
    Functional(String, Vec<Expr<'e>>),
    BinaryOp(BinOp, Box<Expr<'e>>, Box<Expr<'e>>),
    UnaryOp(UnOp, Box<Expr<'e>>),
}

pub type Lit = Literal;

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    IntegerLiteral(i128),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),
    BooleanLiteral(bool),
}

pub type UnOp = UnaryOperator;
#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    LogicNot,
    BitwiseNegate,
    Borrow,
    BorrowMutable,
}

pub type BinOp = BinaryOperator;
#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOperator {
    BangRangeEquals, // !..=
    BangRange,       // !..
    RangeEquals,     // ..=
    Range,           // ..

    Equals,             // ==
    NotEquals,          // !=
    LessThanEqualTo,    // <=
    GreaterThanEqualTo, // >=
    LessThan,           // <
    GreaterThan,        // >

    PlusPlus,          // ++
    Plus,              // +
    MinusMinus,        // --
    Minus,             // -
    Exponent,          // **
    Multiply,          // *
    IntDivide,         // //
    FloatDivide,       // /
    Modulo,            // %
    LogicAnd,          // &&
    LogicOr,           // ||
    LogicXor,          // ^^
    BitwiseAnd,        // &
    BitwiseOr,         // |
    BitwiseXor,        // ^
    BitwiseShiftLeft,  // <<
    BitwiseShiftRight, // >>
}

impl BinOp {
    pub fn get_bin_op(operator: &TknType) -> BinOp {
        return match operator {
            TknType::Operation(token::Op::Exponent) => BinOp::Exponent,
            TknType::Operation(token::Op::Multiply) => BinOp::Multiply,
            TknType::Operation(token::Op::FloatDivide) => BinOp::FloatDivide,
            TknType::Operation(token::Op::IntDivide) => BinOp::IntDivide,
            TknType::Operation(token::Op::Modulo) => BinOp::Modulo,
            TknType::Operation(token::Op::Plus) => BinOp::Plus,
            TknType::Operation(token::Op::Minus) => BinOp::Minus,
            TknType::Operation(token::Op::BitwiseShiftLeft) => BinOp::BitwiseShiftLeft,
            TknType::Operation(token::Op::BitwiseShiftRight) => BinOp::BitwiseShiftRight,
            TknType::Operation(token::Op::BitwiseAnd) => BinOp::BitwiseAnd,
            TknType::Operation(token::Op::BitwiseXor) => BinOp::BitwiseXor,
            TknType::Operation(token::Op::BitwiseOr) => BinOp::BitwiseOr,
            TknType::Operation(token::Op::Equals) => BinOp::Equals,
            TknType::Operation(token::Op::NotEquals) => BinOp::NotEquals,
            TknType::Operation(token::Op::LessThan) => BinOp::LessThan,
            TknType::Operation(token::Op::GreaterThan) => BinOp::GreaterThan,
            TknType::Operation(token::Op::LessThanEqualTo) => BinOp::LessThanEqualTo,
            TknType::Operation(token::Op::GreaterThanEqualTo) => BinOp::GreaterThanEqualTo,
            TknType::Operation(token::Op::LogicAnd) => BinOp::LogicAnd,
            TknType::Operation(token::Op::LogicXor) => BinOp::LogicXor,
            TknType::Operation(token::Op::LogicOr) => BinOp::LogicOr,
            TknType::Operation(token::Op::BangRangeEquals) => BinOp::BangRangeEquals,
            TknType::Operation(token::Op::BangRange) => BinOp::BangRange,
            TknType::Operation(token::Op::RangeEquals) => BinOp::RangeEquals,
            TknType::Operation(token::Op::Range) => BinOp::Range,
            TknType::Operation(token::Op::PlusPlus) => BinOp::PlusPlus,
            _ => panic!("Invalid TokenType, expecting applicable Operation"),
        };
    }
}

pub type OpInfo<'o> = OperatorInfo<'o>;
pub struct OperatorInfo<'o> {
    token: TknType<'o>,
}

pub static OPERATOR_INFO_MAP: Lazy<HashMap<TknType, (OpPrec, OpAssoc)>> =
    Lazy::<HashMap<TknType, (OpPrec, OpAssoc)>>::new(|| {
        HashMap::from([
            (
                TknType::Operation(token::Op::Exponent),
                (OpPrec::Exponent, OpAssoc::Right),
            ),
            (
                TknType::Operation(token::Op::Multiply),
                (OpPrec::MultiplicationDivisionModulo, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::FloatDivide),
                (OpPrec::MultiplicationDivisionModulo, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::IntDivide),
                (OpPrec::MultiplicationDivisionModulo, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::Modulo),
                (OpPrec::MultiplicationDivisionModulo, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::Plus),
                (OpPrec::AdditionSubtraction, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::Minus),
                (OpPrec::AdditionSubtraction, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::BitwiseShiftLeft),
                (OpPrec::BitwiseShift, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::BitwiseShiftRight),
                (OpPrec::BitwiseShift, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::BitwiseAnd),
                (OpPrec::BitwiseAnd, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::BitwiseXor),
                (OpPrec::BitwiseXor, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::BitwiseOr),
                (OpPrec::BitwiseOr, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::Equals),
                (OpPrec::Relational, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::NotEquals),
                (OpPrec::Relational, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::LessThan),
                (OpPrec::Relational, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::GreaterThan),
                (OpPrec::Relational, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::LessThanEqualTo),
                (OpPrec::Relational, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::GreaterThanEqualTo),
                (OpPrec::Relational, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::LogicAnd),
                (OpPrec::LogicAnd, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::LogicXor),
                (OpPrec::LogicXor, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::LogicOr),
                (OpPrec::LogicOr, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::BangRangeEquals),
                (OpPrec::Ranges, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::BangRange),
                (OpPrec::Ranges, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::RangeEquals),
                (OpPrec::Ranges, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::Range),
                (OpPrec::Ranges, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::PlusPlus),
                (OpPrec::Concatenation, OpAssoc::Left),
            ),
        ])
    });

pub type OpPrec = OperatorPrecedence;
#[derive(Clone, Copy)]
pub enum OperatorPrecedence {
    Concatenation = 1,
    Ranges = 2,
    LogicOr = 3,
    LogicXor = 4,
    LogicAnd = 5,
    Relational = 6,
    BitwiseOr = 7,
    BitwiseXor = 8,
    BitwiseAnd = 9,
    BitwiseShift = 10,
    AdditionSubtraction = 11,
    MultiplicationDivisionModulo = 12,
    Exponent = 13,
    Casting = 14,
}

pub type OpAssoc = OperatorAssociativity;
#[derive(Clone, Copy, PartialEq)]
pub enum OperatorAssociativity {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub accessibility: String,
    pub field_type: String,
    pub field_name: String,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub accessibility: String,
    pub location: String,
    pub name: String,
    pub fields: Vec<Field>,
}

pub type Fn<'f> = Function<'f>;
#[derive(Clone, Debug)]
pub struct Function<'f> {
    pub location: String,
    pub accessibility: String,
    pub mutable: bool,
    pub recursive: bool,
    pub name: String,
    pub arguments: &'f [FnParam<'f>],
    pub return_type: String,
    pub body: Stmt<'f>,
}

pub type FnParam<'fp> = FunctionParamater<'fp>;
#[derive(Clone, Debug)]
pub struct FunctionParamater<'fp> {
    pub param_type: Option<String>,
    pub param_name: Option<String>,
    pub param_default: Option<Expr<'fp>>,
}

impl<'fp> FunctionParamater<'fp> {
    pub fn default() -> FunctionParamater<'fp> {
        return FunctionParamater {
            param_type: None,
            param_name: None,
            param_default: None,
        };
    }
}
