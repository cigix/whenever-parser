use whenever_parser;

#[test]
fn smoke_single_line()
{
    // First line of fibo.wnvr
    let input = "1 again (1) defer (3 || N(1)<=N(2) || N(7)>99) 2#N(1),3,7;";

    whenever_parser::parse_line(input).unwrap();
}

#[test]
fn smoke_hello()
{
    // Program for 2 "Hello world!"s
    let program = include_str!("hello.wnvr");

    for line in program.lines()
    {
        whenever_parser::parse_line(line).unwrap();
    }
}

#[test]
fn smoke_beer()
{
    // Program for "99 bottles of beer"
    let program = include_str!("beer.wnvr");

    for line in program.lines()
    {
        whenever_parser::parse_line(line).unwrap();
    }
}

#[test]
fn smoke_fibonacci()
{
    // Program for the first 100 Fibonacci numbers
    let program = include_str!("fibo.wnvr");

    for line in program.lines()
    {
        whenever_parser::parse_line(line).unwrap();
    }
}
