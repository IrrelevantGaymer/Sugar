use crate::{lexer::token::{Tkn, TknType}, term};

use super::{expr::{Expr, ExprType}, functions::{FnType, FullFunctionDefinition}, patterns::Pattern, structs::{Field, Struct}};

type TknTypeFromTkn = fn(&Tkn) -> &TknType;

#[derive(Debug)]
pub enum ParserError<'tkns, 'bumps, 'defs> {
    ExpectedToken { tkn: &'tkns Tkn, expected: TknType },
    /// We accept tkn 
    ExpectedTokens { 
        tkn: &'tkns Tkn, 
        received: std::iter::Map<core::slice::Iter<'tkns, Tkn>, TknTypeFromTkn>,  
        expected: &'static [TknType], 
    },
    AlreadyDefinedWhitelist { tkn: &'tkns Tkn, defined_whitelist: &'tkns Tkn },
    AlreadyDefinedBlacklist { tkn: &'tkns Tkn, defined_blacklist: &'tkns Tkn },
    ExpectedClosingBrace { tkn: &'tkns Tkn, open_brace: &'tkns Tkn },
    ExpectedEndOfWhitelist { tkn: &'tkns Tkn },
    ExpectedEndOfBlacklist { tkn: &'tkns Tkn },
    NoWhitelistOrBlacklist { tkn: &'tkns Tkn },
    MissingAccessor { tkn: &'tkns Tkn },
    ExpectedEndOfStruct { tkn: &'tkns Tkn },
    ExpectedIdentifier { tkn: &'tkns Tkn },
    ExpectedType { tkn: &'tkns Tkn },
    InvalidStatement { tkn: &'tkns Tkn },
    CouldNotMatchType { 
        tkns: &'tkns [Tkn], 
        calculated_type: ExprType, 
        expected_type: ExprType 
    },
    SecondDiscardMany { tkn: &'tkns Tkn, first_discard_many: &'tkns Tkn },
    InvalidPattern { tkn: &'tkns Tkn },
    PatternNotMatchExpectedType { 
        tkn: &'tkns Tkn, 
        pattern: Pattern<'tkns>, 
        expected_type: ExprType 
    },
    VariableDoesNotExist { tkn: &'tkns Tkn },
    InvalidMut { tkn: &'tkns Tkn },
    //TODO add token to variable initialization for better error reporting
    CannotMutateImmutable { tkn: &'tkns Tkn, variable_def: &'tkns Tkn },
    IncorrectNumberPrefixArguments { 
        tkn: &'tkns Tkn, 
        args: Box<[ExprType]>, 
        expected_args: Box<[ExprType]>, 
        function: FullFunctionDefinition<'tkns, 'bumps, 'defs>
    },
    /// When a struct/accessor/function could not be parsed
    InvalidBlock { tkn: &'tkns Tkn },
    MultipleExpressions { tkn: &'tkns Tkn, expr: Box<[Expr<'bumps, 'defs>]>},
    FieldDoesNotExist { tkn: &'tkns Tkn, custom_struct: &'defs Struct },
    AlreadyDefinedField { tkn: &'tkns Tkn, defined_field: &'tkns Tkn },
    FieldExpressionNotDefined { tkn: &'tkns Tkn, field: &'defs Field },
    InvalidExpressionAtom { tkn: &'tkns Tkn },
    InvalidDollarExpression { tkn: &'tkns Tkn },
    InvalidDotExpression { tkn: &'tkns Tkn, expr_type: ExprType },
    AccessorNotDefined { tkn: &'tkns Tkn },
    DefinedIncorrectlyPlacedArgument { 
        tkn: &'tkns Tkn, 
        arg_type: FnType, 
        fix_defined: &'tkns Tkn, 
        fix_type: FnType 
    },
    ConflictingFunctionFixDefinitions { tkn: &'tkns Tkn, fix_defined: &'tkns Tkn },
    ExpectedEndOfFunctionDefinition { tkn: &'tkns Tkn }
}

impl<'tkns, 'exprs, 'defs> ParserError<'tkns, 'exprs, 'defs> {
    pub fn write(&self, f: &mut impl std::io::Write, src: &str) -> std::io::Result<()> {
        use ParserError as PE;
        match self {
            PE::ExpectedToken { 
                tkn: Tkn { 
                    token, 
                    file_name, 
                    line_index, 
                    line_number 
                }, 
                expected 
            } => write!(f, 
                "{red}error:{clear} Expected {expected} but received {token}\n\
                {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                {blue}{pad} | {clear}\n\
                {blue}{line_number} | {clear}{line_of_code}\n\
                {blue}{pad} | {clear}{arrow_pad}{yellow}{arrow}{clear}\n\
                {blue}{pad}:::{clear}\n\
                \n\
                ",
                pad = " ".repeat(line_number.to_string().len()),
                line_of_code = get_line_from_contents(*line_number, src).trim_end(),
                arrow_pad = " ".repeat(*line_index - 1),
                arrow = "^".repeat(token.len()),

                red = term::HIGH_RED_ANSI,
                yellow = term::HIGH_YELLOW_ANSI,
                blue = term::HIGH_BOLD_BLUE_ANSI,
                clear = term::CLEAR_ANSI,
            ),
            PE::ExpectedTokens { 
                tkn: Tkn {
                    token,
                    file_name,
                    line_index,
                    line_number,
                }, 
                received, 
                expected 
            } => write!(f, 
                "{red}error:{clear} Expected {expected} but received {received}\n\
                {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                {blue}{pad} | {clear}\n\
                {blue}{line_number} | {clear}{line_of_code}\n\
                {blue}{pad} | {clear}{arrow_pad}{yellow}{arrow}{clear}\n\
                {blue}{pad}:::{clear}\n\
                \n\
                ",
                expected = slice_to_string(expected),
                received = iter_to_string(&received.clone().take(expected.len())),
                pad = " ".repeat(line_number.to_string().len()),
                line_of_code = get_line_from_contents(*line_number, src).trim_end(),
                arrow_pad = " ".repeat(*line_index - 1),
                arrow = "^".repeat(token.len()),

                red = term::HIGH_RED_ANSI,
                yellow = term::HIGH_YELLOW_ANSI,
                blue = term::HIGH_BOLD_BLUE_ANSI,
                clear = term::CLEAR_ANSI,
            ),
            PE::AlreadyDefinedWhitelist { 
                tkn: Tkn { 
                    token, file_name, line_index, line_number 
                }, 
                defined_whitelist: defined 
            } | PE::AlreadyDefinedBlacklist { 
                tkn: Tkn { 
                    token, file_name, line_index, line_number 
                }, 
                defined_blacklist: defined 
            } => if *line_number == defined.line_number {
                let line_pad_len = defined.line_index - 1;
                let line_len = defined.token.len();
                let arrow_pad_len = *line_index - line_len - line_pad_len - 1;
                let arrow_len = token.len();
                
                write!(f,
                    "{red}error:{clear} whitelist for accessor is already defined\n\
                    {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                    {blue}{pad} | {clear}\n\
                    {blue}{line_number} | {clear}{line_of_code}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}{line}{clear}\
                                         {arrow_pad}{red}{arrow} used more than once {clear}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}|{clear}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}first declared here{clear}\n\
                    {blue}{pad}:::{clear}\n\
                    \n\
                    ",
                    pad = " ".repeat(line_number.to_string().len()),
                    line_of_code = get_line_from_contents(*line_number, src).trim_end(),
                    line_pad = " ".repeat(line_pad_len),
                    line = "-".repeat(line_len),
                    arrow_pad = " ".repeat(arrow_pad_len),
                    arrow = "^".repeat(arrow_len),

                    red = term::HIGH_RED_ANSI,
                    blue = term::HIGH_BOLD_BLUE_ANSI,
                    clear = term::CLEAR_ANSI
                )
            } else {
                let pad_len = core::cmp::Ord::max(
                    line_number.to_string().len(),
                    defined.line_number.to_string().len()
                );

                write!(f,
                    "{red}error:{clear} whitelist for accessor is already defined\n\
                    {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                    {blue}{pad} | {clear}\n\
                    {blue}{line_number_1:>pad_len$} | {clear}{line_of_code_1}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}{line} first declared here {clear}\n\
                    {blue}{line_number_2:>pad_len$} | {clear}{line_of_code_2}\n\
                    {blue}{pad} | {clear}{arrow_pad}{red}{arrow} used more than once {clear}\n\
                    {blue}{pad}:::{clear}\n\
                    \n\
                    ",

                    pad = " ".repeat(line_number.to_string().len()),
                    line_number_1 = defined.line_number,
                    line_number_2 = *line_number,
                    line_of_code_1 = get_line_from_contents(defined.line_number, src).trim_end(),
                    line_of_code_2 = get_line_from_contents(*line_number, src).trim_end(),
                    line_pad = " ".repeat(defined.line_index - 1),
                    arrow_pad = " ".repeat(*line_index - 1),
                    line = "-".repeat(defined.token.len()),
                    arrow = "^".repeat(token.len()),

                    red = term::HIGH_RED_ANSI,
                    blue = term::HIGH_BOLD_BLUE_ANSI,
                    clear = term::CLEAR_ANSI
                )
            },
            PE::ExpectedClosingBrace { 
                tkn: Tkn {
                    token,
                    file_name,
                    line_index,
                    line_number,
                }, open_brace
            } => if *line_number == open_brace.line_number {
                let line_pad_len = open_brace.line_index - 1;
                let line_len = open_brace.token.len();
                let arrow_pad_len = *line_index - line_len - line_pad_len - 1;
                let arrow_len = token.len();
                
                write!(f,
                    "{red}error:{clear} expected closed curly brace '}}' but received {token}\n\
                    {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                    {blue}{pad} | {clear}\n\
                    {blue}{line_number} | {clear}{line_of_code}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}{line}{clear}\
                                         {arrow_pad}{red}{arrow} expected close brace before here {clear}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}|{clear}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}open brace here{clear}\n\
                    {blue}{pad}:::{clear}\n\
                    \n\
                    ",
                    pad = " ".repeat(line_number.to_string().len()),
                    line_of_code = get_line_from_contents(*line_number, src).trim_end(),
                    line_pad = " ".repeat(line_pad_len),
                    line = "-".repeat(line_len),
                    arrow_pad = " ".repeat(arrow_pad_len),
                    arrow = "^".repeat(arrow_len),

                    red = term::HIGH_RED_ANSI,
                    blue = term::HIGH_BOLD_BLUE_ANSI,
                    clear = term::CLEAR_ANSI
                )
            } else {
                let pad_len = core::cmp::Ord::max(
                    line_number.to_string().len(),
                    open_brace.line_number.to_string().len()
                );
                
                write!(f,
                    "{red}error:{clear} expected closed curly brace '}}' but received {token}\n\
                    {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                    {blue}{pad} | {clear}\n\
                    {blue}{line_number_1:>pad_len$} | {clear}{line_of_code_1}\n\
                    {blue}{pad} | {clear}{line_pad}{blue}{line} open brace here {clear}\n\
                    {blue}{line_number_2:>pad_len$} | {clear}{line_of_code_2}\n\
                    {blue}{pad} | {clear}{arrow_pad}{red}{arrow} expected close brace before here {clear}\n\
                    {blue}{pad}:::{clear}\n\
                    \n\
                    ",

                    pad = " ".repeat(line_number.to_string().len()),
                    line_number_1 = open_brace.line_number,
                    line_number_2 = *line_number,
                    line_of_code_1 = get_line_from_contents(open_brace.line_number, src).trim_end(),
                    line_of_code_2 = get_line_from_contents(*line_number, src).trim_end(),
                    line_pad = " ".repeat(open_brace.line_index - 1),
                    arrow_pad = " ".repeat(*line_index - 1),
                    line = "-".repeat(open_brace.token.len()),
                    arrow = "^".repeat(token.len()),

                    red = term::HIGH_RED_ANSI,
                    blue = term::HIGH_BOLD_BLUE_ANSI,
                    clear = term::CLEAR_ANSI
                )
            },
            PE::ExpectedEndOfWhitelist { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::ExpectedEndOfBlacklist { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::NoWhitelistOrBlacklist { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::MissingAccessor { 
                tkn: Tkn {
                    token,
                    file_name,
                    line_index,
                    line_number,
                } 
            } => write!(f, 
                "{red}error:{clear} Expected accessor like pub, prv, pkg, or a custom defined one but received {token}\n\
                {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                {blue}{pad} | {clear}\n\
                {blue}{line_number} | {clear}{line_of_code}\n\
                {blue}{pad} | {clear}{arrow_pad}{yellow}{arrow}{clear}\n\
                {blue}{pad}:::{clear}\n\
                \n\
                ",
                pad = " ".repeat(line_number.to_string().len()),
                line_of_code = get_line_from_contents(*line_number, src).trim_end(),
                arrow_pad = " ".repeat(*line_index - 1),
                arrow = "^".repeat(token.len()),

                red = term::HIGH_RED_ANSI,
                yellow = term::HIGH_YELLOW_ANSI,
                blue = term::HIGH_BOLD_BLUE_ANSI,
                clear = term::CLEAR_ANSI,
            ),
            PE::ExpectedEndOfStruct { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::ExpectedIdentifier { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::ExpectedType { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::InvalidStatement { 
                tkn: Tkn {
                    token,
                    file_name,
                    line_index,
                    line_number,
                }
            } => write!(f, 
                "{red}error:{clear} Expected beginning of accessor, struct, or function definition but received {token}\n\
                {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                {blue}{pad} | {clear}\n\
                {blue}{line_number} | {clear}{line_of_code}\n\
                {blue}{pad} | {clear}{arrow_pad}{red}{arrow}{clear}\n\
                {blue}{pad}:::{clear} note = support for other constructs such as const declarations, traits, directives, etc. will be implemented in the future\n\
                \n\
                ",
                pad = " ".repeat(line_number.to_string().len()),
                line_of_code = get_line_from_contents(*line_number, src).trim_end(),
                arrow_pad = " ".repeat(*line_index - 1),
                arrow = "^".repeat(token.len()),

                red = term::HIGH_RED_ANSI,
                blue = term::HIGH_BOLD_BLUE_ANSI,
                clear = term::CLEAR_ANSI,
            ),
            PE::CouldNotMatchType { 
                tkns, 
                calculated_type, 
                expected_type 
            } => {
                let Some((first_tkn, last_tkn)) = tkns.first().zip(tkns.last()) else {
                    panic!("this will only fail if there are no tokens passed");
                };
                if first_tkn.line_number == last_tkn.line_number {
                    let total_expr_len = last_tkn.line_index - first_tkn.line_index + 
                        last_tkn.token.len();
                    let Tkn {
                        token: _,
                        file_name,
                        line_index,
                        line_number,
                    } = first_tkn;
                    
                    write!(f, 
                        "{red}error:{clear} Mismatched types\n\
                        {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                        {blue}{pad} | {clear}\n\
                        {blue}{line_number} | {clear}{line_of_code}\n\
                        {blue}{pad} | {clear}{arrow_pad}{red}{arrow} expected {expected_type}, found {calculated_type} {clear}\n\
                        {blue}{pad}:::{clear} \n\
                        \n\
                        ",
                        pad = " ".repeat(line_number.to_string().len()),
                        line_of_code = get_line_from_contents(*line_number, src).trim_end(),
                        arrow_pad = " ".repeat(*line_index - 1),
                        arrow = "^".repeat(total_expr_len),
        
                        red = term::HIGH_RED_ANSI,
                        blue = term::HIGH_BOLD_BLUE_ANSI,
                        clear = term::CLEAR_ANSI,
                    )
                } else if first_tkn.line_number.abs_diff(last_tkn.line_number) < 3 {
                    let mut lines = String::new();

                    for line_number in first_tkn.line_number..=last_tkn.line_number {
                        println!("line number: {line_number}");
                        lines += format!(
                            "{blue}{line_number} | {clear}{line_of_code}\n",
                            line_of_code = get_line_from_contents(line_number, src),
                            blue = term::HIGH_BOLD_BLUE_ANSI,
                            clear = term::CLEAR_ANSI,
                        ).as_str();
                    }

                    let Tkn {
                        token: _,
                        file_name,
                        line_index,
                        line_number,
                    } = first_tkn;
                    
                    write!(f, 
                        "{red}error:{clear} Mismatched types\n\
                        {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                        {blue}{pad} | {clear}\n\
                        {blue}{pad} | {clear}{first_pad}{red} +-- expected {expected_type}, found from here ... {clear}\n\
                        {blue}{pad} | {clear}{first_pad}{red} |                                    {clear}\n\
                        {lines}\
                        {blue}{pad} | {clear}{second_pad}{red} |                                      {clear}\n\
                        {blue}{pad} | {clear}{second_pad}{red} +-- ... to here {calculated_type} {clear}\n\
                        {blue}{pad}:::{clear} \n\
                        \n\
                        ",
                        pad = " ".repeat(line_number.to_string().len()),
                        first_pad = " ".repeat(first_tkn.line_index - 1),
                        second_pad = " ".repeat(last_tkn.line_index - 1),
        
                        red = term::HIGH_RED_ANSI,
                        blue = term::HIGH_BOLD_BLUE_ANSI,
                        clear = term::CLEAR_ANSI,
                    )
                } else {
                    let Tkn {
                        token: _,
                        file_name,
                        line_index,
                        line_number,
                    } = first_tkn;

                    let pad_amount = first_tkn.line_number.to_string().len()
                        .max(last_tkn.line_number.to_string().len())
                        .max(3);
                    
                    write!(f, 
                        "{red}error:{clear} Mismatched types\n\
                        {blue}{pad}-->{clear} {file_name}:{line_number}:{line_index}\n\
                        {blue}{pad} | {clear}\n\
                        {blue}{pad} | {clear}{first_pad}{red} +-- expected {expected_type}, found from here ... {clear}\n\
                        {blue}{pad} | {clear}{first_pad}{red} |                                    {clear}\n\
                        {blue}{line_number_1:>pad_amount$} | {clear}{line_of_code_1}\n\
                        {yellow}{ellipses:>pad_amount$}{clear}{blue} | {clear}\n\
                        {blue}{line_number_2:>pad_amount$} | {clear}{line_of_code_2}\n\
                        {blue}{pad} | {clear}{second_pad}{red} |                                      {clear}\n\
                        {blue}{pad} | {clear}{second_pad}{red} +-- ... to here {calculated_type} {clear}\n\
                        {blue}{pad}:::{clear} \n\
                        \n\
                        ",
                        pad = " ".repeat(pad_amount),
                        ellipses = "...",
                        first_pad = " ".repeat(first_tkn.line_index - 1),
                        second_pad = " ".repeat(last_tkn.line_index - 1),
                        line_number_1 = first_tkn.line_number,
                        line_number_2 = last_tkn.line_number,
                        line_of_code_1 = get_line_from_contents(first_tkn.line_number, src),
                        line_of_code_2 = get_line_from_contents(last_tkn.line_number, src),
        
                        red = term::HIGH_RED_ANSI,
                        blue = term::HIGH_BOLD_BLUE_ANSI,
                        yellow = term::HIGH_YELLOW_ANSI,
                        clear = term::CLEAR_ANSI,
                    )
                }
            },
            PE::SecondDiscardMany { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::InvalidPattern { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::PatternNotMatchExpectedType { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::VariableDoesNotExist { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::InvalidMut { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::CannotMutateImmutable { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::IncorrectNumberPrefixArguments { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::InvalidBlock { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::MultipleExpressions { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::FieldDoesNotExist { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::AlreadyDefinedField { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::FieldExpressionNotDefined { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::InvalidExpressionAtom { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::InvalidDollarExpression { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::InvalidDotExpression { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::AccessorNotDefined { tkn } => todo!("error {:?} at token {tkn:?}", self),
            PE::DefinedIncorrectlyPlacedArgument { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::ConflictingFunctionFixDefinitions { tkn, .. } => todo!("error {:?} at token {tkn:?}", self),
            PE::ExpectedEndOfFunctionDefinition { tkn } => todo!("error {:?} at token {tkn:?}", self),
        }
    }
}

fn get_line_from_contents(line_number: usize, contents: &str) -> &str {
    let mut index = 0;
    for _ in 1..line_number {
        while let Some(chr) = contents.chars().nth(index) && chr != '\n' {
            index += 1;
        }
        index += 1;
    }

    let mut len = 0;
    while let Some(chr) = contents.chars().nth(index + len) && chr != '\n' {
        len += 1;
    }

    return &contents[index..index+len];
}

fn slice_to_string<T: std::fmt::Display>(slice: &[T]) -> Box<str> {
    match slice {
        [] => String::new().into_boxed_str(),
        [a] => format!("{a}").into_boxed_str(),
        [a, b] => format!("{a} and {b}").into_boxed_str(),
        list => {
            let (last, values) = list.split_last().unwrap();
            let mut output = String::new();
            for value in values {
                output += value.to_string().as_str();
                output += ", ";
            }
            output += " and ";
            output += last.to_string().as_str();

            output.into_boxed_str()
        }
    }
}

fn iter_to_string<I, T>(iter: &I) -> Box<str> 
where 
    I: ExactSizeIterator<Item = T> + DoubleEndedIterator + Clone, 
    T: std::fmt::Display
{
    if iter.is_empty() {
        return String::new().into_boxed_str();
    } else if iter.len() == 1 {
        let a = iter.clone().next().unwrap(); 
        format!("{a}").into_boxed_str() 
    } else if iter.len() == 2 {
        let [a, b] = iter.clone().next_chunk().ok().unwrap(); 
        format!("{a} and {b}").into_boxed_str() 
    } else {
        let mut values = iter.clone();
        let last = values.next_back().unwrap();

        let mut output = String::new();

        for value in values {
            output += value.to_string().as_str();
            output += ", ";
        }
        output += " and ";
        output += last.to_string().as_str();
        
        output.into_boxed_str()
    }
}

#[allow(dead_code)]
fn num_whitespace(line: &str) -> usize {
    let mut len = 0;

    for chr in line.chars() {
        if chr.is_whitespace() {
            len += 1;
            continue;
        }
        break;
    }

    return len;
}