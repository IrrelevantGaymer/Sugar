use crate::parser::{expr::{Expr, ExprData, ExprType, ExprTypeCons, Lit}, operators::UnOp, stmt::StackLocation, ExprBump};

use super::{Interpreter, VariableData};

pub fn evaluate_un_op<'tkns, 'bumps, 'defs>(
    interpreter: &mut Interpreter<'tkns, 'bumps, 'defs, '_>,
    expr_bump: &'bumps ExprBump,
    line: usize,
    unary_operator: UnOp, 
    expr: &Expr<'bumps, 'defs>,
    expected_type: &ExprType,
    local_scoping: bool,
) -> Option<VariableData> {
    let expr_data = interpreter.evaluate_expression(expr_bump, expr.clone(), expected_type, local_scoping)?;
    let expr = unsafe {
        interpreter.get_expr_from_variable_data(expr_bump, &expr_data)
    };
    let out_type = unary_operator.transform_type(
        ExprTypeCons::new(expr_bump, expr.expr_type.clone())
    ).unwrap().clone_inner();

    macro_rules! evaluate {
        (expr: $expr:ident, in: $in_type:ident, out: $out_type:ident, calculate: $calculate:expr) => {
            if let ExprData::Literal(Lit::$in_type($expr)) = expr.expr_data {
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
                    format!("line {line}: could not interpret expression {expr:?}").as_str()
                );
                
                interpreter.stack_write(&value_data, &bytes);
                return Some(value_data);
            }
        };
    }

    match unary_operator {
        UnOp::PlusFloat => {
            evaluate!(
                expr: float, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: *float
            );
            panic!("line {line}: type {:?} for unary operation {:?} in expression {expr:?} is not supported.", expr.expr_type, unary_operator);
        }
        UnOp::Plus => {
            evaluate!(
                expr: int, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: *int
            );
            panic!("line {line}: type {:?} for unary operation {:?} in expression {expr:?} is not supported.", expr.expr_type, unary_operator);
        }
        UnOp::MinusFloat => {
            evaluate!(
                expr: float, 
                in: FloatLiteral, out: FloatLiteral, 
                calculate: -float
            );
            panic!("line {line}: type {:?} for unary operation {:?} in expression {expr:?} is not supported.", expr.expr_type, unary_operator);
        }
        UnOp::Minus => {
            evaluate!(
                expr: int, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: -int
            );
            panic!("line {line}: type {:?} for unary operation {:?} in expression {expr:?} is not supported.", expr.expr_type, unary_operator);
        }
        UnOp::LogicNot => {
            evaluate!(
                expr: bool, 
                in: BooleanLiteral, out: BooleanLiteral, 
                calculate: !bool
            );
            panic!("line {line}: type {:?} for unary operation {:?} in expression {expr:?} is not supported.", expr.expr_type, unary_operator);
        }
        UnOp::BitwiseNegate => {
            evaluate!(
                expr: int, 
                in: IntegerLiteral, out: IntegerLiteral, 
                calculate: !int
            );
            panic!("line {line}: type {:?} for unary operation {:?} in expression {expr:?} is not supported.", expr.expr_type, unary_operator);
        }
        UnOp::Borrow => todo!(),
        UnOp::BorrowInteriorMutable => todo!(),
        UnOp::BorrowMutable => todo!(),
    }
}