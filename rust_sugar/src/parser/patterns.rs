use std::{cell::RefCell, ptr::{self}};

use stack_frame_allocators::stack_frame_dict_allocator::StackFrameDictAllocator;

use crate::{lexer::token::{Kwrd, Tkn, TknType}, parser::tokens};

use super::{expr::{Expr, ExprData, ExprType, ExprTypeCons, VariableData}, stmt::{StackLocation, Stmt, StmtData}, ExprBump, ParserError, StmtBump};

#[derive(Clone, Debug)]
pub enum Pattern<'tkns> {
    Ident { tkn: &'tkns Tkn, mutable: bool, name: String },
    Tuple {
        start: Vec<Self>,
        end: Vec<Self>
    },
    Array {
        start: Vec<Self>, 
        end: Vec<Self>
    },
    AmbiguousGroup {
        start: Vec<Self>,
        end: Vec<Self>
    },
    DiscardSingle
    //TODO enum and struct pattern matching
}

pub fn parse_identifier_pattern<'tkns, 'bumps, 'defs>(
    tokens: &'tkns [Tkn], 
    index: &mut usize
) -> Result<Pattern<'tkns>, ParserError<'tkns, 'bumps, 'defs>> {
    let mut peek = *index;

    #[derive(PartialEq)]
    enum GroupPatternParsingStage {
        Start, End
    }

    if let Some(TknType::Keyword(Kwrd::Mutable)) = tokens::get_token(tokens, peek).map(|e| &e.token) &&
       let Some(TknType::Identifier(ident)) = tokens::get_token(tokens, peek + 1).map(|e| &e.token) 
    {
        let tkn = &tokens[*index];
        *index += 2;

        return Ok(Pattern::Ident{ tkn, mutable: true, name: ident.clone() });
    } else if let Some(TknType::Identifier(ident)) = tokens::get_token(tokens, peek).map(|e| &e.token) {
        let tkn = &tokens[*index];
        *index += 1;

        return Ok(Pattern::Ident{ tkn, mutable: false, name: ident.clone() });
    } else if tokens::is_expected_token(tokens, TknType::OpenParen, &mut peek) {
        let mut start_idents = vec![];
        let mut end_idents = vec![];

        let mut stage = GroupPatternParsingStage::Start;
        let mut discard_many = None;

        loop {
            if tokens::is_expected_token(tokens, TknType::CloseParen, &mut peek) {
                break;
            } else if tokens::is_expected_token(tokens, TknType::DiscardMany, &mut peek) {
                if stage == GroupPatternParsingStage::Start {
                    stage = GroupPatternParsingStage::End;
                    discard_many = Some(&tokens[peek - 1]);
                    continue;
                }

                return Err(ParserError::SecondDiscardMany { 
                    tkn: &tokens[peek - 1], 
                    first_discard_many: unsafe { discard_many.unwrap_unchecked() }
                });
            }

            let ident = parse_identifier_pattern(tokens, &mut peek)?;
            match stage {
                GroupPatternParsingStage::Start => start_idents.push(ident),
                GroupPatternParsingStage::End => end_idents.push(ident),
            }
        }

        return Ok(Pattern::Tuple { 
            start: start_idents, 
            end: end_idents 
        });
    } else if tokens::is_expected_token(tokens, TknType::OpenSquareBracket, &mut peek) {
        let mut start_idents = vec![];
        let mut end_idents = vec![];
        
        let mut stage = GroupPatternParsingStage::Start;
        let mut discard_many = None;
        
        loop {
            if tokens::is_expected_token(tokens, TknType::CloseSquareBracket, &mut peek) {
                break;
            } else if tokens::is_expected_token(tokens, TknType::DiscardMany, &mut peek) {
                if stage == GroupPatternParsingStage::Start {
                    stage = GroupPatternParsingStage::End;
                    discard_many = Some(&tokens[peek - 1]);
                    continue;
                }
                
                return Err(ParserError::SecondDiscardMany { 
                    tkn: &tokens[peek - 1], 
                    first_discard_many: unsafe { discard_many.unwrap_unchecked() } 
                });
            }

            let ident = parse_identifier_pattern(tokens, &mut peek)?;
            match stage {
                GroupPatternParsingStage::Start => start_idents.push(ident),
                GroupPatternParsingStage::End => end_idents.push(ident),
            }
        }

        return Ok(Pattern::Array {
            start: start_idents, 
            end: end_idents
        });
    } else if tokens::is_expected_token(tokens, TknType::DiscardSingle, &mut peek) {
        return Ok(Pattern::DiscardSingle);
    } else if tokens::is_expected_token(tokens, TknType::Dollar, &mut peek) {
        todo!("dollar not implemented yet for pattern matching");
    }

    return Err(ParserError::InvalidPattern { tkn: &tokens[peek] });
}

pub fn declare_variable_pattern<'tkns, 'bumps, 'defs, 'sfda, 'i>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'tkns, 'bumps>>,
    ident: &mut Pattern<'tkns>,
    stack_location: StackLocation,
    mut expr_type: ExprType,
    tokens: &'tkns [Tkn],
    index: usize,
    line: usize
) -> Result<
    Vec<&'bumps StmtData<'bumps, 'defs>>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut stmts: Vec<&StmtData> = vec![];
    
    match ident {
        Pattern::Ident {tkn, mutable, name} => {
            let expr_type = expr_bump.alloc(RefCell::new(expr_type));
            
            stmts.push(stmt_bump.alloc(StmtData {
                line,
                stmt: Stmt::Declare(
                    name.clone(), 
                    stack_location, 
                    expr_type
                )
            }));

            let StmtData {stmt: Stmt::Declare(name, _, expr_type), ..} = stmts.last().unwrap() else {
                unreachable!();
            };
            
            variables.push(
                name, 
                VariableData::new(
                    tkn,
                    *mutable, 
                    ExprTypeCons::new_stored(expr_type)
                )
            );
        },
        Pattern::Tuple { start, end } => {
            let (start_expr_types, end_expr_types);
            if expr_type == ExprType::AmbiguousType {
                expr_type = ExprType::Tuple { 
                    start: vec![ExprType::AmbiguousType; start.len()],
                    end: vec![ExprType::AmbiguousType; end.len()]
                };
                let ExprType::Tuple { 
                    start: start_e_types, 
                    end: end_e_types 
                } = expr_type else {
                    unreachable!();
                };
                (start_expr_types, end_expr_types) = (start_e_types.into_boxed_slice(), end_e_types.into_boxed_slice());
            // Actual Tuple types outside of patterns, cannot use the DiscardMany type
            } else if let ExprType::Tuple { start: ref mut group_types, end: _ } = expr_type {
                if start.len() + end.len() > group_types.len() {
                    return Err(ParserError::PatternNotMatchExpectedType { 
                        tkn: &tokens[index], 
                        pattern: ident.clone(), 
                        expected_type: expr_type 
                    });
                }

                // let (start_group_types, end_group) = group_types.split_at_mut(start.len());
                // let (_, end_group_types) = end_group.split_at_mut(end_group.len() - end.len());
                // (start_expr_types, end_expr_types) = (start_group_types, end_group_types);
                (start_expr_types, end_expr_types) = unsafe {
                    let len = group_types.len();
                    let ptr = group_types.as_mut_ptr();

                    (
                        Box::from_raw(ptr::slice_from_raw_parts_mut(ptr, start.len())), 
                        Box::from_raw(ptr::slice_from_raw_parts_mut(ptr.add(start.len()), len - start.len()))
                    )
                };
            } else {
                return Err(ParserError::PatternNotMatchExpectedType { 
                    tkn: &tokens[index], 
                    pattern: ident.clone(), 
                    expected_type: expr_type 
                });
            }

            for (ident, expr_type) in start.into_iter().zip(start_expr_types.to_vec().into_iter())
                .chain(end.into_iter().zip(end_expr_types.to_vec().into_iter())) 
            {
                
                stmts.extend(declare_variable_pattern(
                    expr_bump, stmt_bump, 
                    variables, 
                    ident, 
                    stack_location, 
                    expr_type, 
                    tokens, 
                    index,
                    line
                )?);
            }
        },
        Pattern::Array { start, end } => {
            let array_type;
            if expr_type == ExprType::AmbiguousType {
                expr_type = ExprType::Array { length: None, expr_type: Box::new(ExprType::AmbiguousType) };
                let ExprType::Array { expr_type: a_type, .. } = expr_type else {
                    unreachable!();
                };
                array_type = a_type;
            } else if let ExprType::Array { length, expr_type: ref a_type} = expr_type && 
                (length.is_some_and(|l| l >= start.len() + end.len()) || length.is_none())
            {
                array_type = a_type.clone();
            } else {
                return Err(ParserError::PatternNotMatchExpectedType { 
                    tkn: &tokens[index], 
                    pattern: ident.clone(), 
                    expected_type: expr_type.clone() 
                });
            }

            for ident in start.into_iter().chain(end) {
                stmts.extend(declare_variable_pattern(
                    expr_bump, stmt_bump, 
                    variables, 
                    ident, 
                    stack_location, 
                    *array_type.clone(), 
                    tokens, 
                    index,
                    line
                )?);
            } 
        },
        Pattern::AmbiguousGroup { .. } => todo!("not implemented yet"),
        Pattern::DiscardSingle => ()
    }

    return Ok(stmts);
}

pub fn assign_variable_pattern<'tkns, 'bumps, 'defs, 'sfda, 'i>(
    expr_bump: &'bumps ExprBump,
    stmt_bump: &'bumps StmtBump,
    variables: &'sfda StackFrameDictAllocator<'i, String, VariableData<'tkns, 'bumps>>,
    declaration: bool,
    ident: &mut Pattern<'tkns>,
    expr: Expr<'bumps, 'defs>,
    tokens: &'tkns [Tkn],
    index: usize,
    line: usize
) -> Result<
    Vec<&'bumps StmtData<'bumps, 'defs>>, 
    ParserError<'tkns, 'bumps, 'defs>
> {
    let mut stmts: Vec<&StmtData> = vec![];

    let Expr { expr_data, mut expr_type, .. } = expr;

    match ident {
        Pattern::Ident { mutable, name, .. } => {
            let VariableData {
                tkn: ident_tkn,
                mutable: ident_mut,
                expr_type: ident_type
            } = variables.get_in_stack(&*name).ok_or_else(|| ParserError::VariableDoesNotExist { 
                tkn: &tokens[index]
            })?.get();

            // TODO refactor declaration to account for branched assignments of immutable variables
            if !declaration {
                if *mutable {
                    return Err(ParserError::InvalidMut { tkn: &tokens[index] });
                }
                if !ident_mut {
                    return Err(ParserError::CannotMutateImmutable { 
                        tkn: &tokens[index], 
                        variable_def: ident_tkn 
                    });
                }
            }

            let expr_type = ident_type.clone().match_type(&mut expr_type)
                .ok_or_else(|| ParserError::CouldNotMatchType { 
                    tkns: core::slice::from_ref(&tokens[index]), 
                    calculated_type: ident_type.clone_inner(), 
                    expected_type: expr_type.clone_inner() 
                })?;

            stmts.push(stmt_bump.alloc(StmtData {
                line,
                stmt: Stmt::Assign {
                    variable: Expr {
                        line,
                        expr_data: expr_bump.alloc(ExprData::Identifier(name.clone())),
                        expr_type: expr_type.clone()
                    },
                    assign: Expr {
                        line,
                        expr_data,
                        expr_type: expr_type.clone()
                    }
                }
            }));
        },
        Pattern::Tuple { start: _, end: _ } => {
            todo!("not implemented yet");
        },
        Pattern::Array { start: _, end: _ } => {
            todo!("not implemented yet");
        },
        Pattern::AmbiguousGroup { .. } => todo!("not implemented yet"),
        Pattern::DiscardSingle => ()
    };
    
    return Ok(stmts);
}