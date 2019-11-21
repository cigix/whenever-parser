use crate::lexer;
use crate::ast;

use crate::lexer::TokenVariant;

// Nomenclature:
// tokenX: the Xth token read, 0 based
// cursorX: the place in the input where we will get tokenX and cursorX+1 when
//          eating

pub fn eat_absnumber<'a>(input: &'a str)
    -> Result<(ast::AbsNumber<'a>, &'a str), (String, &'a str)>
{
    let cursor0 = input;
    let (token0, cursor1) = lexer::eat(cursor0)?;
    match token0.variant
    {
        TokenVariant::Number(val) =>
        {
            let numbertok = ast::NumberToken { tok: token0.tok, val };
            Ok((ast::AbsNumber { alt: Box::new(numbertok) }, cursor1))
        }
        TokenVariant::N =>
        {
            let keywordtok = ast::NToken { tok: token0.tok };
            let (token1, cursor2) = lexer::eat(cursor1)?;
            if let TokenVariant::LeftParens = token1.variant {} else
            {
                return Err((String::from("Expected `(`"), token1.tok));
            }
            let lparenstok = ast::LeftParensToken { tok: token1.tok };
            let (number, cursor3) = eat_number(cursor2)?;
            let (token3, cursor4) = lexer::eat(cursor3)?;
            if let TokenVariant::RightParens = token3.variant {} else
            {
                return Err((String::from("Expected `)`"), token3.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token3.tok };
            let n = ast::N::new(keywordtok, lparenstok, number, rparenstok);
            Ok((ast::AbsNumber { alt: Box::new(n) }, cursor4))
        }
        TokenVariant::Read =>
        {
            let keywordtok = ast::ReadToken { tok: token0.tok };
            let (token1, cursor2) = lexer::eat(cursor1)?;
            if let TokenVariant::LeftParens = token1.variant {} else
            {
                return Err((String::from("Expected `(`"), token1.tok));
            }
            let lparenstok = ast::LeftParensToken { tok: token1.tok };
            let (token2, cursor3) = lexer::eat(cursor2)?;
            if let TokenVariant::RightParens = token2.variant {} else
            {
                return Err((String::from("Expected `)`"), token2.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token2.tok };
            let read = ast::Read::new(keywordtok, lparenstok, rparenstok);
            Ok((ast::AbsNumber { alt: Box::new(read) }, cursor3))
        }
        _ => Err((String::from("Expected number"), token0.tok))
    }
}

pub fn eat_number<'a>(input: &'a str)
    -> Result<(ast::Number<'a>, &'a str), (String, &'a str)>
{
    let cursor0 = input;
    let (token0, cursor1) = lexer::eat(cursor0)?;
    let cursorlast; // cursor at the end of the current number
    let number1 : ast::Number = match token0.variant
    {
        TokenVariant::Number(_) | TokenVariant::N | TokenVariant::Read =>
        {
            let (absnum, cursor1) = eat_absnumber(cursor0)?;
            let absolutenumber = ast::AbsoluteNumber::new(absnum);
            cursorlast = cursor1;
            ast::Number { alt: Box::new(absolutenumber) }
        }
        TokenVariant::Plus | TokenVariant::Minus =>
        {
            let unmathop = ast::UnMathOp { alt: match token0.variant
                {
                    TokenVariant::Plus =>
                        Box::new(ast::PlusToken { tok: token0.tok }),
                    TokenVariant::Minus =>
                        Box::new(ast::MinusToken { tok: token0.tok }),
                    _ => unreachable!()
                }
            };
            let (number, cursor2) = eat_number(cursor1)?;
            let unopnum = ast::UnOpNumber::new(unmathop, number);
            cursorlast = cursor2;
            ast::Number { alt: Box::new(unopnum) }
        }
        TokenVariant::LeftParens =>
        {
            let lparenstok = ast::LeftParensToken { tok: token0.tok };
            let (number, cursor2) = eat_number(cursor1)?;
            let (token2, cursor3) = lexer::eat(cursor2)?;
            if let TokenVariant::RightParens = token2.variant {} else
            {
                return Err((String::from("Expected `)`"), token2.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token2.tok };
            let parensnum = ast::ParensNumber::new(lparenstok,
                                                   number,
                                                   rparenstok);
            cursorlast = cursor3;
            ast::Number { alt: Box::new(parensnum) }
        }
        TokenVariant::String | TokenVariant::U =>
        {
            let (string, cursor1) = eat_string(cursor0)?;
            let stringtonum = ast::StringToNum { string };
            cursorlast = cursor1;
            ast::Number { alt: Box::new(stringtonum) }
        }
        _ => { return Err((String::from("Expected number"), token0.tok)); }
    };

    let (tokenlast, cursorlastplus1) = lexer::eat(cursorlast)?;
    match tokenlast.variant
    {
        TokenVariant::Plus | TokenVariant::Minus | TokenVariant::MathOp =>
        {
            let binmathop = ast::BinMathOp { alt : match tokenlast.variant
                {
                    TokenVariant::Plus =>
                        Box::new(ast::PlusToken { tok: tokenlast.tok }),
                    TokenVariant::Minus =>
                        Box::new(ast::MinusToken { tok: tokenlast.tok }),
                    TokenVariant::MathOp =>
                        Box::new(ast::MathOpToken { tok: tokenlast.tok }),
                    _ => unreachable!()
                }
            };
            let (number2, cursorlastplus2) = eat_number(cursorlastplus1)?;
            let binopnum = ast::BinOpNumber::new(number1, binmathop, number2);
            Ok((ast::Number { alt: Box::new(binopnum) }, cursorlastplus2))
        }
        _ => Ok((number1, cursorlast))
    }
}

pub fn eat_boolean<'a>(input: &'a str)
    -> Result<(ast::Boolean<'a>, &'a str), (String, &'a str)>
{
    let cursor0 = input;
    let (token0, cursor1) = lexer::eat(cursor0)?;
    let cursorlast; // cursor at the end of the current boolean
    let boolean1 : ast::Boolean = match token0.variant
    {
        TokenVariant::UnBoolOp =>
        {
            let unbooloptok = ast::UnBoolOpToken { tok: token0.tok };
            let (boolean, cursor2) = eat_boolean(cursor1)?;
            let unopboolean = ast::UnOpBoolean::new(unbooloptok, boolean);
            cursorlast = cursor2;
            ast::Boolean { alt: Box::new(unopboolean) }
        }
        TokenVariant::LeftParens =>
        {
            let lparenstok = ast::LeftParensToken { tok: token0.tok };
            let (boolean, cursor2) = eat_boolean(cursor1)?;
            let (token2, cursor3) = lexer::eat(cursor2)?;
            if let TokenVariant::RightParens = token2.variant {} else
            {
                return Err((String::from("Expected `)`"), token2.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token2.tok };
            let parensbool = ast::ParensBoolean::new(lparenstok,
                                                     boolean,
                                                     rparenstok);
            cursorlast = cursor3;
            ast::Boolean { alt: Box::new(parensbool) }
        }
        TokenVariant::Number(_) | TokenVariant::N | TokenVariant::Read
            | TokenVariant::Plus | TokenVariant::Minus
            | TokenVariant::String | TokenVariant::U =>
        {
            let (number1, cursor1) = eat_number(cursor0)?;
            let (token1, cursor2) = lexer::eat(cursor1)?;
            match token1.variant
            {
                TokenVariant::BinNumBoolOp =>
                {
                    // number BINNUMBOOLOP number
                    // reduce => binopnumbool
                    let binnumbooloptok = ast::BinNumBoolOpToken {
                        tok: token1.tok };
                    let (number2, cursor3) = eat_number(cursor2)?;
                    let binopnumboolean =
                        ast::BinOpNumBoolean::new(number1,
                                                  binnumbooloptok,
                                                  number2);
                    cursorlast = cursor3;
                    ast::Boolean { alt: Box::new(binopnumboolean) }
                }
                _ =>
                {
                    // number
                    // reduce => numtobool
                    let numtobool = ast::NumToBool { num: number1 };
                    cursorlast = cursor1;
                    ast::Boolean { alt: Box::new(numtobool) }
                }
            }
        }
        _ => { return Err((String::from("Expected boolean"), token0.tok)); }
    };

    let (tokenlast, cursorlastplus1) = lexer::eat(cursorlast)?;
    match tokenlast.variant
    {
        TokenVariant::BinBoolOp =>
        {
            let binbooloptok = ast::BinBoolOpToken { tok: tokenlast.tok };
            let (boolean2, cursorlastplus2) = eat_boolean(cursorlastplus1)?;
            let binopboolean = ast::BinOpBoolean::new(boolean1,
                                                      binbooloptok,
                                                      boolean2);
            Ok((ast::Boolean { alt: Box::new(binopboolean) }, cursorlastplus2))
        }
        _ => Ok((boolean1, cursorlast))
    }
}

pub fn eat_string<'a>(input: &'a str)
    -> Result<(ast::String_<'a>, &'a str), (String, &'a str)>
{
    let cursor0 = input;
    let (token0, cursor1) = lexer::eat(cursor0)?;
    let cursorlast; // cursor at the end of the current string
    let string1 : ast::String_<'a> = match token0.variant
    {
        TokenVariant::String =>
        {
            let stringtoken = ast::StringToken { tok: token0.tok };
            cursorlast = cursor1;
            ast::String_ { alt: Box::new(stringtoken) }
        }
        TokenVariant::U =>
        {
            let keywordtok = ast::UToken { tok: token0.tok };
            let (token1, cursor2) = lexer::eat(cursor1)?;
            if let TokenVariant::LeftParens = token1.variant {} else
            {
                return Err((String::from("Expected `(`"), token1.tok));
            }
            let lparenstok = ast::LeftParensToken { tok: token1.tok };
            let (number, cursor3) = eat_absnumber(cursor2)?;
            let (token3, cursor4) = lexer::eat(cursor3)?;
            if let TokenVariant::RightParens = token3.variant {} else
            {
                return Err((String::from("Expected `)`"), token3.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token3.tok };
            let u = ast::U::new(keywordtok, lparenstok, number, rparenstok);
            cursorlast = cursor4;
            ast::String_ { alt: Box::new(u) }
        }
        TokenVariant::Number(_) | TokenVariant::N | TokenVariant::Read
            | TokenVariant::Plus | TokenVariant::Minus
            | TokenVariant::LeftParens =>
        {
            let (number, cursor1) = eat_number(cursor0)?;
            let numtostring = ast::NumToString { num: number };
            cursorlast = cursor1;
            ast::String_ { alt: Box::new(numtostring) }
        }
        _ => { return Err((String::from("Expected string"), token0.tok)); }
    };

    let (tokenlast, cursorlastplus1) = lexer::eat(cursorlast)?;
    match tokenlast.variant
    {
        TokenVariant::Plus =>
        {
            let plustoken = ast::PlusToken { tok: tokenlast.tok };
            let (string2, cursorlastplus2) = eat_string(cursorlastplus1)?;
            let concat = ast::Concat::new(string1, plustoken, string2);
            Ok((ast::String_ { alt: Box::new(concat) }, cursorlastplus2))
        }
        _ => Ok((string1, cursorlast))
    }
}

pub fn eat_lineops<'a>(input: &'a str)
    -> Result<(ast::LineOps<'a>, &'a str), (String, &'a str)>
{
    let cursor0 = input;
    let (number, cursor1) = eat_number(cursor0)?;
    let (token1, cursor2) = lexer::eat(cursor1)?;
    match token1.variant
    {
        TokenVariant::Comma =>
        {
            // number COMMA lineops
            // reduce => numtolineop COMMA lineops
            // reduce => lineop COMMA lineops
            // reduce => lineoplist
            let numtolineop = ast::NumToLineOp { num: number };
            let lineop = ast::LineOp::new(ast::SingleLineOp {
                alt: Box::new(numtolineop)
            });
            let commatok = ast::CommaToken { tok: token1.tok };
            let (lineops, cursor3) = eat_lineops(cursor2)?;
            let lineoplist = ast::LineOpList::new(lineop, commatok, lineops);
            Ok((ast::LineOps { alt: Box::new(lineoplist) }, cursor3))
        }
        TokenVariant::Sharp =>
        {
            // number SHARP number
            // reduce => countlineop
            // reduce => lineop
            let sharptok = ast::SharpToken { tok: token1.tok };
            let (count, cursor3) = eat_number(cursor2)?;
            let countlineop = ast::CountLineOp::new(number, sharptok, count);
            let lineop = ast::LineOp::new(ast::SingleLineOp {
                alt: Box::new(countlineop) });

            let (token3, cursor4) = lexer::eat(cursor3)?;
            match token3.variant
            {
                TokenVariant::Comma =>
                {
                    // lineop COMMA lineops
                    // reduce => lineoplist
                    let commatok = ast::CommaToken { tok: token3.tok };
                    let (lineops, cursor5) = eat_lineops(cursor4)?;
                    let lineoplist = ast::LineOpList::new(lineop,
                                                          commatok,
                                                          lineops);
                    Ok((ast::LineOps { alt: Box::new(lineoplist) }, cursor5))
                }
                _ => Ok((ast::LineOps { alt: Box::new(lineop) }, cursor3))
            }
        }
        _ =>
        {
            // number
            // reduce => numtolineop
            // reduce => lineop
            let numtolineop = ast::NumToLineOp { num: number };
            let lineop = ast::LineOp::new(ast::SingleLineOp {
                alt: Box::new(numtolineop) });
            Ok((ast::LineOps { alt: Box::new(lineop) }, cursor1))
        }
    }
}

pub fn eat_statement<'a>(input: &'a str)
    -> Result<(ast::Statement<'a>, &'a str), (String, &'a str)>
{
    let cursor0 = input;
    let (token0, cursor1) = lexer::eat(cursor0)?;
    match token0.variant
    {
        TokenVariant::Again =>
        {
            let keywordtok = ast::AgainToken { tok: token0.tok };
            let (token1, cursor2) = lexer::eat(cursor1)?;
            if let TokenVariant::LeftParens = token1.variant {} else
            {
                return Err((String::from("Expected `(`"), token1.tok));
            }
            let lparenstok = ast::LeftParensToken { tok: token1.tok };
            let (boolean, cursor3) = eat_boolean(cursor2)?;
            let (token3, cursor4) = lexer::eat(cursor3)?;
            if let TokenVariant::RightParens = token3.variant {} else
            {
                return Err((String::from("Expected `)`"), token3.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token3.tok };
            let (statement, cursor5) = eat_statement(cursor4)?;
            let again = ast::Again::new(keywordtok,
                                        lparenstok,
                                        boolean,
                                        rparenstok,
                                        statement);
            Ok((ast::Statement { alt: Box::new(again) }, cursor5))
        }
        TokenVariant::Defer =>
        {
            let keywordtok = ast::DeferToken { tok: token0.tok };
            let (token1, cursor2) = lexer::eat(cursor1)?;
            if let TokenVariant::LeftParens = token1.variant {} else
            {
                return Err((String::from("Expected `(`"), token1.tok));
            }
            let lparenstok = ast::LeftParensToken { tok: token1.tok };
            let (boolean, cursor3) = eat_boolean(cursor2)?;
            let (token3, cursor4) = lexer::eat(cursor3)?;
            if let TokenVariant::RightParens = token3.variant {} else
            {
                return Err((String::from("Expected `)`"), token3.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token3.tok };
            let (statement, cursor5) = eat_statement(cursor4)?;
            let defer = ast::Defer::new(keywordtok,
                                        lparenstok,
                                        boolean,
                                        rparenstok,
                                        statement);
            Ok((ast::Statement { alt: Box::new(defer) }, cursor5))
        }
        TokenVariant::Forget =>
        {
            let keywordtok = ast::ForgetToken { tok: token0.tok };
            let (token1, cursor2) = lexer::eat(cursor1)?;
            if let TokenVariant::LeftParens = token1.variant {} else
            {
                return Err((String::from("Expected `(`"), token1.tok));
            }
            let lparenstok = ast::LeftParensToken { tok: token1.tok };
            let (boolean, cursor3) = eat_boolean(cursor2)?;
            let (token3, cursor4) = lexer::eat(cursor3)?;
            if let TokenVariant::RightParens = token3.variant {} else
            {
                return Err((String::from("Expected `)`"), token3.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token3.tok };
            let (statement, cursor5) = eat_statement(cursor4)?;
            let forget = ast::Forget::new(keywordtok,
                                          lparenstok,
                                          boolean,
                                          rparenstok,
                                          statement);
            Ok((ast::Statement { alt: Box::new(forget) }, cursor5))
        }
        TokenVariant::Print =>
        {
            let keywordtok = ast::PrintToken { tok: token0.tok };
            let (token1, cursor2) = lexer::eat(cursor1)?;
            if let TokenVariant::LeftParens = token1.variant {} else
            {
                return Err((String::from("Expected `(`"), token1.tok));
            }
            let lparenstok = ast::LeftParensToken { tok: token1.tok };
            let (string, cursor3) = eat_string(cursor2)?;
            let (token3, cursor4) = lexer::eat(cursor3)?;
            if let TokenVariant::RightParens = token3.variant {} else
            {
                return Err((String::from("Expected `)`"), token3.tok));
            }
            let rparenstok = ast::RightParensToken { tok: token3.tok };
            let print = ast::Print::new(keywordtok,
                                        lparenstok,
                                        string,
                                        rparenstok);
            Ok((ast::Statement { alt: Box::new(print) }, cursor4))
        }
        TokenVariant::Number(_) | TokenVariant::N | TokenVariant::Read
            | TokenVariant::Plus | TokenVariant::Minus
            | TokenVariant::LeftParens | TokenVariant::String
            | TokenVariant::U =>
        {
            let (lineops, cursor1) = eat_lineops(cursor0)?;
            let lineoperations = ast::LineOperations::new(lineops);
            Ok((ast::Statement { alt: Box::new(lineoperations) }, cursor1))
        }
        _ => Err((String::from("Expected statement"), token0.tok))
    }
}

pub fn eat_line<'a>(input: &'a str)
    -> Result<(ast::Line<'a>, &'a str), (String, &'a str)>
{
    let cursor0 = input;

    let (token0, cursor1) = lexer::eat(cursor0)?;
    let lineno;
    if let TokenVariant::Number(val) = token0.variant
    {
        lineno = ast::NumberToken { tok: token0.tok, val };
    }
    else
    {
        return Err((String::from("Expected number"), token0.tok));
    }

    let (statement, cursor2) = eat_statement(cursor1)?;

    let (token2, cursor3) = lexer::eat(cursor2)?;
    if let TokenVariant::Semicolon = token2.variant {} else
    {
        return Err((String::from("Expected `;`"), token2.tok));
    }
    let semicolontok = ast::SemicolonToken { tok: token2.tok };

    Ok((ast::Line::new(lineno, statement, semicolontok), cursor3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absnumber_check()
    {
        let input = "123abc";

        match eat_absnumber(input)
        {
            Ok((root, cursor)) =>
            {
                assert_eq!(root.alt.get_str(), &input[..3]);
                assert_eq!(cursor, &input[3..]);
            }
            Err((error, at)) => panic!("{}: {}", error, at)
        }
    }
}
