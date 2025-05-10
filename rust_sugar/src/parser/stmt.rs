use std::{cell::RefCell, collections::HashMap};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;

use crate::full_result::{FullResult, OptionToFullResult, ResultToFullResult};
use crate::lexer::token::{Kwrd, Op, Tkn, TknType, TokenType};

use super::expr::{Expr, ExprTypeCons};
use super::functions::FullFnDef;
use super::structs::Struct;
use super::{expr::{self, ExprType, VariableData}, patterns, tokens, ExprBump, ParserError, StmtBump};

#[derive(Debug)]
pub struct StmtData<'bumps, 'defs> {
    pub line: usize,
    pub stmt: Stmt<'bumps, 'defs>
}

pub type Stmt<'bumps, 'defs> = Statement<'bumps, 'defs>;

#[derive(Clone, Debug)]
pub enum Statement<'bumps, 'defs> {
    Compound(Vec<&'bumps StmtData<'bumps, 'defs>>),
    While {
        cond: Expr<'bumps, 'defs>, 
        body: Vec<&'bumps StmtData<'bumps, 'defs>>
    },
    Conditional{ 
        conds: Vec<Expr<'bumps, 'defs>>, 
        bodies: Vec<Vec<&'bumps StmtData<'bumps, 'defs>>> 
    },
    Return(Option<Expr<'bumps, 'defs>>),
    Declare(String, StackLocation, &'bumps RefCell<ExprType>),
    Assign{ 
        variable: Expr<'bumps, 'defs>, 
        assign: Expr<'bumps, 'defs>
    },
    Expr(Expr<'bumps, 'defs>)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StackLocation {
    Oxy, GC
}

pub fn parse_statement<'tkns, 'bumps, 'defs, 'fn_defs, 'sfda, 'i>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    functions: &'fn_defs RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'tkns, 'bumps>>,
    tokens: &'tkns [Tkn], 
    expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    Vec<&'bumps StmtData<'bumps, 'defs>>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    // While(Expr<'s>, &'s Stmt<'s>),

    let line = tokens[*index].line_number;
    //println!("parsing statement starting at token {}", &tokens[*index]);
    return parse_compound_statement             (expr_bump, stmt_bump, structs, line, functions, variables, tokens, expected_type, index).map(|e| vec![e])
        .or_else(|_| parse_while_statement      (expr_bump, stmt_bump, structs, line, functions, variables, tokens, expected_type, index).map(|e| vec![e]))
        .or_else(|_| parse_conditional_statement(expr_bump, stmt_bump, structs, line, functions, variables, tokens, expected_type, index).map(|e| vec![e]))
        .or_else(|_| parse_variable_declaration (expr_bump, stmt_bump, structs, line, functions, variables, tokens, expected_type, index)                 )
        .or_else(|_| parse_variable_assignment  (expr_bump, stmt_bump, structs, line, functions, variables, tokens, expected_type, index)                 )
        .or_else(|_| parse_return               (expr_bump, stmt_bump, structs, line, functions, variables, tokens, expected_type, index).map(|e| vec![e]))
        .or_else(|_| parse_expr_statement       (expr_bump, stmt_bump, structs, line, functions, variables, tokens, expected_type, index).map(|e| vec![e]))
        .map_soft_err(|_| ParserError::InvalidStatement { 
            tkn: &tokens[*index] 
        });
}

fn parse_compound_statement<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    tokens: &'tkns [Tkn], 
    expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    &'bumps StmtData<'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = *index;

    //println!("parsing compound statement at {}", &tokens[*index]);

    let mut stmts: Vec<&StmtData> = vec![];
    let expect_closing_brace;
    let mut open_brace = None;

    if tokens::is_expected_token(tokens, TokenType::Dollar, &mut peek) {
        expect_closing_brace = false;
    } else if tokens::is_expected_token(tokens, TokenType::OpenCurlyBrace, &mut peek) {
        expect_closing_brace = true;
        open_brace = Some(&tokens[peek - 1]);
    } else {
        return FullResult::SoftErr(ParserError::ExpectedTokens { 
            tkn: &tokens[peek], 
            received: tokens[peek..].iter().map(|tkn| &tkn.token),
            expected: &[TknType::Dollar, TknType::OpenCurlyBrace] 
        });
    }
    
    //variables.print();
    let mut possible_error = None;
    variables.new_scope(|variables| {
        loop {
            let stmt_possible = parse_statement(
                expr_bump, 
                stmt_bump, 
                structs,
                functions, 
                &variables, 
                tokens, 
                expected_type,
                &mut peek
            );
            match stmt_possible {
                FullResult::Ok(mut stmt) => stmts.append(&mut stmt),
                FullResult::SoftErr(_) => break,
                FullResult::HardErr(err) => {
                    possible_error = Some(err);
                    break;
                }
            }
        }
        //variables.print();
    });

    if expect_closing_brace {
        tokens::expect_token(tokens, TokenType::CloseCurlyBrace, &mut peek)
            .ok_or_else_hard(|| ParserError::ExpectedClosingBrace { 
                tkn: &tokens[peek], 
                open_brace: unsafe { open_brace.unwrap_unchecked() } 
            })?;
    }

    *index = peek;
    return FullResult::Ok(stmt_bump.alloc(StmtData {
        line,
        stmt: Stmt::Compound(stmts)
    }));
}

fn parse_variable_declaration<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>, 
    tokens: &'tkns [Tkn], 
    _expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    Vec<&'bumps StmtData<'bumps, 'defs>>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = *index;

    //println!("parsing variable declaration at {}", &tokens[*index]);

    tokens::expect_token(tokens, TknType::Keyword(Kwrd::Let), &mut peek)
        .ok_or_else_soft(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::Keyword(Kwrd::Let) 
        })?;

    let stack_location = if tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Oxidize), &mut peek) {
        StackLocation::Oxy
    } else {
        StackLocation::GC
    };

    let start_ident = peek;
    let mut ident = patterns::parse_identifier_pattern(tokens, &mut peek).harden()?;
    
    // TODO factor out type parsing
    let data_type = if tokens::is_expected_token(tokens, TknType::Colon, &mut peek) {
        super::get_type(tokens, &mut peek, structs).ok_or_else_hard(|| ParserError::ExpectedType { 
            tkn: &tokens[peek]
        })?
    } else {
        ExprType::AmbiguousType
    };

    let mut stmts = patterns::declare_variable_pattern(
        expr_bump, stmt_bump, 
        variables, 
        &mut ident, 
        stack_location, 
        data_type, 
        tokens, 
        start_ident,
        line
    ).harden()?;//.inspect_err(|e| {dbg!(e);})?;

    if tokens::is_expected_token(tokens, TknType::Operation(Op::Assign), &mut peek) {
        let expr = expr::parse_expression_set(
            expr_bump, 
            structs, 
            tokens, 
            &mut peek, 
            line,
            functions,
            &variables
        ).harden()?;//.inspect_err(|e| {dbg!(e);})?;

        stmts.extend(patterns::assign_variable_pattern(
            expr_bump, stmt_bump, 
            variables, 
            true, &mut ident, 
            expr, 
            tokens, 
            start_ident,
            line
        ).harden()?);//.inspect_err(|e| {dbg!(e);})?);
    }

    tokens::expect_token(tokens, TknType::Semicolon, &mut peek)
        .ok_or_else(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::Semicolon 
        }).harden()?;
        
    *index = peek;
    return FullResult::Ok(stmts);
}

fn parse_return<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    tokens: &'tkns [Tkn], 
    expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    &'bumps StmtData<'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = *index;
    //println!("parsing return statement at {}", &tokens[*index]);

    tokens::expect_token(tokens, TokenType::Keyword(Kwrd::Return), &mut peek)
        .ok_or_else_soft(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::Keyword(Kwrd::Return) 
        })?;

    if tokens::is_token(tokens, TknType::Semicolon, peek) {
        peek += 1;
        *index = peek;
        return FullResult::Ok(stmt_bump.alloc(StmtData {
            line, 
            stmt: Stmt::Return(None)
        }));
    }

    let Expr { expr_data, mut expr_type, .. } = expr::parse_expression_set(
        expr_bump, 
        structs,
        tokens, 
        &mut peek, 
        line,
        functions, 
        &variables
    ).harden()?;

    expr_type.match_type(&mut ExprTypeCons::new(expr_bump, expected_type.clone()));

    tokens::expect_token(tokens, TknType::Semicolon, &mut peek).ok_or_else(|| ParserError::ExpectedToken { 
        tkn: &tokens[peek], 
        expected: TknType::Semicolon 
    }).harden()?;

    *index = peek;
    return FullResult::Ok(stmt_bump.alloc(StmtData {
        line, 
        stmt: Stmt::Return(Some(Expr {line, expr_data, expr_type: expr_type.clone()}))
    }));
}

fn parse_variable_assignment<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    tokens: &'tkns [Tkn], 
    _expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    Vec<&'bumps StmtData<'bumps, 'defs>>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = *index;

    //println!("parsing variable assignment at {}", &tokens[*index]);

    let mut ident = patterns::parse_identifier_pattern(tokens, &mut peek).soften()?;
    tokens::expect_token(tokens, TknType::Operation(Op::Assign), &mut peek)
        .ok_or_else_soft(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::Operation(Op::Assign) 
        })?;
    let expr = expr::parse_expression_set(
        expr_bump, 
        structs, 
        tokens, 
        &mut peek, 
        line, 
        functions, 
        variables
    ).harden()?;

    let stmts = patterns::assign_variable_pattern(
        expr_bump, stmt_bump, variables, 
        false, 
        &mut ident, 
        expr, 
        tokens, *index, line
    ).harden()?;

    tokens::expect_token(tokens, TknType::Semicolon, &mut peek)
        .ok_or_else_hard(|| ParserError::ExpectedToken {
            tkn: &tokens[peek],
            expected: TknType::Semicolon
        })?;
    
    *index = peek;
    return FullResult::Ok(stmts);
}

pub fn parse_while_statement<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    tokens: &'tkns [Tkn],
    expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    &'bumps StmtData<'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = *index;
    
    tokens::expect_token(tokens, TokenType::Keyword(Kwrd::While), &mut peek)
        .ok_or_else_soft(|| ParserError::ExpectedToken {
            tkn: &tokens[peek],
            expected: TknType::Keyword(Kwrd::If)
        })?;

    let start_expr = peek;

    let Expr {expr_data, expr_type, ..} = expr::parse_expression_set(
        expr_bump, 
        structs,
        tokens, 
        &mut peek, 
        line,
        functions, 
        &variables
    ).harden()?;

    if *expr_type.get() != ExprType::Bool {
        return FullResult::HardErr(ParserError::CouldNotMatchType { 
            tkns: &tokens[start_expr..peek], 
            calculated_type: expr_type.clone_inner(), 
            expected_type: ExprType::Bool 
        });
    }

    let cond = Expr {
        line, 
        expr_data, 
        expr_type: expr_type.clone()
    };

    tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)
        .ok_or_else_hard(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::OpenCurlyBrace 
        })?;
    let open_brace = &tokens[peek - 1];

    let mut stmts = vec![];

    //variables.print();
    let mut possible_error = None;
    variables.new_scope(|variables| {
        loop {
            let stmt_possible = parse_statement(
                expr_bump, 
                stmt_bump, 
                structs,
                functions,
                &variables, 
                tokens, 
                expected_type,
                &mut peek
            );
            match stmt_possible {
                FullResult::Ok(mut stmt) => stmts.append(&mut stmt),
                FullResult::SoftErr(_) => break,
                FullResult::HardErr(err) => {
                    possible_error = Some(err);
                    break;
                }
            }
        }
    });
    if let Some(err) = possible_error {
        return FullResult::HardErr(err);
    }

    tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek)
        .ok_or_else_hard(|| ParserError::ExpectedClosingBrace { 
            tkn: &tokens[peek], 
            open_brace 
        })?;

    *index = peek;
    return FullResult::Ok(expr_bump.alloc(StmtData { 
        line, stmt: Stmt::While { cond, body: stmts }
    }));
}

pub fn parse_conditional_statement<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    tokens: &'tkns [Tkn],
    expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    &'bumps StmtData<'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = *index;
    
    tokens::expect_token(tokens, TokenType::Keyword(Kwrd::If), &mut peek)
        .ok_or_else_soft(|| ParserError::ExpectedToken {
            tkn: &tokens[peek],
            expected: TknType::Keyword(Kwrd::If)
        })?;

    let mut conds: Vec<Expr> = vec![];

    let start_expr = peek;

    let Expr {expr_data, expr_type, ..} = expr::parse_expression_set(
        expr_bump, 
        structs,
        tokens, 
        &mut peek, 
        line,
        functions, 
        &variables
    ).harden()?;

    if *expr_type.get() != ExprType::Bool {
        return FullResult::HardErr(ParserError::CouldNotMatchType { 
            tkns: &tokens[start_expr..peek], 
            calculated_type: expr_type.clone_inner(), 
            expected_type: ExprType::Bool 
        });
    }

    conds.push(Expr {line, expr_data, expr_type: expr_type.clone()});

    tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)
        .ok_or_else_hard(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::OpenCurlyBrace 
        })?;
    let open_brace = &tokens[peek - 1];

    let mut all_stmts = vec![];

    let mut stmts = vec![];

    //variables.print();
    let mut possible_error = None;
    variables.new_scope(|variables| {
        loop {
            let stmt_possible = parse_statement(
                expr_bump, 
                stmt_bump, 
                structs,
                functions,
                &variables, 
                tokens, 
                expected_type,
                &mut peek
            );
            match stmt_possible {
                FullResult::Ok(mut stmt) => stmts.append(&mut stmt),
                FullResult::SoftErr(_) => break,
                FullResult::HardErr(err) => {
                    possible_error = Some(err);
                    break;
                }
            }
        }
    });
    if let Some(err) = possible_error {
        return FullResult::HardErr(err);
    }

    tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek)
        .ok_or_else_hard(|| ParserError::ExpectedClosingBrace { 
            tkn: &tokens[peek], 
            open_brace 
        })?;

    all_stmts.push(stmts);

    loop {
        if !tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Else), &mut peek) {
            break;
        }
        
        if tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::If), &mut peek) {
            let start_expr = peek;
            let Expr {
                expr_data, 
                expr_type,
                ..
            } = expr::parse_expression_set(
                expr_bump, 
                structs,
                tokens, 
                &mut peek, 
                line,
                functions, 
                &variables
            ).harden()?;

            if *expr_type.get() != ExprType::Bool {
                return FullResult::HardErr(ParserError::CouldNotMatchType { 
                    tkns: &tokens[start_expr..peek], 
                    calculated_type: expr_type.clone_inner(), 
                    expected_type: ExprType::Bool 
                });
            }
            
            conds.push(Expr {line, expr_data, expr_type: expr_type.clone()});

            tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)
                .ok_or_else_hard(|| ParserError::ExpectedToken { 
                    tkn: &tokens[peek], 
                    expected: TknType::OpenCurlyBrace 
                })?;
            let open_brace = &tokens[peek - 1];

            let mut stmts = vec![];
            loop {
                let stmt_possible = parse_statement(
                    expr_bump, 
                    stmt_bump,
                    structs, 
                    functions,
                    &variables, 
                    tokens, 
                    expected_type,
                    &mut peek
                );
                match stmt_possible {
                    FullResult::Ok(mut stmt) => stmts.append(&mut stmt),
                    FullResult::SoftErr(_) => break,
                    FullResult::HardErr(err) => {
                        possible_error = Some(err);
                        break;
                    }
                }
            }
            if let Some(err) = possible_error {
                return FullResult::HardErr(err);
            }

            tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek)
                .ok_or_else_hard(|| ParserError::ExpectedClosingBrace { 
                    tkn: &tokens[peek], 
                    open_brace 
                })?;

            all_stmts.push(stmts);
            continue;
        }

        tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek)
            .ok_or_else_hard(|| ParserError::ExpectedToken { 
                tkn: &tokens[peek], 
                expected: TknType::OpenCurlyBrace 
            })?;
        let open_brace = &tokens[peek - 1];

        let mut stmts = vec![];
        loop {
            let stmt_possible = parse_statement(
                expr_bump, 
                stmt_bump, 
                structs,
                functions, 
                &variables, 
                tokens, 
                expected_type,
                &mut peek
            );
            match stmt_possible {
                FullResult::Ok(mut stmt) => stmts.append(&mut stmt),
                FullResult::SoftErr(_) => break,
                FullResult::HardErr(err) => {
                    possible_error = Some(err);
                    break;
                }
            }
        }
        if let Some(err) = possible_error {
            return FullResult::HardErr(err);
        }

        tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek)
            .ok_or_else_hard(|| ParserError::ExpectedClosingBrace { 
                tkn: &tokens[peek], 
                open_brace 
            })?;

        all_stmts.push(stmts);
        break;
    }

    *index = peek;
    return FullResult::Ok(expr_bump.alloc(StmtData { 
        line, stmt: Stmt::Conditional { conds, bodies: all_stmts }
    }));
}

pub fn parse_expr_statement<'tkns, 'bumps, 'defs>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    structs: &'defs [Struct],
    line: usize,
    functions: &RefCell<HashMap<String, FullFnDef<'tkns, 'bumps, 'defs>>>,
    variables: &StackFrameDictAllocator<'_, String, VariableData<'tkns, 'bumps>>,
    tokens: &'tkns [Tkn],
    _expected_type: &mut ExprType,
    index: &mut usize
) -> FullResult<
    &'bumps StmtData<'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut peek = *index;

    let stmt = stmt_bump.alloc(StmtData {
        line, 
        stmt: Stmt::Expr(
            expr::parse_expression_set(
                expr_bump, 
                structs, 
                tokens, 
                &mut peek, 
                line, 
                functions, 
                variables
            ).soften()?
        )
    });

    tokens::expect_token(tokens, TknType::Semicolon, &mut peek)
        .ok_or_else_hard(|| ParserError::ExpectedToken { 
            tkn: &tokens[peek], 
            expected: TknType::Semicolon 
        })?;

    *index = peek;
    return FullResult::Ok(stmt);
}