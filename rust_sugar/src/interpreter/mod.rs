use std::{collections::HashMap, ops::{Add, AddAssign}, ptr::NonNull};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;
use text_io::try_read;

use crate::parser::{accessors::Accessor, expr::{Expr, ExprData, ExprType, ExprTypeCons, Lit}, functions::{BuiltInFunction, FnParam, Fun}, stmt::{StackLocation, Stmt, StmtData}, structs::Struct, ExprBump};

pub mod bin_op;
pub mod un_op;

#[allow(dead_code)]
pub struct Interpreter<'tkns, 'bumps, 'defs, 'i> {
    oxy_stack: NonNull<[u8]>,
    gc_stack: NonNull<[u8]>,
    oxy_stack_ptr: usize,
    gc_stack_ptr: usize,
    variables: Vec<StackFrameDictAllocator<'i, String, VariableData>>,

    accessors: &'defs [Accessor],
    defs: &'defs [Struct],
    functions: &'defs [Fun<'tkns, 'bumps, 'defs>]
}

#[derive(Clone, Debug)]
pub struct VariableData {
    index: StackIndex,
    expr_type: ExprType
}

impl std::fmt::Display for VariableData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VariableData {{ index: {:?}, expr_type: {:?} }}", self.index, self.expr_type)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StackIndex {
    GC(usize),
    Oxy(usize)
}

impl Add<usize> for StackIndex {
    type Output = StackIndex;

    fn add(self, rhs: usize) -> Self::Output {
        match self {
            StackIndex::GC(index) => StackIndex::GC(index + rhs),
            StackIndex::Oxy(index) => StackIndex::Oxy(index + rhs),
        }
    }
}

impl AddAssign<usize> for StackIndex {
    fn add_assign(&mut self, rhs: usize) {
        match self {
            StackIndex::GC(index) => *index += rhs,
            StackIndex::Oxy(index) => *index += rhs,
        }
    }
}

type TypedExpr<'bumps, 'types, 'defs> = TypedExpression<'bumps, 'types, 'defs>;
#[derive(Debug)]
pub struct TypedExpression<'bumps, 'types, 'defs> {
    expr_data: &'bumps ExprData<'bumps, 'defs>,
    expr_type: &'types ExprType
}

impl<'tkns, 'bumps, 'defs, 'i> Interpreter<'tkns, 'bumps, 'defs, 'i> {
    const OXY_STACK_SIZE: usize = 1024;
    const GC_STACK_SIZE: usize = 1024;
    
    pub fn new(
        (accessors, defs, functions): (
            &'defs [Accessor], 
            &'defs [Struct], 
            &'defs [Fun<'tkns, 'bumps, 'defs>]
        )
    ) -> Self {
        let (oxy_stack, gc_stack) = unsafe {
            let oxy_stack = core::slice::from_raw_parts(std::alloc::alloc(
                std::alloc::Layout::array::<u8>(Self::OXY_STACK_SIZE).unwrap_unchecked()
            ), Self::OXY_STACK_SIZE);
            let gc_stack = core::slice::from_raw_parts(std::alloc::alloc(
                std::alloc::Layout::array::<u8>(Self::GC_STACK_SIZE).unwrap_unchecked()
            ), Self::GC_STACK_SIZE);

            (
                NonNull::new_unchecked(oxy_stack as *const [u8] as *mut [u8]), 
                NonNull::new_unchecked(gc_stack as *const [u8] as *mut [u8])
            )
        };

        return Interpreter {
            oxy_stack,
            gc_stack,
            oxy_stack_ptr: 0,
            gc_stack_ptr: 0,
            variables: vec![],

            accessors, defs, functions
        };
    }

    pub fn interpret(&mut self, expr_bump: &'bumps ExprBump) {
        let Some(main_fun) = self.functions.iter().find(|e| e.name == "main") else {
            eprintln!("no entry point to the program was found");
            return;
        };

        if !main_fun.left_args.is_empty() || !main_fun.right_args.is_empty() {
            eprintln!("a main function with arguments is not supported");
            return;
        }

        if main_fun.return_type != ExprType::Void {
            eprintln!("a main function with return type is not supported");
            return;
        }

        self.variables.push(StackFrameDictAllocator::new());
        //println!("lets a go");
        self.interpret_statements(expr_bump, &main_fun.body, &main_fun.return_type)
            .expect("Did not expect return value");
    }

    pub fn interpret_statements(
        &mut self, 
        expr_bump: &'bumps ExprBump,
        stmts: &[&'bumps StmtData<'bumps, 'defs>],
        expected_type: &ExprType
    ) -> Result<(), VariableData> {
        let oxy_stack_ptr_start = self.oxy_stack_ptr;
        let gc_stack_ptr_start = self.gc_stack_ptr;
        
        'stmts: for StmtData { line, stmt } in stmts {
            //println!("stmt {stmt:?} on line {line}");
            match stmt {
                Stmt::Compound(stmts) => {
                    self.variables.push(self.variables(true).new_frame());
                    let output = self.interpret_statements(
                        expr_bump, 
                        stmts.as_slice(), 
                        expected_type
                    );
                    self.variables.pop();

                    output?;
                },
                Stmt::While { cond, body } => {
                    loop {
                        let Some(variable_data) = self.evaluate_expression(
                            expr_bump, 
                            cond.clone(), 
                            expected_type,
                            true
                        ) else {
                            panic!("evaluating condition should yield something");
                        };
    
                        if variable_data.expr_type != ExprType::Bool {
                            panic!("encountered non bool type in conditional statement");
                        }
    
                        let ExprData::Literal(Lit::BooleanLiteral(eval_cond)) = (unsafe {
                            self.get_expr_from_variable_data(expr_bump, &variable_data).expr_data}
                        ) else {
                            unreachable!("expr_data should be a boolean");
                        };

                        if !eval_cond {
                            break;
                        }
                        
                        self.variables.push(self.variables(true).new_frame());
                        let output = self.interpret_statements(
                            expr_bump, 
                            body.as_slice(), 
                            expected_type
                        );
                        self.variables.pop();

                        output?;
                    }
                },
                Stmt::Conditional { conds, bodies } => {
                    for (cond, body) in conds.iter().zip(bodies) {
                        let Some(variable_data) = self.evaluate_expression(
                            expr_bump, 
                            cond.clone(), 
                            expected_type,
                            true
                        ) else {
                            panic!("line {line}: evaluating condition should yield something");
                        };
    
                        if variable_data.expr_type != ExprType::Bool {
                            panic!("line {line}: encountered non bool type in conditional statement");
                        }
    
                        let ExprData::Literal(Lit::BooleanLiteral(eval_cond)) = (unsafe {
                            self.get_expr_from_variable_data(expr_bump, &variable_data).expr_data}
                        ) else {
                            unreachable!("expr_data should be a boolean");
                        };
    
                        if !eval_cond {
                            continue;
                        }
                        
                        self.variables.push(self.variables(true).new_frame());
                        let output = self.interpret_statements(
                            expr_bump, 
                            body.as_slice(), 
                            expected_type
                        );
                        self.variables.pop();

                        output?;

                        continue 'stmts;
                    }
                    if bodies.len() > conds.len() {
                        self.variables.push(self.variables(true).new_frame());
                        let output = self.interpret_statements(
                            expr_bump, 
                            bodies.last().unwrap().as_slice(), 
                            expected_type
                        );
                        self.variables.pop();

                        output?;
                    }
                },
                Stmt::Return(expr_data_opt) => return match expr_data_opt.as_ref().map(|expr|
                    self.evaluate_expression(expr_bump, expr.clone(), expected_type, true)
                        .expect(format!("line {line}: could not evaluate expression {expr:?}").as_str())
                ).ok_or(()) {
                    Ok(ok) => Err(ok),
                    Err(err) => Ok(err)
                },
                Stmt::Declare(name, stack_location, expr_type) => {
                    let expr_type = expr_type.borrow().clone();
                    self.push_variable(name, expr_type, *stack_location);
                },
                Stmt::Assign { variable, assign } => {
                    let var_name = match variable.clone() {
                        Expr {
                            expr_data: ExprData::Identifier(var_name),
                            ..
                        } => var_name,
                        Expr { line, ..} => panic!("line {line}: assign expression requires an identifer")
                    };

                    let assign_variable_data = self.evaluate_expression(
                        expr_bump, 
                        assign.clone(), 
                        expected_type,
                        true
                    ).expect(format!("line {line}: could not evaluate expression {assign:?}").as_str());

                    let TypedExpr {
                        expr_data: assign_data,
                        expr_type: assign_type
                    } = unsafe {
                        self.get_expr_from_variable_data(expr_bump, &assign_variable_data)
                    };

                    let bytes = self.to_interpreter_bytes(
                        expr_bump, 
                        assign_data, 
                        assign_type, 
                        *line, 
                        expected_type, 
                        true
                    ).expect(format!("line {line}: could not interpret expression {assign:?}").as_str());

                    let variable_data = unsafe { self.variables.last().unwrap_unchecked()
                        .get_in_stack(var_name)
                        .expect(format!("line {line}: could not find variable {var_name}").as_str())
                        .get()
                    };

                    let mut stack = Stack {
                        oxy_stack: self.oxy_stack,
                        oxy_stack_ptr: &mut self.oxy_stack_ptr,
                        gc_stack: self.gc_stack,
                        gc_stack_ptr: &mut self.gc_stack_ptr,
                    };

                    stack.stack_write(variable_data, &bytes);
                },
                Stmt::Expr(expr) => {
                    self.evaluate_expression(expr_bump, expr.clone(), expected_type, true);
                }
            };

        }

        self.oxy_stack_ptr = oxy_stack_ptr_start;
        self.gc_stack_ptr = gc_stack_ptr_start;

        return Ok(());
    }

    pub fn evaluate_expression(
        &mut self,
        expr_bump: &'bumps ExprBump,
        expr: Expr<'bumps, 'defs>,
        expected_type: &ExprType,
        local_scoping: bool
    ) -> Option<VariableData> {
        let line = expr.line;
        match expr.expr_data {
            ExprData::Identifier(ident) => {
                let data = self.variables(local_scoping)
                    .get_in_stack(ident)
                    .expect(format!("could not find variable {ident}").as_str())
                    .get()
                    .clone();
                return Some(data);
            }
            ExprData::Literal(_) => {
                let variable_data = self.stack_alloc(
                    line,
                    &*expr.expr_type.get(), 
                    StackLocation::Oxy
                );
                let bytes = self.to_interpreter_bytes(
                    expr_bump, 
                    expr.expr_data, 
                    &*expr.expr_type.get(),
                    line,
                    expected_type,
                    local_scoping
                ).expect(
                    format!(
                        "line {line}: could not interpret expression {expr:?}"
                    ).as_str()
                );
                self.stack_write(&variable_data, &bytes);
                return Some(variable_data);
            }
            ExprData::Custom { .. } => {
                let variable_data = self.stack_alloc(
                    line,
                    &*expr.expr_type.get(), 
                    StackLocation::Oxy
                );
                let bytes = self.to_interpreter_bytes(
                    expr_bump, 
                    expr.expr_data, 
                    &*expr.expr_type.get(), 
                    line,
                    expected_type,
                    local_scoping
                ).expect(
                    format!(
                        "line {line}: could not interpret expression {expr:?}"
                    ).as_str()
                );
                self.stack_write(&variable_data, &bytes);
                return Some(variable_data);
            }
            ExprData::AnonymousCustom { .. } => {
                let variable_data = self.stack_alloc(
                    line,
                    &*expr.expr_type.get(), 
                    StackLocation::Oxy
                );
                let bytes = self.to_interpreter_bytes(
                    expr_bump, 
                    expr.expr_data, 
                    &*expr.expr_type.get(), 
                    line,
                    expected_type,
                    local_scoping
                ).expect(
                    format!(
                        "line {line}: could not interpret expression {expr:?}"
                    ).as_str()
                );
                self.stack_write(&variable_data, &bytes);
                return Some(variable_data);
            }
            ExprData::CustomField { data, field } => {
                let struct_data = self.evaluate_expression(
                    expr_bump,
                    (*data).clone(), 
                    expected_type,
                    local_scoping
                ).expect("evaluating struct data should yield something");

                if struct_data.expr_type != *data.expr_type.get() {
                    panic!(
                        "struct data of type {:?} does not match with expected type {:?}",
                        struct_data.expr_type,
                        *data.expr_type.get()
                    );
                }

                let ExprType::Custom { ident } = struct_data.expr_type else {
                    panic!("Type of Custom Field expression should never be not a Custom Type");
                };

                let custom_struct = self.defs.iter()
                    .find(|custom_struct| custom_struct.name == *ident)
                    .expect(format!("{ident} Type does not exist").as_str());

                let offset = custom_struct.fields.iter()
                    .scan(0, |accum, struct_field| {
                        let offset = *accum;
                        *accum += struct_field.field_type.size_of(self.defs);
                        return Some((offset, struct_field));
                    })
                    .find_map(|(offset, struct_field)| 
                        (struct_field == *field).then_some(offset)
                    )
                    .expect(format!(
                        "Field {:?} should exist for Struct {:?}", 
                        field.field_name, 
                        custom_struct
                    ).as_str());
                
                let field_data = VariableData {
                    index: struct_data.index + offset,
                    expr_type: field.field_type.clone()
                };

                Some(field_data)
            }
            ExprData::AnonymousCustomField { data, field_name } => {
                let struct_data = self.evaluate_expression(
                    expr_bump,
                    (*data).clone(), 
                    expected_type,
                    local_scoping
                ).expect("evaluating struct data should yield something");

                if struct_data.expr_type != *data.expr_type.get() {
                    panic!(
                        "struct data of type {:?} does not match with expected type {:?}",
                        struct_data.expr_type,
                        *data.expr_type.get()
                    );
                }

                let ExprType::AnonymousCustom { ref fields } = struct_data.expr_type else {
                    panic!("Type of Custom Field expression should never be not a Custom Type");
                };

                let (field_type, offset) = fields.iter()
                    .scan(0, |accum, (field_name, field_type)| {
                        let offset = *accum;
                        *accum += field_type.size_of(self.defs);
                        return Some((offset, field_name, field_type));
                    })
                    .find_map(|(offset, anonymous_field_name, field_type)| 
                        (anonymous_field_name == field_name).then_some((field_type, offset))
                    )
                    .expect(format!(
                        "Field {:?} should exist for Anonymous Struct {:?}", 
                        field_name, 
                        struct_data.expr_type
                    ).as_str());
                
                let field_data = VariableData {
                    index: struct_data.index + offset,
                    expr_type: field_type.clone()
                };

                Some(field_data)
            }
            ExprData::Conditional { conds, bodies } => {
                for i in 0..conds.len() {
                    let cond = self.evaluate_expression(
                        expr_bump,
                        conds[i].clone(), 
                        expected_type,
                        local_scoping
                    ).expect("evaluating condition should yield something");

                    if cond.expr_type != ExprType::Bool {
                        panic!("encountered non bool type in conditional statement");
                    }

                    let ExprData::Literal(Lit::BooleanLiteral(cond)) = ( unsafe {
                        self.get_expr_from_variable_data(expr_bump, &cond).expr_data 
                    }) else {
                        unreachable!("expr_data should be a boolean");
                    };

                    if !cond {
                        continue;
                    }

                    self.interpret_statements(
                        expr_bump, 
                        &[bodies[i]], 
                        expected_type
                    ).err()?;
                }
                
                if let Some(else_expr) = bodies.get(conds.len()) {
                    self.interpret_statements(
                        expr_bump, 
                        &[else_expr], 
                        expected_type
                    ).err()?;
                }
                
                panic!("conditional expression did not cover all possible variants");
            }
            ExprData::Function { name, left_args, right_args } => {
                if let Some(built_in) = BuiltInFunction::from_name(name) && built_in.match_args(&left_args, &right_args) {
                    match built_in {
                        BuiltInFunction::print_string => {
                            let ExprData::Literal(Lit::StringLiteral(print)) = right_args[0].clone().expr_data else {
                                unreachable!("If right args does not match pattern, we would've failed earlier")
                            };

                            print!("{}", unicode_escape::decode(print)
                                .expect(format!("{print} was an invalid string").as_str())
                            );

                            return None;
                        },
                        BuiltInFunction::print_i32 => {
                            let variable_data = self.evaluate_expression(
                                expr_bump, 
                                right_args[0].clone(), 
                                expected_type,
                                local_scoping
                            ).expect(format!(
                                "line {line}: could not interpret expression {:?}", 
                                right_args[0]
                            ).as_str());

                            let expr_data = unsafe {
                                self.get_expr_from_variable_data(expr_bump, &variable_data).expr_data 
                            };
                            
                            let ExprData::Literal(Lit::IntegerLiteral(print)) = expr_data else {
                                unreachable!("If right args does not match pattern, we would've failed earlier")
                            };

                            print!("{print}");

                            return None;
                        },
                        BuiltInFunction::read_char => {
                            let output_type = ExprType::AnonymousCustom { 
                                fields: Box::new([
                                    (String::from("value"), ExprType::Char),
                                    (String::from("success"), ExprType::Bool)
                                ]) 
                            };
                            let read_data = self.stack_alloc(line, &output_type, StackLocation::Oxy);
                            let (value, success) = match try_read!() {
                                Ok(char) => (char, true),
                                Err(_) => (char::default(), false),
                            };

                            let output_data = expr_bump.alloc(ExprData::AnonymousCustom { 
                                fields: Box::new([
                                    (
                                        String::from("value"), 
                                        expr_bump.alloc(ExprData::Literal(Lit::CharLiteral(value)))
                                    ),
                                    (
                                        String::from("success"), 
                                        expr_bump.alloc(ExprData::Literal(Lit::BooleanLiteral(success)))
                                    )
                                ]) 
                            });

                            let bytes = self.to_interpreter_bytes(
                                expr_bump, 
                                output_data, 
                                &output_type, 
                                line, 
                                expected_type, 
                                local_scoping
                            ).expect(
                                format!("line {line}: could not interpret expression {expr:?}").as_str()
                            );

                            self.stack_write(&read_data, &*bytes);

                            return Some(read_data);
                        },
                        BuiltInFunction::read_i32 => {
                            let output_type = ExprType::AnonymousCustom { 
                                fields: Box::new([
                                    (String::from("value"), ExprType::I32),
                                    (String::from("success"), ExprType::Bool)
                                ]) 
                            };
                            let read_data = self.stack_alloc(line, &output_type, StackLocation::Oxy);
                            let (value, success): (i32, bool) = match try_read!() {
                                Ok(char) => (char, true),
                                Err(_) => (i32::default(), false),
                            };

                            let output_data = expr_bump.alloc(ExprData::AnonymousCustom { 
                                fields: Box::new([
                                    (
                                        String::from("value"), 
                                        expr_bump.alloc(ExprData::Literal(Lit::IntegerLiteral(value as i128)))
                                    ),
                                    (
                                        String::from("success"), 
                                        expr_bump.alloc(ExprData::Literal(Lit::BooleanLiteral(success)))
                                    )
                                ]) 
                            });

                            let bytes = self.to_interpreter_bytes(
                                expr_bump, 
                                output_data, 
                                &output_type, 
                                line, 
                                expected_type, 
                                local_scoping
                            ).expect(
                                format!("line {line}: could not interpret expression {expr:?}").as_str()
                            );

                            self.stack_write(&read_data, &*bytes);

                            return Some(read_data);
                        },
                        BuiltInFunction::panic => {
                            let ExprData::Literal(Lit::StringLiteral(panic)) = right_args[0].clone().expr_data else {
                                unreachable!("If right args does not match pattern, we would've failed earlier")
                            };

                            panic!("line {line}: {}", unicode_escape::decode(panic)
                                .expect(format!("{panic} was an invalid string").as_str())
                            );
                        },
                    }
                }

                self.variables.push(StackFrameDictAllocator::new());
                
                let fun = self.functions.iter()
                    .find(|fun| fun.name == *name)
                    .expect(format!("function named {} could not be found", name).as_str());
                
                let oxy_stack_ptr_start = self.oxy_stack_ptr;

                for (arg, input) in fun.left_args.iter().zip(left_args) {
                    let FnParam {
                        param_name,
                        param_type: arg_type,
                        ..
                    } = arg;

                    let variable_data = self.evaluate_expression(
                        expr_bump, 
                        input.clone(), 
                        expected_type,
                        false
                    )?;
                    
                    let arg_name = param_name.as_ref().expect("argument name required");
                    
                    unsafe {
                        let bytes = self.get_bytes_from_index(
                            variable_data.index, 
                            variable_data.expr_type.size_of(self.defs)
                        ).to_vec().into_boxed_slice();

                        self.push_variable(arg_name, arg_type.clone(), StackLocation::Oxy);
                        self.write_variable(arg_name, &bytes);
                    }
                }

                for (arg, input) in fun.right_args.iter().zip(right_args) {
                    let FnParam {
                        param_name,
                        param_type: arg_type,
                        ..
                    } = arg;

                    let variable_data = self.evaluate_expression(
                        expr_bump, 
                        input.clone(), 
                        expected_type,
                        false
                    )?;
                    
                    let arg_name = param_name.as_ref().expect("argument name required");
                    
                    unsafe {
                        let bytes = self.get_bytes_from_index(
                            variable_data.index, 
                            variable_data.expr_type.size_of(self.defs)
                        ).to_vec().into_boxed_slice();

                        self.push_variable(arg_name, arg_type.clone(), StackLocation::Oxy);
                        self.write_variable(arg_name, &bytes);
                    }
                }

                ////println!("calculating {:?}", fun.body);
                let out = self.interpret_statements(expr_bump, &fun.body, &fun.return_type).err();
                self.variables.pop();
                self.oxy_stack_ptr = oxy_stack_ptr_start;

                return out;
            }
            ExprData::BinaryOp(binary_operator, left, right) => {
                return bin_op::evaluate_bin_op(
                    self, 
                    expr_bump, 
                    line, 
                    expr, 
                    *binary_operator, 
                    left, right, 
                    expected_type, 
                    local_scoping
                );
            }
            ExprData::UnaryOp(unary_operator, expr) => {
                return un_op::evaluate_un_op(
                    self, 
                    expr_bump, 
                    line, 
                    *unary_operator, 
                    expr, 
                    expected_type, 
                    local_scoping
                );
            }
            data => panic!("{data:?} is not supported yet"),
        }
    }

    pub fn variables(&self, local_scoping: bool) -> &StackFrameDictAllocator<'i, String, VariableData> {
        if local_scoping {
            return unsafe { self.variables.last().unwrap_unchecked() };
        }
        return unsafe { &self.variables.last_chunk::<2>().unwrap_unchecked()[0] }
    }

    pub fn push_variable(
        &mut self, 
        name: &str, 
        expr_type: ExprType, 
        stack_location: StackLocation
    ) {
        let type_size = expr_type.size_of(self.defs);

        if !expr_type.is_real_type() {
            panic!("{expr_type:?} is not real!!!!");
        }

        match stack_location {
            StackLocation::GC => {
                self.gc_stack_ptr += (type_size - (self.gc_stack_ptr % type_size)) % 
                    type_size;
                if self.gc_stack_ptr + type_size > Self::GC_STACK_SIZE {
                    panic!("gc stack overflow, ptr exceeded size of {} with offset {}", 
                        Self::GC_STACK_SIZE, 
                        self.gc_stack_ptr
                    );
                }
                unsafe {
                    self.gc_stack.as_mut_ptr().add(self.gc_stack_ptr).write_bytes(0, type_size);
                }
                let index = StackIndex::GC(self.gc_stack_ptr);
                let variable_data = VariableData {
                    index,
                    expr_type
                };
                self.variables(true).push(name, variable_data);
                ////println!("declaring variable {name}");
                ////self.variables().print();
                self.gc_stack_ptr += type_size;
            },
            StackLocation::Oxy => {
                self.oxy_stack_ptr += (type_size - (self.oxy_stack_ptr % type_size)) % 
                    type_size;
                if self.oxy_stack_ptr + type_size > Self::OXY_STACK_SIZE {
                    panic!("oxy stack overflow, ptr exceeded size of {} with offset {}", 
                        Self::OXY_STACK_SIZE, 
                        self.oxy_stack_ptr
                    );
                }
                unsafe {
                    self.oxy_stack.as_mut_ptr().add(self.oxy_stack_ptr).write_bytes(0, type_size);
                }
                let index = StackIndex::Oxy(self.oxy_stack_ptr);
                let variable_data = VariableData {
                    index,
                    expr_type
                };
                self.variables(true).push(name, variable_data);
                ////self.variables().print();
                self.oxy_stack_ptr += type_size;
            }
        }
    }

    pub fn write_variable(&mut self, name: &str, bytes: &[u8]) {
        let variable_index = self.variables(true)
            .get_in_stack(name)
            .expect(format!("could not find variable {name}").as_str())
            .get()
            .index;

        let variable_ptr = unsafe { 
            match variable_index {
                StackIndex::GC(index) => self.gc_stack.cast::<u8>().add(index),
                StackIndex::Oxy(index) => self.oxy_stack.cast::<u8>().add(index)
            }
        };

        unsafe {
            for (offset, byte) in bytes.iter().enumerate() {
                variable_ptr.add(offset).write(*byte);
            }
        }
    }

    pub fn stack_alloc(
        &mut self, 
        line: usize, 
        expr_type: &ExprType, 
        stack_location: StackLocation
    ) -> VariableData {
        Stack::stack_alloc(&mut Stack {
            oxy_stack: self.oxy_stack,
            oxy_stack_ptr: &mut self.oxy_stack_ptr,
            gc_stack: self.gc_stack,
            gc_stack_ptr: &mut self.gc_stack_ptr,
        }, line, self.defs, expr_type.clone(), stack_location)
    }

    pub fn stack_write(&mut self, variable_data: &VariableData, bytes: &[u8]) {
        Stack::stack_write(&mut Stack { 
            oxy_stack: self.oxy_stack,
            oxy_stack_ptr: &mut self.oxy_stack_ptr,
            gc_stack: self.gc_stack,
            gc_stack_ptr: &mut self.gc_stack_ptr
        }, variable_data, bytes);
    }

    //TODO refactor Custom ExprData to store Box<ExprData> instead of &ExprData to avoid constant allocations
    pub unsafe fn get_expr_from_variable_data<'types>(
        &self, 
        expr_bump: &'bumps ExprBump,
        variable_data: &'types VariableData
    ) -> TypedExpr<'bumps, 'types, 'defs> {
        let expr_type_size = variable_data.expr_type.size_of(self.defs);

        let expr_data = match &variable_data.expr_type {
            ExprType::AmbiguousType => unreachable!(),
            ExprType::I8 => ExprData::Literal(Lit::IntegerLiteral(i8::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::I16 => ExprData::Literal(Lit::IntegerLiteral(i16::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::I32 => ExprData::Literal(Lit::IntegerLiteral(i32::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::I64 => ExprData::Literal(Lit::IntegerLiteral(i64::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::I128 => ExprData::Literal(Lit::IntegerLiteral(i128::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::ISize => ExprData::Literal(Lit::IntegerLiteral(isize::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }) as i128)),
            ExprType::AmbiguousNegInteger => panic!(),
            ExprType::U8 => ExprData::Literal(Lit::IntegerLiteral(u8::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::U16 => ExprData::Literal(Lit::IntegerLiteral(u16::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::U32 => ExprData::Literal(Lit::IntegerLiteral(u32::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::U64 => ExprData::Literal(Lit::IntegerLiteral(u64::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::U128 => ExprData::Literal(Lit::IntegerLiteral(u128::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }) as i128)),
            ExprType::USize => ExprData::Literal(Lit::IntegerLiteral(usize::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }) as i128)),
            ExprType::AmbiguousPosInteger => panic!(),
            ExprType::F32 => ExprData::Literal(Lit::FloatLiteral(f32::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::F64 => ExprData::Literal(Lit::FloatLiteral(f64::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }).into())),
            ExprType::AmbiguousFloat => panic!(),
            ExprType::Char => ExprData::Literal(Lit::CharLiteral(unsafe {
                char::from_u32_unchecked(u32::from_le_bytes(
                    self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
                ).into())
            })),
            ExprType::StringLiteral => todo!(),
            ExprType::Bool => ExprData::Literal(Lit::BooleanLiteral(u8::from_le_bytes(unsafe {
                self.get_bytes_from_index(variable_data.index, expr_type_size).try_into().unwrap_unchecked()
            }) != 0)),
            ExprType::Ref(_expression_type) => todo!(),
            ExprType::MutRef(_expression_type) => todo!(),
            ExprType::Array { .. } => todo!(),
            ExprType::Tuple { .. } => todo!(),
            ExprType::AmbiguousGroup { .. } => todo!(),
            ExprType::Function { .. } => todo!(),
            ExprType::FunctionPass { .. } => todo!(),
            ExprType::DiscardSingle => todo!(),
            ExprType::DiscardMany => todo!(),
            ExprType::Custom { ident } => {
                let mut fields = HashMap::new();

                let custom_struct = self.defs.iter()
                    .find(|custom_struct| custom_struct.name == *ident)
                    .expect(format!("{ident} Type does not exist").as_str());

                let mut index = variable_data.index;

                for field in &custom_struct.fields {
                    let field_data = VariableData {
                        index,
                        expr_type: field.field_type.clone()
                    };

                    let field_expr = self.get_expr_from_variable_data(
                        expr_bump,
                        &field_data
                    ).expr_data;

                    fields.insert(field.field_name.as_str(), field_expr);

                    index += field.field_type.size_of(self.defs);
                }

                ExprData::Custom { fields }
            },
            ExprType::AnonymousCustom { fields } => {
                let mut anonymous_fields = vec![];
                
                let mut index = variable_data.index;

                for (field_name, field_type) in fields.iter() {
                    let field_data = VariableData {
                        index,
                        expr_type: field_type.clone()
                    };

                    let field_expr = self.get_expr_from_variable_data(
                        expr_bump,
                        &field_data
                    ).expr_data;

                    anonymous_fields.push((field_name.clone(), field_expr));

                    index += field_type.size_of(self.defs);
                }
                
                ExprData::AnonymousCustom { fields: anonymous_fields.into_boxed_slice() }
            }
            ExprType::Void => todo!(),
            ExprType::Never => panic!("Encountered Never Type")
        };

        return TypedExpr {
            expr_data: expr_bump.alloc(expr_data),
            expr_type: &variable_data.expr_type
        }
    }

    //TODO use this more to avoid unneccessary copies and allocations
    pub unsafe fn get_bytes_from_index(&self, index: StackIndex, size: usize) -> &[u8] {
        match index {
            StackIndex::GC(i) => &self.gc_stack.as_ref()[i..i+size],
            StackIndex::Oxy(i) => &self.oxy_stack.as_ref()[i..i+size],
        }
    } 

    pub fn to_interpreter_bytes(
        &mut self, 
        expr_bump: &'bumps ExprBump, 
        expr_data: &'bumps ExprData<'bumps, 'defs>, 
        expr_type: &ExprType,
        line: usize,
        expected_type: &ExprType,
        local_scoping: bool
    ) -> Option<Box<[u8]>> {
        let mut vec: Vec<u8> = vec![];
        
        match (expr_data, expr_type) {
            (ExprData::Literal(Lit::BooleanLiteral(bool)), ExprType::Bool) => {
                vec.push(if *bool {1} else {0});
            },
            (ExprData::Literal(Lit::IntegerLiteral(value)), ExprType::U8 | ExprType::I8) => {
                vec.extend_from_slice(&(*value as u8).to_le_bytes());
            },
            (ExprData::Literal(Lit::IntegerLiteral(value)), ExprType::U16 | ExprType::I16) => {
                //TODO detect the endian of the machine and work off that
                vec.extend_from_slice(&(*value as u16).to_le_bytes());
            },
            (ExprData::Literal(Lit::IntegerLiteral(value)), ExprType::U32 | ExprType::I32) => {
                //TODO detect the endian of the machine and work off that
                vec.extend_from_slice(&(*value as u32).to_le_bytes());
            },
            (ExprData::Literal(Lit::IntegerLiteral(value)), ExprType::U64 | ExprType::I64) => {
                //TODO detect the endian of the machine and work off that
                vec.extend_from_slice(&(*value as u64).to_le_bytes());
            },
            (ExprData::Literal(Lit::IntegerLiteral(value)), ExprType::U128 | ExprType::I128) => {
                //TODO detect the endian of the machine and work off that
                vec.extend_from_slice(&(*value as u128).to_le_bytes());
            },
            (ExprData::Literal(Lit::FloatLiteral(value)), ExprType::F32) => {
                vec.extend_from_slice(&(*value as f32).to_le_bytes());
            },
            (ExprData::Literal(Lit::FloatLiteral(value)), ExprType::F64) => {
                vec.extend_from_slice(&(*value as f64).to_le_bytes());
            },
            (ExprData::Literal(Lit::CharLiteral(value)), ExprType::Char) => {
                let mut bytes = vec![];
                value.encode_utf8(bytes.as_mut_slice());
                vec.extend_from_slice(&bytes);
            },
            (ExprData::Literal(Lit::StringLiteral(value)), ExprType::StringLiteral) => {
                vec.extend_from_slice(&(value.as_ptr().addr().to_le_bytes()));
                vec.extend_from_slice(&(value.len().to_le_bytes()));
            },
            (ExprData::Custom {fields}, ExprType::Custom { ident }) => {
                let custom_struct = self.defs.iter()
                    .find(|custom_struct| custom_struct.name == *ident)
                    .expect(format!("Type {ident} does not exist").as_str());

                for field in &custom_struct.fields {
                    let variable_data = self.evaluate_expression(
                        expr_bump, 
                        Expr {
                            line,
                            expr_data: &*fields[field.field_name.as_str()],
                            expr_type: ExprTypeCons::new(expr_bump, field.field_type.clone())
                        }, 
                        expected_type,
                        local_scoping
                    )?;

                    let TypedExpr {
                        expr_data: field_data,
                        expr_type: field_type
                    } = unsafe {
                        self.get_expr_from_variable_data(expr_bump, &variable_data)
                    };

                    let field_data = expr_bump.alloc(field_data);

                    vec.extend_from_slice(
                        &self.to_interpreter_bytes(
                            expr_bump,
                            field_data,
                            field_type, 
                            line,
                            expected_type,
                            local_scoping
                        )?
                    );
                }
            },
            (
                ExprData::AnonymousCustom { fields: data_fields }, 
                ExprType::AnonymousCustom { fields: type_fields }
            ) => {
                assert_eq!(data_fields.len(), type_fields.len());
                for (
                    (field_data_name, field_data), 
                    (field_type_name, field_type)
                ) in data_fields.iter().zip(type_fields) {
                    assert_eq!(field_data_name, field_type_name);
                    let variable_data = self.evaluate_expression(
                        expr_bump, 
                        Expr {
                            line,
                            expr_data: field_data,
                            expr_type: ExprTypeCons::new(expr_bump, field_type.clone())
                        }, 
                        expected_type, 
                        local_scoping
                    )?;

                    let TypedExpr {
                        expr_data: field_data,
                        expr_type: field_type
                    } = unsafe {
                        self.get_expr_from_variable_data(expr_bump, &variable_data)
                    };

                    let field_data = expr_bump.alloc(field_data);

                    vec.extend_from_slice(
                        &self.to_interpreter_bytes(
                            expr_bump,
                            field_data,
                            field_type, 
                            line,
                            expected_type,
                            local_scoping
                        )?
                    );
                }
            }
            (expr_data, expr_type) => {
                eprintln!("data {:?} of type {:?} is not supported", expr_data, expr_type);
                return None;
            }
        }
    
        return Some(vec.into_boxed_slice());
    }
}

pub struct Stack<'stack> {
    oxy_stack: NonNull<[u8]>,
    oxy_stack_ptr: &'stack mut usize,
    gc_stack: NonNull<[u8]>, 
    gc_stack_ptr: &'stack mut usize, 
}

impl<'stack> Stack<'stack> {
    pub fn stack_alloc(
        &mut self, 
        line: usize,
        defs: &[Struct],
        expr_type: ExprType, 
        stack_location: StackLocation
    ) -> VariableData {
        let type_size = expr_type.size_of(defs);

        if !expr_type.is_real_type() {
            panic!("{expr_type:?} is not real!!!! on line {line}");
        }

        let index;
        match stack_location {
            StackLocation::GC => {
                //TODO add actual garbage collector where everything on stack is a pointer

                //align ptr
                *self.gc_stack_ptr += (type_size - (*self.gc_stack_ptr % type_size)) % 
                    type_size;
                if *self.gc_stack_ptr + type_size > Interpreter::GC_STACK_SIZE {
                    panic!("gc stack overflow, ptr exceeded size of {} with offset {}", 
                        Interpreter::GC_STACK_SIZE, 
                        *self.gc_stack_ptr
                    );
                }
                index = StackIndex::GC(*self.gc_stack_ptr);
                unsafe {
                    self.gc_stack.as_mut_ptr().add(*self.gc_stack_ptr).write_bytes(0, type_size);
                }

                *self.gc_stack_ptr += type_size;
            },
            StackLocation::Oxy => {
                //align ptr
                *self.oxy_stack_ptr += (type_size - (*self.oxy_stack_ptr % type_size)) % 
                    type_size;
                if *self.oxy_stack_ptr + type_size > Interpreter::OXY_STACK_SIZE {
                    panic!("gc stack overflow, ptr exceeded size of {} with offset {}", 
                        Interpreter::OXY_STACK_SIZE, 
                        *self.oxy_stack_ptr
                    );
                }
                index = StackIndex::Oxy(*self.oxy_stack_ptr);
                unsafe {
                    self.oxy_stack.as_mut_ptr().add(*self.oxy_stack_ptr).write_bytes(0, type_size);
                }

                *self.oxy_stack_ptr += type_size;
            }
        }
        
        return VariableData { index, expr_type }
    }

    pub fn stack_write(&mut self, variable_data: &VariableData, bytes: &[u8]) {
        let variable_ptr = unsafe { 
            match variable_data.index {
                StackIndex::GC(index) => self.gc_stack.cast::<u8>().add(index),
                StackIndex::Oxy(index) => self.oxy_stack.cast::<u8>().add(index)
            }
        };

        unsafe {
            for (offset, byte) in bytes.iter().enumerate() {
                variable_ptr.add(offset).write(*byte);
            }
        }
    }
}