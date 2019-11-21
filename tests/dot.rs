use whenever_parser;

use whenever_parser::ast;

#[test]
fn to_dot_1_1()
{
    let input = "1 1;";
    let expected = r#"  "0x0_4_Line" [label=<<I>Line</I>>];
  "0x0_4_Line" -> "0x0_1";
  "0x0_1" [label="1 (1)"];
  "0x0_4_Line" -> "0x2_1_LineOperations";
  "0x2_1_LineOperations" [label=<<I>LineOperations</I>>];
  "0x2_1_LineOperations" -> "0x2_1_LineOp";
  "0x2_1_LineOp" [label=<<I>LineOp</I>>];
  "0x2_1_LineOp" -> "0x2_1_AbsoluteNumber_NumToLineOp";
  "0x2_1_AbsoluteNumber_NumToLineOp" [label=<<I>Number ⮕ SingleLineOp</I>>];
  "0x2_1_AbsoluteNumber_NumToLineOp" -> "0x2_1_AbsoluteNumber";
  "0x2_1_AbsoluteNumber" [label=<<I>AbsoluteNumber</I>>];
  "0x2_1_AbsoluteNumber" -> "0x2_1";
  "0x2_1" [label="1 (1)"];
  "0x0_4_Line" -> "0x3_1";
  "0x3_1" [label=";"];
"#;

    let line = whenever_parser::parse_line(input).unwrap();
    let actual = ast::to_dot_normalized(&line);
    assert_eq!(actual, expected);
}

#[test]
fn to_dot_fibo()
{
    // First line of fibo.wnvr
    let input = "1 again (1) defer (3 || N(1)<=N(2) || N(7)>99) 2#N(1),3,7;";
    let expected = include_str!("fibo1.dot");

    let line = whenever_parser::parse_line(input).unwrap();
    let actual = format!("digraph {{\n{}}}\n", ast::to_dot_normalized(&line));
    assert_eq!(actual, expected);
}
