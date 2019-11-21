pub mod lexer;
pub mod ast;
pub mod parser;

/// Turn the input into an AST.
///
/// Returns the root of the AST, an `ast::Line` struct.
///
/// # Errors
///
/// Will return an description of the error and a slice of the input where the
/// error happened:
/// * input could not be tokenized (see `lexer::eat`)
/// * a token that was not expected in that context was found
/// * an `ast::Line` could have been built but there were remaining tokens
pub fn parse_line<'a>(line: &'a str)
    -> Result<ast::Line<'a>, (String, &'a str)>
{
    let cursor0 = line;

    let (line, cursor1) = parser::eat_line(cursor0)?;

    let (token1, _) = lexer::eat(cursor1)?;
    match token1.variant
    {
        lexer::TokenVariant::EOI => Ok(line),
        _ => Err((String::from("Expected end of input"), token1.tok))
    }
}
