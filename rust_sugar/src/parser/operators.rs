use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::lexer::token::{self, TknType};

use super::expr::{ExprType, ExprTypeCons, ExpressionType};

pub type UnOp = UnaryOperator;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Plus,
    Minus,
    LogicNot,
    BitwiseNegate,
    Borrow,
    BorrowMutable,
}

impl UnOp {
    pub fn transform_type(self, expr_type: ExprTypeCons) -> Option<ExprTypeCons> {
        use ExpressionType as ET;
        use UnaryOperator as UO;

        match (self, expr_type.clone_inner()) {
            (UO::LogicNot, ET::Bool) |
            (UO::Minus, 
                ET::AmbiguousFloat | ET::F32 | ET::F64 | 
                ET::AmbiguousNegInteger | ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128
            ) |
            (UO::Plus, 
                ET::AmbiguousFloat | ET::F32 | ET::F64 |
                ET::AmbiguousNegInteger | ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 |
                ET::AmbiguousPosInteger | ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
            ) => Some(expr_type),
            _ => None
        }
    }
}

pub type BinOp = BinaryOperator;
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn transform_type<'etc>(
        self, 
        left: ExprTypeCons<'etc>, 
        right: ExprTypeCons<'etc>,
    ) -> Option<ExprTypeCons<'etc>> {
        use ExprType as ET;
        use BinOp as BO;

        let left_expr_type = left.clone_inner();
        let right_expr_type = right.clone_inner();

        match (self, left_expr_type, right_expr_type) {
            (BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo,
                l @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128 | 
                    ET::F32 | ET::F64
                ), 
                r @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128 | 
                    ET::F32 | ET::F64
                ),
            ) if l == r => return Some(left),
            (BO::IntDivide | BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                l @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 |
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
                ), 
                r @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 |
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
                ),
            ) if l == r => return Some(left),
            (BO::FloatDivide,
                l @ (ET::F32 | ET::F64),
                r @ (ET::F32 | ET::F64),
            ) if l == r => return Some(left),
            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                l @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128 
                ), 
                r @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128 
                ),
            ) if l == r => return Some(ExprTypeCons::new_temp(ET::Bool)),

            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                l @ (ET::F32 | ET::F64), 
                r @ (ET::F32 | ET::F64),
            ) if l == r => return Some(ExprTypeCons::new_temp(ET::Bool)),

            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | BO::IntDivide | 
                    BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::AmbiguousPosInteger | ET::AmbiguousNegInteger, 
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128,
            ) |
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | BO::IntDivide |
                    BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128,
                ET::AmbiguousPosInteger | ET::AmbiguousNegInteger, 
            ) => {
                return left.match_type(right)
            },

            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | BO::IntDivide | 
                    BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::AmbiguousPosInteger, 
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128,
            ) |
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | BO::IntDivide |
                    BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128,
                ET::AmbiguousPosInteger, 
            ) => return left.match_type(right),

            (BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | BO::FloatDivide,
                ET::AmbiguousFloat, 
                ET::F32 | ET::F64,
            ) | 
            (BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | BO::FloatDivide,
                ET::F32 | ET::F64, 
                ET::AmbiguousFloat, 
            ) => return left.match_type(right),

            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                ET::AmbiguousPosInteger | ET::AmbiguousNegInteger, 
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128,
            ) |
            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128,
                ET::AmbiguousPosInteger | ET::AmbiguousNegInteger, 
            ) => {
                left.match_type(right);
                return Some(ExprTypeCons::new_temp(ET::Bool));
            },

            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                ET::AmbiguousPosInteger, 
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128,
            ) |
            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128,
                ET::AmbiguousPosInteger, 
            ) => {
                left.match_type(right);
                return Some(ExprTypeCons::new_temp(ET::Bool));
            },

            (
                BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                ET::AmbiguousFloat, 
                ET::F32 | ET::F64,
            ) |
            (
                BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                ET::F32 | ET::F64,
                ET::AmbiguousFloat, 
            ) => {
                left.match_type(right);
                return Some(ExprTypeCons::new_temp(ET::Bool));
            },

            (BO::LogicAnd | BO::LogicOr | BO::LogicXor,
                ET::Bool, ET::Bool
            ) => return Some(left),

            _ => return None
        }

        // match (self, &mut *left.get().borrow_mut(), &mut *right.get().borrow_mut()) {
        //     (Plus | Minus | Exponent | Multiply | Modulo,
        //         l @ (I8 | I16 | I32 | I64 | I128 | U8 | U16 | U32 | U64 | U128 | F32 | F64), 
        //         r @ (I8 | I16 | I32 | I64 | I128 | U8 | U16 | U32 | U64 | U128 | F32 | F64)
        //     ) if l == r => (),
        //     (Plus | Minus | Exponent | Multiply | Modulo,
        //         l @ (AmbiguousPosInteger | AmbiguousNegInteger), 
        //         r @ (I8 | I16 | I32 | I64 | I128)
        //     ) => {
        //         left.match_type(right);
        //     },
        //     (Plus | Minus | Exponent | Multiply | Modulo,
        //         l @ (I8 | I16 | I32 | I64 | I128),
        //         r @ (AmbiguousPosInteger | AmbiguousNegInteger), 
        //     ) => {
        //         right.match_type(left);
        //     },
        //     // (Plus | Minus | Exponent | Multiply | Modulo,
        //     //     AmbiguousPosInteger
        //     // )
        //     (IntDivide, 
        //         l @ (I8 | I16 | I32 | I64 | I128), 
        //         r @ (I8 | I16 | I32 | I64 | I128)
        //     ) if l == r => (),
        //     (IntDivide, 
        //         l @ (AmbiguousPosInteger | AmbiguousNegInteger), 
        //         r @ (I8 | I16 | I32 | I64 | I128)
        //     ) => {
        //         left.match_type(right);
        //     },
        //     (IntDivide, 
        //         l @ (I8 | I16 | I32 | I64 | I128),
        //         r @ (AmbiguousPosInteger | AmbiguousNegInteger), 
        //     ) => {
        //         right.match_type(left);
        //     },
        //     (FloatDivide,
        //         l @ (F32 | F64),
        //         r @ (F32 | F64)
        //     ) if l == r => (),
        //     _ => return None
        // }
    }
    
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
    }
);

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