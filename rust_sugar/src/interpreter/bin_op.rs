use crate::parser::{expr::{Expr, ExprData, ExprType, ExprTypeCons, Lit}, operators::BinOp, stmt::StackLocation, ExprBump};

use super::{Interpreter, VariableData};

pub fn evaluate_bin_op<'tkns, 'bumps, 'defs>(
    interpreter: &mut Interpreter<'tkns, 'bumps, 'defs, '_>,
    expr_bump: &'bumps ExprBump,
    line: usize,
    expr: Expr<'bumps, 'defs>,
    binary_operator: BinOp, 
    left: &Expr<'bumps, 'defs>, 
    right: &Expr<'bumps, 'defs>,
    expected_type: &ExprType,
    local_scoping: bool,
) -> Option<VariableData> {
    let left_expr; 
    let right_expr;
    let left_data = interpreter.evaluate_expression(expr_bump, left.clone(), expected_type, local_scoping)?;
    let right_data = interpreter.evaluate_expression(expr_bump, right.clone(), expected_type, local_scoping)?;
    unsafe {
        left_expr = interpreter.get_expr_from_variable_data(expr_bump, &left_data);
        right_expr = interpreter.get_expr_from_variable_data(expr_bump, &right_data);
    }
    ////println!("binary operator {binary_operator:?} to {left_expr:?} and {right_expr:?}");
    let out_type = binary_operator.transform_type(
        expr_bump,
        &mut ExprTypeCons::new(expr_bump, left_expr.expr_type.clone()), 
        &mut ExprTypeCons::new(expr_bump, right_expr.expr_type.clone())
    ).unwrap().clone_inner();

    macro_rules! evaluate {
        (
            left: $left:ident, right: $right:ident, 
            in: $in_type:ident, out: $out_type:ident, 
            calculate: $calculate:expr
        ) => {
            if let ExprData::Literal(Lit::$in_type($left)) = left_expr.expr_data
                && let ExprData::Literal(Lit::$in_type($right)) = right_expr.expr_data
            {
                let value = expr_bump.alloc(
                    ExprData::Literal(Lit::$out_type($calculate))
                );
                let value_data = interpreter.stack_alloc(line, &out_type, StackLocation::Oxy);

                let bytes = interpreter.to_interpreter_bytes(
                    expr_bump, 
                    value, 
                    &out_type, 
                    line, 
                    expected_type,
                    local_scoping
                ).expect(
                    format!(
                        "line {}: could not interpret expression {:?}",
                        line, expr
                    ).as_str()
                );

                interpreter.stack_write(&value_data, &bytes);
                return Some(value_data);
            }
        };
        (
            left: $left:ident, right: $right:ident, 
            in: $in_type:ident, out: $out_type:ident, 
            calculate: $calculate:expr, 
            $guard:expr
        ) => {
            if let ExprData::Literal(Lit::$in_type($left)) = left_expr.expr_data
                && let ExprData::Literal(Lit::$in_type($right)) = right_expr.expr_data
            {
                $guard

                let value = expr_bump.alloc(
                    ExprData::Literal(Lit::$out_type($calculate))
                );
                let value_data = interpreter.stack_alloc(line, &out_type, StackLocation::Oxy);

                let bytes = interpreter.to_interpreter_bytes(
                    expr_bump, 
                    value, 
                    &out_type, 
                    line, 
                    expected_type,
                    local_scoping
                ).expect(
                    format!(
                        "line {}: could not interpret expression {:?}",
                        line, expr
                    ).as_str()
                );

                interpreter.stack_write(&value_data, &bytes);
                return Some(value_data);
            }
        };
    }

    match binary_operator {
        BinOp::ExponentFloat => {
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: float1.powf(*float2)
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Exponent => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1.pow(*int2 as u32),
                if *int2 < 0 {
                    panic!("line {line}: Exponent for Integer Types cannot use a negative value for the Exponent. {int2} is negative.");
                }
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::MultiplyFloat => {
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: float1 * float2
            );  
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Multiply => {
            evaluate!(
                left: int1, right: int2, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: int1 * int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::DivideFloat => {
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: float1 / float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Divide => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 / int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::ModuloFloat => {
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: float1 % float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Modulo => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 % int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::PlusFloat => {
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: float1 + float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Plus => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 + int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::MinusFloat => {
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: float1 - float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Minus => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 - int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::LogicAnd => {
            evaluate!(
                left: bool1, right: bool2, 
                in: BooleanLiteral, out: BooleanLiteral, 
                calculate: *bool1 && *bool2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::LogicOr => {
            evaluate!(
                left: bool1, right: bool2, 
                in: BooleanLiteral, out: BooleanLiteral, 
                calculate: *bool1 || *bool2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::BitwiseXor => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 ^ int2
            );
            evaluate!(
                left: bool1, right: bool2, 
                in: BooleanLiteral, out: BooleanLiteral, 
                calculate: bool1 ^ bool2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::BitwiseAnd => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 & int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::BitwiseOr => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 | int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::BitwiseShiftLeft => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 << int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::BitwiseShiftRight => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: int1 >> int2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Equals => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: BooleanLiteral, 
                calculate: int1 == int2
            );
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: BooleanLiteral, 
                calculate: float1 == float2
            );
            evaluate!(
                left: bool1, right: bool2, 
                in: BooleanLiteral, out: BooleanLiteral, 
                calculate: bool1 == bool2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::NotEquals => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: BooleanLiteral, 
                calculate: int1 != int2
            );
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: BooleanLiteral, 
                calculate: float1 != float2
            );
            evaluate!(
                left: bool1, right: bool2, 
                in: BooleanLiteral, out: BooleanLiteral, 
                calculate: bool1 != bool2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::LessThan => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: BooleanLiteral, 
                calculate: int1 < int2
            );
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: BooleanLiteral, 
                calculate: float1 < float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::LessThanEqualTo => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: BooleanLiteral, 
                calculate: int1 <= int2
            );
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: BooleanLiteral, 
                calculate: float1 <= float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::GreaterThan => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: BooleanLiteral, 
                calculate: int1 > int2
            );
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: BooleanLiteral, 
                calculate: float1 > float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::GreaterThanEqualTo => {
            evaluate!(
                left: int1, right: int2, 
                in: IntegerLiteral, out: BooleanLiteral, 
                calculate: int1 >= int2
            );
            evaluate!(
                left: float1, right: float2, 
                in: FloatLiteral, out: BooleanLiteral, 
                calculate: float1 >= float2
            );
            panic!("line {line}: types {:?} and {:?} for binary operation {:?} in expression {expr:?} is not supported.", left_expr.expr_type, right_expr.expr_type, binary_operator);
        }
        BinOp::Concat => todo!(),
        BinOp::Range => todo!(), 
        BinOp::BangRangeEquals => todo!(), 
        BinOp::BangRange => todo!(), 
        BinOp::RangeEquals => todo!(),
    }
}