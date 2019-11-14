pub enum TokenVariant
{
    // Whenever specific tokens
    Again,
    Defer,
    Forget,
    N,
    Print,
    Read,
    U,

    // Same token for number and string contexts
    Plus, // +
    // Same token for unary and binary operators
    Minus, // -

    // Usual tokens
    Number(usize),
    String,
    UnBoolOp, // !
    BinBoolOp, // && ||
    BinNumBoolOp, // == != < <= > >=
    MathOp, // * /

    // Separator tokens
    Comma,
    LeftParens,
    RightParens,
    Semicolon,
    Sharp,

    // End of input
    EOI
}

pub struct Token<'a>
{
    pub tok: &'a str,
    pub variant: TokenVariant
}

/// Reads a token from the input.
///
/// Returns a token and a slice from the end of the token to the end of the
/// input.
///
/// Note: difference between input and output slices may be more than the length
/// of the token, as leading whitespace is ignored.
///
/// # Errors
///
/// Will return `Err` if:
/// * end of input is reached in an unfinished token (such as a string)
/// * a number contains an invalid digit
/// * token is not known (reminder: keywords are case sensitive)
///
/// `Err`'s contents are a description of the issue and the slice where it
/// happened.
pub fn eat<'a>(input: &'a str)
    -> Result<(Token<'a>, &'a str), (String, &'a str)>
{
    let input = input.trim_start();

macro_rules! make_token
{
    ($type: expr, $input: ident, $pos: expr) =>
    {
        {
            let (token, output) = $input.split_at($pos);
            Ok((Token { tok: token, variant: $type }, output))
        }
    };
}

    match input.chars().next()
    {
        None => make_token!(TokenVariant::EOI, input, 0),
        Some('+') => make_token!(TokenVariant::Plus, input, 1),
        Some('-') => make_token!(TokenVariant::Minus, input, 1),
        Some('*') | Some('/') =>
            make_token!(TokenVariant::MathOp, input, 1),
        Some('!') => make_token!(TokenVariant::UnBoolOp, input, 1),
        Some(',') => make_token!(TokenVariant::Comma, input, 1),
        Some('(') => make_token!(TokenVariant::LeftParens, input, 1),
        Some(')') => make_token!(TokenVariant::RightParens, input, 1),
        Some(';') => make_token!(TokenVariant::Semicolon, input, 1),
        Some('#') => make_token!(TokenVariant::Sharp, input, 1),
        Some('"') =>
        {
            let mut escape = false;

            for (i, c) in input.char_indices().skip(1)
            {
                match c
                {
                    '"' if escape == false => {
                        return make_token!(TokenVariant::String, input, i + 1)
                    },
                    '\\' if escape == false => escape = true,
                    _ => escape = false,
                }
            }

            Err((String::from("End of input while reading string"), input))
        }
        Some(c) if c.is_ascii_digit() => // Number
        {
            let mut base = 10;
            let mut start = 0;
            let mut end = 0;
            if c == '0'
            {
                match input.chars().skip(1).next()
                {
                    Some('b') => // 0bXXXX...: binary
                    {
                        let mut pos = 2;
                        for (i, c) in input.char_indices().skip(2)
                        {
                            if c != '0' && c != '1'
                            {
                                break;
                            }
                            pos = i + 1;
                        }
                        if 2 < pos
                        {
                            base = 2;
                            start = 2;
                            end = pos
                        }
                    },
                    Some('x') | Some('X') => // 0xXXXX...: hexadecimal
                    {
                        let mut pos = 2;
                        for (i, c) in input.char_indices().skip(2)
                        {
                            if !c.is_ascii_hexdigit()
                            {
                                break;
                            }
                            pos = i + 1;
                        }
                        if 2 < pos
                        {
                            base = 16;
                            start = 2;
                            end = pos;
                        }
                    }
                    Some(d) if d.is_ascii_digit() => // 0XXX...: octal
                    {
                        // We will use the same digit reading as base 10
                        base = 8;
                        start = 1;
                    }
                    Some(_) | None => () // End of input
                }
            }
            if end == 0
            {
                for (i, c) in input.char_indices().skip(start)
                {
                    if !c.is_ascii_digit()
                    {
                        break;
                    }
                    end = i + 1;
                }
            }

            let (tok, output) = input.split_at(end);
            match usize::from_str_radix(&tok[start..], base)
            {
                Ok(num) => Ok((Token { tok,
                                       variant: TokenVariant::Number(num) },
                               output)),
                Err(error) => Err((error.to_string(), &tok[start..]))
            }
        }
        Some(c) if c.is_ascii_alphabetic() =>
        {
            let mut end = 1;
            for (i, c) in input.char_indices().skip(1)
            {
                if !c.is_ascii_alphabetic()
                {
                    break;
                }
                end = i + 1
            }

            match &input[..end]
            {
                "again" => make_token!(TokenVariant::Again, input, end),
                "defer" => make_token!(TokenVariant::Defer, input, end),
                "forget" => make_token!(TokenVariant::Forget, input, end),
                "N" => make_token!(TokenVariant::N, input, end),
                "print" => make_token!(TokenVariant::Print, input, end),
                "read" => make_token!(TokenVariant::Read, input, end),
                "U" => make_token!(TokenVariant::U, input, end),
                _ => Err((String::from("Unknown token"), input))
            }
        }
        Some('<') | Some('>') =>
        {
            match input.chars().skip(1).next()
            {
                Some('=') => make_token!(TokenVariant::BinNumBoolOp, input, 2),
                _ => make_token!(TokenVariant::BinNumBoolOp, input, 1)
            }
        }
        Some(_) =>
        {
            // Match the remaining tokens: && || == !=
            match input.get(..2)
            {
                Some("&&") | Some("||") =>
                    make_token!(TokenVariant::BinBoolOp, input, 2),
                Some("==") | Some("!=") =>
                    make_token!(TokenVariant::BinNumBoolOp, input, 2),
                _ => Err((String::from("Unknown token"), input))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! check_token
    {
        ($input: expr, $type: pat, $from: expr, $at: expr) =>
        {
            {
                let input = $input;

                let res = eat(input);
                if let Ok((token, output)) = res
                {
                    if let $type = token.variant
                    {
                        assert_eq!(token.tok, &input[$from..$at]);
                        assert_eq!(output, &input[$at..]);
                    }
                    else
                    {
                        panic!("Not a {}", stringify!($type));
                    }
                }
                else
                {
                    panic!("Not OK");
                }
            }
        };
    }

    #[test]
    fn end_of_input_check()
    {
        check_token!("", TokenVariant::EOI, 0, 0);
        check_token!("    \t\n ", TokenVariant::EOI, 7, 7)
    }

    #[test]
    fn plus_check()
    {
        check_token!("+abc", TokenVariant::Plus, 0, 1)
    }

    #[test]
    fn string_check()
    {
        // Actual bytes: "ab\"\\"de
        check_token!("\"ab\\\"\\\\\"de", TokenVariant::String, 0, 8)
    }

    #[test]
    fn skip_spaces()
    {
        check_token!("       \t\n !", TokenVariant::UnBoolOp, 10, 11)
    }

    macro_rules! check_token_number
    {
        ($input: expr, $val: expr, $from: expr, $at: expr) =>
        {
            {
                let input = $input;

                let res = eat(input);
                if let Ok((token, output)) = res
                {
                    if let TokenVariant::Number(num) = token.variant
                    {
                        assert_eq!(token.tok, &input[$from..$at]);
                        assert_eq!(num, $val);
                        assert_eq!(output, &input[$at..]);
                    }
                    else
                    {
                        panic!("Not a TokenVariant::Number");
                    }
                }
                else
                {
                    panic!("Not OK");
                }
            }
        };
    }

    #[test]
    fn number_check()
    {
        check_token_number!("42abc", 42, 0, 2);
        check_token_number!("  1337", 1337, 2, 6);
        check_token_number!("0", 0, 0, 1);
        check_token_number!("0b", 0, 0, 1);
        check_token_number!("0x", 0, 0, 1);
        check_token_number!("00", 0, 0, 2);
        check_token_number!("0a", 0, 0, 1);
        check_token_number!("0b101010", 0b101010, 0, 8);
        check_token_number!("0xdeadbeef", 0xdeadbeef, 0, 10);
        check_token_number!("0XCaFe", 0xCAFE, 0, 6);
        check_token_number!("0777", 0x1ff, 0, 4);
        check_token_number!(" 42💻", 42, 1, 3);
    }

    #[test]
    fn keyword_check()
    {
        check_token!("  again ", TokenVariant::Again, 2, 7);
        check_token!("print()", TokenVariant::Print, 0, 5);
    }

    macro_rules! check_token_error
    {
        ($input: expr) =>
        {
            {
                let input = $input;

                let res = eat(input);

                assert!(res.is_err());
            }
        }
    }

    #[test]
    fn error_check()
    {
        check_token_error!("\"abcd"); // End of input while in string
        check_token_error!("089"); // Wrong digits for base
        check_token_error!("Again"); // Wrong case keyword
        check_token_error!("abcd"); // Unknown token
        check_token_error!("💻"); // Unknown non-ASCII token
    }

    #[test]
    fn string_tokenization()
    {
        // First line of fibo.wnvr
        let line =
            "1 again (1) defer (3 || N(1)<=N(2) || N(7)>99) 2#N(1),3,7;";
        let mut input : &str = line;

        let expected = vec![
            "1", "again", "(", "1", ")", "defer", "(", "3", "||", "N", "(", "1",
            ")", "<=", "N", "(", "2", ")", "||", "N", "(", "7", ")", ">", "99",
            ")", "2", "#", "N", "(", "1", ")", ",", "3", ",", "7", ";"];
        let mut actual = Vec::new();

        loop
        {
            match eat(input)
            {
                Err((error, _)) => panic!("{}. input: \"{}\"", error, input),
                Ok((token, remainder)) =>
                {
                    if let TokenVariant::EOI = token.variant
                    {
                        break;
                    }
                    actual.push(token.tok);
                    input = remainder;
                }
            }
        }

        assert_eq!(actual, expected);
    }
}
