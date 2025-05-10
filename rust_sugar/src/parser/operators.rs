use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::lexer::token::{self, Op, TknType};

use super::{expr::{ExprType, ExprTypeCons, ExpressionType}, ExprBump};

pub type UnOp = UnaryOperator;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    PlusFloat,
    Plus,
    MinusFloat,
    Minus,
    LogicNot,
    BitwiseNegate,
    Borrow,
    BorrowInteriorMutable,
    BorrowMutable,
}

impl UnOp {
    pub fn transform_type(self, expr_type: ExprTypeCons) -> Option<ExprTypeCons> {
        use ExpressionType as ET;
        use UnaryOperator as UO;

        match (self, expr_type.clone_inner()) {
            (_, ET::Never) => Some(expr_type),
            (UO::LogicNot, ET::Bool) |
            (UO::PlusFloat, 
                ET::AmbiguousFloat | ET::F32 | ET::F64
            ) |
            (UO::MinusFloat, 
                ET::AmbiguousFloat | ET::F32 | ET::F64
            ) |
            (UO::Plus, 
                ET::AmbiguousNegInteger | ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 |
                ET::AmbiguousPosInteger | ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
            ) |
            (UO::Minus, 
                ET::AmbiguousNegInteger | ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128
            ) => Some(expr_type),
            _ => None
        }
    }
}

pub type BinOp = BinaryOperator;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
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

    /// ++
    Concat,
    /// +
    Plus,
    /// +.
    PlusFloat,
    /// -
    Minus,
    /// -.
    MinusFloat,
    /// **
    Exponent,
    /// **.
    ExponentFloat,
    /// *
    Multiply,
    /// *.
    MultiplyFloat,
    /// /
    Divide,
    /// /.
    DivideFloat,
    /// %
    Modulo,
    /// %.
    ModuloFloat,
    /// &&
    LogicAnd,
    /// ||
    LogicOr,
    /// &
    BitwiseAnd,
    /// |
    BitwiseOr,
    /// ^
    BitwiseXor,
    /// <<
    BitwiseShiftLeft,
    /// >>
    BitwiseShiftRight,
}

impl BinOp {
    pub fn transform_type<'bumps>(
        self, 
        expr_bump: &'bumps ExprBump,
        left: &mut ExprTypeCons<'bumps>, 
        right: &mut ExprTypeCons<'bumps>,
    ) -> Option<ExprTypeCons<'bumps>> {
        use ExprType as ET;
        use BinOp as BO;

        let left_expr_type = left.clone_inner();
        let right_expr_type = right.clone_inner();

        match (self, left_expr_type, right_expr_type) {
            //Propogating Never Types
            (_, ET::Never, _) => return Some(ExprTypeCons::new(expr_bump, ET::Never)),
            (_, _, ET::Never) => return Some(ExprTypeCons::new(expr_bump, ET::Never)),
            
            //Unambiguous Integer Operations
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | 
                    BO::Divide | BO::Modulo | BO::BitwiseAnd | BO::BitwiseOr | 
                    BO::BitwiseXor,
                l @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
                ), 
                r @ (ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128 | 
                    ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128
                ),
            ) if l == r => return Some(left.clone()),

            //Unambiguous Floating Point Operations
            (
                BO::PlusFloat | BO::MinusFloat | BO::ExponentFloat | 
                    BO::MultiplyFloat | BO::DivideFloat,
                l @ (ET::F32 | ET::F64),
                r @ (ET::F32 | ET::F64),
            ) if l == r => return Some(left.clone()),

            //Unambiguous Integer Comparisons
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
            ) if l == r => return Some(ExprTypeCons::new(expr_bump, ET::Bool)),

            //Unambiguous Floating Point Comparisons
            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                l @ (ET::F32 | ET::F64), 
                r @ (ET::F32 | ET::F64),
            ) if l == r => return Some(ExprTypeCons::new(expr_bump, ET::Bool)),

            //Other Comparisons
            (
                BO::Equals | BO::NotEquals | 
                    BO::LessThanEqualTo | BO::GreaterThanEqualTo | 
                    BO::LessThan | BO::GreaterThan,
                l @ ET::Char, 
                r @ ET::Char,
            ) if l == r => return Some(ExprTypeCons::new(expr_bump, ET::Bool)),

            //Potentially Ambiguous Integer Operations
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | 
                    BO::Divide | BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::AmbiguousPosInteger | ET::AmbiguousNegInteger, 
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128,
            ) |
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | 
                    BO::Divide | BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::I8 | ET::I16 | ET::I32 | ET::I64 | ET::I128,
                ET::AmbiguousPosInteger | ET::AmbiguousNegInteger, 
            ) |
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | 
                    BO::Divide |  BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::AmbiguousPosInteger, 
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128,
            ) |
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | 
                    BO::Divide | BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::U8 | ET::U16 | ET::U32 | ET::U64 | ET::U128,
                ET::AmbiguousPosInteger, 
            ) |
            (
                BO::Plus | BO::Minus | BO::Exponent | BO::Multiply | BO::Modulo | BO::Divide | 
                    BO::BitwiseAnd | BO::BitwiseOr | BO::BitwiseXor,
                ET::AmbiguousNegInteger | ET::AmbiguousPosInteger,
                ET::AmbiguousNegInteger | ET::AmbiguousPosInteger
            ) => return left.match_type(right),

            //Potentially Ambiguous Floating Point Operations
            (
                BO::PlusFloat | BO::MinusFloat | BO::ExponentFloat | 
                    BO::MultiplyFloat | BO::DivideFloat,
                ET::AmbiguousFloat, 
                ET::F32 | ET::F64,
            ) | 
            (
                BO::PlusFloat | BO::MinusFloat | BO::ExponentFloat | 
                    BO::MultiplyFloat | BO::DivideFloat,
                ET::F32 | ET::F64, 
                ET::AmbiguousFloat, 
            ) |
            (
                BO::PlusFloat | BO::MinusFloat | BO::ExponentFloat | 
                    BO::MultiplyFloat | BO::DivideFloat,
                ET::AmbiguousFloat, 
                ET::AmbiguousFloat, 
            )=> return left.match_type(right),

            //Potentially Ambiguous Integer Comparisons
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
            ) |
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
                return Some(ExprTypeCons::new(expr_bump, ET::Bool));
            },

            //Potentially Ambiguous Floating Point Comparisons
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
                return Some(ExprTypeCons::new(expr_bump, ET::Bool));
            },

            //Boolean Operations
            (BO::LogicAnd | BO::LogicOr | BO::BitwiseXor,
                ET::Bool, ET::Bool
            ) => return Some(left.clone()),

            _ => return None
        }
    }
    
    pub fn get_bin_op(operator: &TknType) -> BinOp {
        let TknType::Operation(operator) = operator else {
            panic!("Invalid TokenType, expecting applicable Operation");
        };
        return match operator {
            Op::ExponentFloat => BinOp::ExponentFloat,
            Op::Exponent => BinOp::Exponent,
            Op::MultiplyFloat => BinOp::MultiplyFloat,
            Op::Multiply => BinOp::Multiply,
            Op::DivideFloat => BinOp::DivideFloat,
            Op::Divide => BinOp::Divide,
            Op::ModuloFloat => BinOp::ModuloFloat,
            Op::Modulo => BinOp::Modulo,
            Op::PlusFloat => BinOp::PlusFloat,
            Op::Plus => BinOp::Plus,
            Op::MinusFloat => BinOp::MinusFloat,
            Op::Minus => BinOp::Minus,
            Op::BitwiseShiftLeft => BinOp::BitwiseShiftLeft,
            Op::BitwiseShiftRight => BinOp::BitwiseShiftRight,
            Op::BitwiseAnd => BinOp::BitwiseAnd,
            Op::BitwiseXor => BinOp::BitwiseXor,
            Op::BitwiseOr => BinOp::BitwiseOr,
            Op::Equals => BinOp::Equals,
            Op::NotEquals => BinOp::NotEquals,
            Op::LessThan => BinOp::LessThan,
            Op::GreaterThan => BinOp::GreaterThan,
            Op::LessThanEqualTo => BinOp::LessThanEqualTo,
            Op::GreaterThanEqualTo => BinOp::GreaterThanEqualTo,
            Op::LogicAnd => BinOp::LogicAnd,
            Op::LogicOr => BinOp::LogicOr,
            Op::BangRangeEquals => BinOp::BangRangeEquals,
            Op::BangRange => BinOp::BangRange,
            Op::RangeEquals => BinOp::RangeEquals,
            Op::Range => BinOp::Range,
            Op::PlusPlus => BinOp::Concat,
            Op::ConcatEquals            |
            Op::PlusFloatEquals         |
            Op::PlusEquals              |
            Op::MinusFloatEquals        |
            Op::MinusEquals             |
            Op::ExponentFloatEquals     |
            Op::ExponentEquals          |
            Op::MultiplyFloatEquals     |
            Op::MultiplyEquals          |
            Op::DivideFloatEquals       |
            Op::DivideEquals            |
            Op::ModuloFloatEquals       |
            Op::ModuloEquals            |
            Op::LogicAndEquals          |
            Op::LogicOrEquals           |
            Op::BitwiseAndEquals        |
            Op::BitwiseOrEquals         |
            Op::BitwiseXorEquals        |
            Op::BitwiseShiftLeftEquals  |
            Op::BitwiseShiftRightEquals |
            Op::LogicNot                |
            Op::BitwiseNegate           |
            Op::Arrow                   |
            Op::Assign => panic!("Not Binary Operation"),
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
                TknType::Operation(token::Op::DivideFloat),
                (OpPrec::MultiplicationDivisionModulo, OpAssoc::Left),
            ),
            (
                TknType::Operation(token::Op::Divide),
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