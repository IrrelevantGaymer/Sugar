use std::{cell::RefCell, collections::HashMap};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;

use crate::lexer::token::{Keyword, Kwrd, Op, Tkn, TknType, TokenType};

use super::{expr::{self, Expr, ExprData, ExprType, ExprTypeCons, VariableData}, functions::FullFunctionDefinition, tokens, ExprBump, ParserError, StmtBump};

pub type Stmt<'s> = Statement<'s>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'s> {
    Compound(Vec<&'s Stmt<'s>>),
    While(&'s ExprData<'s>, Vec<&'s Stmt<'s>>),
    Conditional{ 
        conds: Vec<&'s ExprData<'s>>, 
        bodies: Vec<Vec<&'s Stmt<'s>>> 
    },
    Return(Option<&'s ExprData<'s>>),
    Declare(String, RefCell<ExprType>),
    Assign{ 
        variable: &'s ExprData<'s>, 
        assign: &'s ExprData<'s>
    },
}

pub fn parse_statement<'ps, 'sfda, 'i>(
    expr_bump: &'ps ExprBump,
    stmt_bump: &'ps StmtBump,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'ps>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>,
    tokens: &'ps [Tkn], 
    index: &mut usize
) -> Result<Vec<&'ps Stmt<'ps>>, ParserError<'ps>> {
    // While(Expr<'s>, &'s Stmt<'s>),
    // Conditional(Vec<Expr<'s>>, Vec<Stmt<'s>>),
    // // Insert(Expr<'s>, Expr<'s>),
    return parse_compound_statement(expr_bump, stmt_bump, functions, variables, tokens, index).map(|e| vec![e])
        .or_else(|_| parse_conditional_statement(expr_bump, stmt_bump, functions, variables, tokens, index).map(|e| vec![e]))
        .or_else(|_| parse_variable_declaration(expr_bump, stmt_bump, functions, &variables, tokens, index))
        .or_else(|_| parse_variable_assignment(expr_bump, stmt_bump, functions, &variables, tokens, index))
        .or_else(|_| parse_return(expr_bump, stmt_bump, functions, &variables, tokens, index).map(|e| vec![e]))
        .map_err(|_| {
            let err = ParserError::new(
                &tokens[*index], 
                "Could not parse statement"
            );
            return err;
        });
}

fn parse_compound_statement<'pcs, 'sfda, 'i>(
    expr_bump: &'pcs ExprBump,
    stmt_bump: &'pcs StmtBump,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pcs>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>,
    tokens: &'pcs [Tkn], 
    index: &mut usize
) -> Result<&'pcs Stmt<'pcs>, ParserError<'pcs>> {
    let mut peek = *index;
    let mut stmts: Vec<&Stmt> = vec![];
    let expect_closing_brace;

    if tokens::is_expected_token(tokens, TokenType::Dollar, &mut peek) {
        expect_closing_brace = false;
    } else if !tokens::is_expected_token(tokens, TokenType::OpenCurlyBrace, &mut peek) {
        return Err(ParserError::new(
            &tokens[*index],
            "Unexpected Token.  Was expecting an '{' or a '$'",
        ));
    } else {
        expect_closing_brace = true;
    }
    
    //variables.print();
    variables.new_scope(|variables| {
        loop {
            let stmt_possible = parse_statement(
                expr_bump, 
                stmt_bump, 
                functions, 
                &variables, 
                tokens, 
                &mut peek
            );
            let Ok(mut stmt) = stmt_possible else {
                break;
            };
            stmts.append(&mut stmt);
        }
        //variables.print();
    });

    if expect_closing_brace {
        tokens::expect_token(tokens, TokenType::CloseCurlyBrace, &mut peek).ok_or(ParserError::new(
            &tokens[peek],
            "Unexpected Token. Was expecting an '}'"
        ))?;
    }

    *index = peek;
    return Ok(stmt_bump.alloc(Stmt::Compound(stmts)));
}

fn parse_variable_declaration<'pvd, 'sfda, 'i>(
    expr_bump: &'pvd ExprBump,
    stmt_bump: &'pvd StmtBump,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pvd>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>, 
    tokens: &'pvd [Tkn], 
    index: &mut usize
) -> Result<Vec<&'pvd Stmt<'pvd>>, ParserError<'pvd>> {
    let mut peek = *index;
    let data_type = match tokens.get(*index).map(|e| &e.token) {
        Some(TknType::Type(data)) => data.to_expr_type(),
        Some(TknType::Keyword(Kwrd::Let)) => ExprType::AmbiguousType,
        Some(TknType::Identifier(data)) => ExprType::Custom { ident: data.to_string() },
        _ => {
            return Err(ParserError::new(
                &tokens[*index],
                "Unexpected Token.  Expected a Type, Identifier, or Let",
            ))
        }
    };
    peek += 1;

    let mutable = tokens::is_expected_token(tokens, TokenType::Keyword(Keyword::Mutable), index);

    let mut idents = vec![];
    let mut stmts: Vec<&Stmt> = vec![];
    while let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token) {
        idents.push(ident.clone());
        //println!("pushing {ident} from {}", &tokens[peek]);
        //variables.print();
        
        stmts.push(stmt_bump.alloc(Stmt::Declare(ident.clone(), RefCell::new(data_type.clone()))));
        let Stmt::Declare(ident, expr_type) = stmts.last().unwrap() else {
            unreachable!("The last statment pushed was of a declare variant");
        };
        variables.push(
            ident.clone(), 
            VariableData::new(mutable, ExprTypeCons::Stored(expr_type, vec![]))
        );
        peek += 1;

        tokens::is_expected_token(tokens, TokenType::Comma, index);
    }

    if idents.len() == 0 {
        return Err(ParserError::new(
            &tokens[*index],
            "Expected identifier(s) to be declared",
        ));
    }

    if tokens::is_expected_token(tokens, TknType::Operation(Op::Assign), &mut peek) {
        let start_expr = peek;
        let Expr {expr_data, expr_type} = expr::parse_expression_set(
            expr_bump, 
            tokens, 
            &mut peek, 
            functions,
            &variables
        )?;

        // if !data_type.match_type(expr_type.clone()) {
        //     return Err(ParserError::new(
        //         &tokens[start_expr],
        //         "Could not match type"
        //     ));
        // }

        for ident in idents {
            ExprTypeCons::new_stored(tokens, &peek, &ident, &variables)?
                .match_type(expr_type.clone())
                .ok_or(ParserError::new(
                    &tokens[start_expr],
                    "Could not match type"
                ))?;
            
            stmts.push(stmt_bump.alloc(Stmt::Assign{
                variable: expr_bump.alloc(ExprData::Identifier(ident.clone())),
                assign: expr_data
            }));
        }
    }

    tokens::expect_token(tokens, TknType::Semicolon, &mut peek).ok_or(ParserError::new(
        &tokens[peek],
        "Expected Semicolon",
    ))?;
    
    *index = peek;
    //println!("finished parsing variable declaration ending at {}", &tokens[peek]);
    return Ok(stmts);
}

fn parse_return<'pr, 'sfda, 'i>(
    expr_bump: &'pr ExprBump,
    stmt_bump: &'pr StmtBump,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pr>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>,
    tokens: &'pr [Tkn], 
    index: &mut usize
) -> Result<&'pr Stmt<'pr>, ParserError<'pr>> {
    let mut peek = *index;
    //println!("parsing return statement at {}", &tokens[peek]);

    tokens::expect_token(tokens, TokenType::Keyword(Kwrd::Return), &mut peek).ok_or(ParserError::new(
        &tokens[peek],
        "Unexpected Token.  Was expecting \"return\""
    ))?;

    let Expr { expr_data, .. } = if tokens::is_token(tokens, TknType::Semicolon, peek) {
        peek += 1;
        *index = peek;
        return Ok(stmt_bump.alloc(Stmt::Return(None)));
    } else {
        //println!("starting parsing return expression at {}", &tokens[peek]);
        expr::parse_expression_set(
            expr_bump, 
            tokens, 
            &mut peek, 
            functions, 
            &variables
        )?
    };

    tokens::expect_token(tokens, TknType::Semicolon, &mut peek).ok_or(ParserError::new(
        &tokens[peek],
        "Expected Semicolon",
    ))?;

    *index = peek;
    //println!("finished parsing return statement ending at {}", &tokens[peek]);
    return Ok(stmt_bump.alloc(Stmt::Return(Some(expr_data))));
}

fn parse_variable_assignment<'pva, 'sfda, 'i>(
    expr_bump: &'pva ExprBump,
    stmt_bump: &'pva StmtBump,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pva>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>,
    tokens: &'pva [Tkn], 
    index: &mut usize
) -> Result<Vec<&'pva Stmt<'pva>>, ParserError<'pva>> {
    let mut peek = *index;
    let mut idents: Vec<(String, usize)> = vec![];

    //println!("parsing variable assignment starting at {}", &tokens[peek]);

    loop {
        if let Some(TknType::Identifier(ident)) = tokens.get(peek).map(|e| &e.token)
            && let Some(TknType::Operation(Op::Assign)) = tokens.get(peek + 1).map(|e| &e.token)
        {
            idents.push((ident.clone(), peek));
            peek += 2;
            continue;
        } else {
            break;
        }
    }

    let start_expr = peek;
    let Expr {
        expr_data,
        expr_type
    } = expr::parse_expression_set(expr_bump, tokens, &mut peek, functions, variables)?;

    let mut stmts: Vec<&Stmt> = vec![];
    for (ident, idx) in idents {
        ExprTypeCons::new_stored(tokens, &start_expr, &ident, variables)?
            .match_type(expr_type.clone())
            .ok_or(ParserError::new(
                &tokens[idx],
                "Something something types"
            ))?;
        
        stmts.push(stmt_bump.alloc(Stmt::Assign{
            variable: expr_bump.alloc(ExprData::Identifier(ident.clone())),
            assign: expr_data,
        }));
    }

    tokens::expect_token(tokens, TknType::Semicolon, &mut peek).ok_or(ParserError::new(
        &tokens[peek],
        "Expected Semicolon",
    ))?;
    *index = peek;
    //println!("finished parsing variable assignment ending at {}", &tokens[peek]);
    return Ok(stmts);
}

pub fn parse_conditional_statement<'pca, 'sfda, 'i>(
    expr_bump: &'pca ExprBump,
    stmt_bump: &'pca StmtBump,
    functions: &'sfda RefCell<HashMap<String, FullFunctionDefinition<'pca>>>,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'i>>,
    tokens: &'pca [Tkn],
    index: &mut usize
) -> Result<&'pca Stmt<'pca>, ParserError<'pca>> {
    let mut peek = *index;
    //println!("parsing conditional statement starting at {}", &tokens[peek]);
    
    tokens::expect_token(tokens, TokenType::Keyword(Kwrd::If), &mut peek).ok_or(ParserError::new(
        &tokens[peek],
        "Unexpected Token.  Was expecting \"if\""
    ))?;

    let mut conds: Vec<&ExprData> = vec![];

    let start_expr = peek;
    let Expr {
        expr_data, 
        expr_type
    }
    ;
    // = expr::parse_expression(expr_bump, tokens, &mut peek, variables, 0)?;

    match expr::parse_expression_set(
        expr_bump, 
        tokens, 
        &mut peek, 
        functions, 
        &variables
    ) {
        Ok(Expr {expr_data: ed, expr_type: et}) => {
            expr_data = ed;
            expr_type = et;
        },
        Err(err) => {
            //eprintln!("{err:?}");
            return Err(err);
        }
    }

    if *expr_type.get().borrow() != ExprType::Bool {
        return Err(ParserError::new(
            &tokens[start_expr],
            "If Statement condition must return a Bool"
        ));
    }

    conds.push(expr_data);

    tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek).ok_or(ParserError::new(
        &tokens[peek],
        "Unexpected Token.  Was expecting '{'."
    ))?;
    
    let mut all_stmts = vec![];

    let mut stmts = vec![];

    //println!("starting conditional statement at {}", &tokens[peek]);
    //variables.print();
    variables.new_scope(|variables| {
        loop {
            let stmt_possible = parse_statement(
                expr_bump, 
                stmt_bump, 
                functions,
                &variables, 
                tokens, 
                &mut peek
            );
            match stmt_possible {
                Ok(mut stmt) => {
                    stmts.append(&mut stmt);
                },
                Err(_) => {
                    break;
                }
            }
        }
        //println!("after conditional statement");
        //variables.print();
    });

    tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek).ok_or(ParserError::new(
        &tokens[peek],
        "Unexpected Token.  Was expecting '}'."
    ))?;

    all_stmts.push(stmts);

    loop {
        //println!("{:?}", tokens::get_token(tokens, peek));
        if !tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::Else), &mut peek) {
            break;
        }
        
        if tokens::is_expected_token(tokens, TknType::Keyword(Kwrd::If), &mut peek) {
            let start_expr = peek;
            let Expr {
                expr_data, 
                expr_type
            } = expr::parse_expression_set(
                expr_bump, 
                tokens, 
                &mut peek, 
                functions, 
                &variables
            )?;

            if *expr_type.get().borrow() != ExprType::Bool {
                return Err(ParserError::new(
                    &tokens[start_expr],
                    "If Statement condition must return a Bool"
                ));
            }
            
            conds.push(expr_data);

            tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek).ok_or(ParserError::new(
                &tokens[peek],
                "Unexpected Token.  Was expecting '{'."    
            ))?;

            let mut stmts = vec![];
            loop {
                let stmt_possible = parse_statement(
                    expr_bump, 
                    stmt_bump, 
                    functions,
                    &variables, 
                    tokens, 
                    &mut peek
                );
                //dbg!(&stmt_possible);
                match stmt_possible {
                    Ok(mut stmt) => {
                        stmts.append(&mut stmt);
                    },
                    Err(_) => {
                        break;
                    }
                }
            }

            tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek).ok_or(ParserError::new(
                &tokens[peek],
                "Unexpected Token.  Was expecting '}'."    
            ))?;

            all_stmts.push(stmts);
            continue;
        }

        tokens::expect_token(tokens, TknType::OpenCurlyBrace, &mut peek).ok_or(ParserError::new(
            &tokens[peek],
            "Unexpected Token.  Was expecting '{'."    
        ))?;

        let mut stmts = vec![];
        loop {
            let stmt_possible = parse_statement(
                expr_bump, 
                stmt_bump, 
                functions, 
                &variables, 
                tokens, 
                &mut peek
            );
            match stmt_possible {
                Ok(mut stmt) => {
                    stmts.append(&mut stmt);
                },
                Err(_) => {
                    break;
                }
            }
        }

        tokens::expect_token(tokens, TknType::CloseCurlyBrace, &mut peek).ok_or(ParserError::new(
            &tokens[peek],
            "Unexpected Token.  Was expecting '}'."    
        ))?;

        all_stmts.push(stmts);
        break;
    }

    *index = peek;
    //println!("finished parsing conditional statement ending at {}", &tokens[peek]);
    return Ok(expr_bump.alloc(Stmt::Conditional { conds, bodies: all_stmts }));
}