// Grammar:
// The grammar has been refined so that alternations and sequences don't mix,
// allowing to express all alternations through traits and all sequences
// through structs.
// Lowercase means nonterminal, uppercase means terminal (tokens).
//
// Root:
// line := NUMBER statement SEMICOLON
//
// number := absolutenumber | unopnum | binopnum | parensnum | stringtonum
// absolutenumber := absnum // This avoids alternation in alternation
// absnum := NUMBER | n | read
// unmathop := PLUS | MINUS
// unopnum := unmathop number
// binmathop := PLUS | MINUS | MATHOP
// binopnum := number binmathop number
// parensnum := LPAREN number RPAREN
//
// boolean := unopbool | binopbool | binopnumbool | parensbool | numtobool
// unopbool := UNBOOLOP boolean
// binopbool := boolean BINBOOLOP boolean
// binopnumbool := number BINNUMBOOLOP number
// parensbool := LPAREN boolean RPAREN
//
// n := N LPAREN number RPAREN
// read := READ LPAREN RPAREN
//
// statement := lineoperations | again | defer | forget | print
// lineoperations := lineops // ditto
// lineops := lineop | lineoplist
// lineop := singlelineop // ditto
// singlelineop := numtolineop | countlineop
// countlineop := number SHARP number
// lineoplist := lineop COMMA lineops
// again := AGAIN LPAREN boolean RPAREN statement
// defer := DEFER LPAREN boolean RPAREN statement
// forget := FORGET LPAREN boolean RPAREN statement
// print := PRINT LPAREN string RPAREN
//
// string := STRING | u | concat | numtostring
// concat := string PLUS string
// u := U LPAREN absnum RPAREN
//
// These conversion are not straightforward and will require their own nodes in
// the AST:
// numtobool := number
// numtolineop := number
// numtostring := number
// stringtonum := string

use std::ptr;

// Traits defining relationships between nodes
pub trait Graph<'a> {
    /// Returns the range of characters spanned by the node.
    fn get_str(&self) -> &'a str;
    /// Returns a text representation of the node.
    fn get_label(&self) -> String;
    /// Generates arrows for each children and returns the concatenation of the
    /// call to `to_dot` on them.
    ///
    /// TODO: Find a non-intrusive fashion to do the same thing.
    fn to_dot_recurse(&self) -> String;

    /// Returns a unique identifier for this node.
    fn get_id(&self) -> String;
    /// Returns a representation of the subtree of `self` in Graphiz' dot
    /// format.
    fn to_dot(&self) -> String
    {
        let id = self.get_id();
        let mut dot =
            format!("  \"{}\" [label={}];\n", id, self.get_label());
        dot += self.to_dot_recurse().as_str();
        return dot;
    }
}

/// Returns the output of `to_dot()` with identifiers normalized for
/// consistency.
///
/// Replaces all occurrences of pointers with their offset from
/// `root.get_str().as_ptr()`.
pub fn to_dot_normalized<'a>(root: &dyn Graph<'a>) -> String
{
    let dot = root.to_dot();
    let base = root.get_str().as_ptr() as usize;

    let hexsize = format!("{:x}", base).len(); // TODO: atoi maybe ?
    let mut split = dot.split("0x");

    let mut normalized = String::from(split.next().unwrap());

    for abnormal in split
    {
        let (addr, remainder) = abnormal.split_at(hexsize);
        let offset = usize::from_str_radix(addr, 16).unwrap() - base;
        normalized += format!("{:#x}", offset).as_str();
        normalized += remainder;
    }

    normalized
}

// Traits for all alternations, and struct wrappers
//
// Alternation structs are not part of the graph, they are just here to wrap
// around a Box<> and provide a consistent interface.
trait Alternation<'a>
{
    fn get_str(&self) -> &'a str;
    fn get_id(&self) -> String;
    fn to_dot(&self) -> String;
}
macro_rules! define_alternation
{
    ($name: ident, $altname: ident) =>
    {
        pub trait $altname<'a>: Graph<'a> {}
        pub struct $name<'a>
        {
            pub alt: Box<dyn $altname<'a> + 'a>
        }
        impl<'a> Alternation<'a> for $name<'a>
        {
            fn get_str(&self) -> &'a str { self.alt.get_str() }
            fn get_id(&self) -> String { self.alt.get_id() }
            fn to_dot(&self) -> String { self.alt.to_dot() }
        }
    }
}
macro_rules! implement_alternations
{
    ($name: ident, $($alt: ident),*) =>
    {
        $(impl<'a> $alt<'a> for $name<'a> {})*
    }
}

define_alternation!(Number, NumberAlt);
define_alternation!(AbsNumber, AbsNumAlt);
define_alternation!(UnMathOp, UnMathOpAlt);
define_alternation!(BinMathOp, BinMathOpAlt);
define_alternation!(Boolean, BoolAlt);
define_alternation!(Statement, StatementAlt);
define_alternation!(LineOps, LineOpsAlt);
define_alternation!(SingleLineOp, SingleLineOpAlt);
define_alternation!(String_, StringAlt); // Fixing collision is too much work

// Structs defining terminals
macro_rules! define_terminal
{
    ($name: ident) =>
    {
        pub struct $name<'a>
        {
            pub tok: &'a str
        }
        impl<'a> Graph<'a> for $name<'a>
        {
            fn get_str(&self) -> &'a str { self.tok }
            fn get_id(&self) -> String
            {
                format!("{:p}_{}", self.tok.as_ptr(), self.tok.len())
            }
            fn get_label(&self) -> String { format!("\"{}\"", self.tok) }
            fn to_dot_recurse(&self) -> String { String::new() }
        }
    };
}

define_terminal!(AgainToken);
define_terminal!(DeferToken);
define_terminal!(ForgetToken);
define_terminal!(NToken);
define_terminal!(PrintToken);
define_terminal!(ReadToken);
define_terminal!(UToken);

define_terminal!(PlusToken);
implement_alternations!(PlusToken, UnMathOpAlt, BinMathOpAlt);
define_terminal!(MinusToken);
implement_alternations!(MinusToken, UnMathOpAlt, BinMathOpAlt);

define_terminal!(StringToken);
implement_alternations!(StringToken, StringAlt);
define_terminal!(UnBoolOpToken);
define_terminal!(BinBoolOpToken);
define_terminal!(BinNumBoolOpToken);
define_terminal!(MathOpToken);
implement_alternations!(MathOpToken, BinMathOpAlt);

define_terminal!(CommaToken);
define_terminal!(LeftParensToken);
define_terminal!(RightParensToken);
define_terminal!(SemicolonToken);
define_terminal!(SharpToken);

pub struct NumberToken<'a>
{
    pub tok: &'a str,
    pub val: usize
}
impl<'a> Graph<'a> for NumberToken<'a>
{
    fn get_str(&self) -> &'a str { self.tok }
    fn get_id(&self) -> String
    {
        format!("{:p}_{}", self.tok.as_ptr(), self.tok.len())
    }
    fn get_label(&self) -> String
    {
        format!("\"{} ({})\"", self.tok, self.val)
    }
    fn to_dot_recurse(&self) -> String { String::new() }
}
implement_alternations!(NumberToken, AbsNumAlt);

// Structs defining nonterminals
macro_rules! define_nonterminal
{
    ($name: ident, $($field: ident, $fieldtype: ident),*) =>
    {
        pub struct $name<'a>
        {
            pub range: &'a str,
            $(pub $field: $fieldtype<'a>,)*
        }
        impl<'a> $name<'a>
        {
            pub fn new($($field: $fieldtype<'a>,)*) -> $name<'a>
            {
                let mut minptr : *const u8 = ptr::null();
                let mut minpos = std::usize::MAX;
                let mut maxpos = 0usize;
                $(
                    let s = $field.get_str();
                    let ptr = s.as_ptr();
                    let pos = ptr as usize;

                    if pos < minpos
                    {
                        minptr = ptr;
                        minpos = pos;
                    }
                    let endpos = pos + s.len();
                    if maxpos < endpos
                    {
                        maxpos = endpos;
                    }
                 )*
                let len = maxpos - minpos;
                if minptr.is_null() || len < 1
                {
                    panic!("Invalid range: {:p} len: {}", minptr, len);
                }
                let range : &'a str = unsafe {
                    let slice = std::slice::from_raw_parts(minptr,
                                                           len as usize);
                    std::str::from_utf8(slice).unwrap()
                };
                $name {
                    range,
                    $($field,)*
                }
            }
        }
        impl<'a> Graph<'a> for $name<'a>
        {
            fn get_str(&self) -> &'a str { self.range }
            fn get_id(&self) -> String
            {
                format!(concat!("{:p}_{}_", stringify!($name)),
                        self.range.as_ptr(), self.range.len())
            }
            fn get_label(&self) -> String
            {
                String::from(concat!("<<I>", stringify!($name), "</I>>"))
            }
            fn to_dot_recurse(&self) -> String
            {
                let mut res = String::new();
                let id = self.get_id();
                $(
                    res += format!("  \"{}\" -> \"{}\";\n",
                                   id, self.$field.get_id()).as_str();
                    res += self.$field.to_dot().as_str();
                )*
                res
            }
        }
    }
}
define_nonterminal!(Line, num, NumberToken,
                          stmt, Statement,
                          semi, SemicolonToken);
define_nonterminal!(AbsoluteNumber, num, AbsNumber);
implement_alternations!(AbsoluteNumber, NumberAlt);
define_nonterminal!(UnOpNumber, op, UnMathOp,
                                num, Number);
implement_alternations!(UnOpNumber, NumberAlt);
define_nonterminal!(BinOpNumber, num1, Number,
                                 op, BinMathOp,
                                 num2, Number);
implement_alternations!(BinOpNumber, NumberAlt);
define_nonterminal!(ParensNumber, lparen, LeftParensToken,
                                  num, Number,
                                  rparen, RightParensToken);
implement_alternations!(ParensNumber, NumberAlt);
define_nonterminal!(UnOpBoolean, op, UnBoolOpToken,
                                 boolean, Boolean);
implement_alternations!(UnOpBoolean, BoolAlt);
define_nonterminal!(BinOpBoolean, boolean1, Boolean,
                                  op, BinBoolOpToken,
                                  boolean2, Boolean);
implement_alternations!(BinOpBoolean, BoolAlt);
define_nonterminal!(BinOpNumBoolean, num1, Number,
                                     op, BinNumBoolOpToken,
                                     num2, Number);
implement_alternations!(BinOpNumBoolean, BoolAlt);
define_nonterminal!(ParensBoolean, lparen, LeftParensToken,
                                   boolean, Boolean,
                                   rparen, RightParensToken);
implement_alternations!(ParensBoolean, BoolAlt);
define_nonterminal!(N, keyword, NToken,
                       lparen, LeftParensToken,
                       num, Number,
                       rparen, RightParensToken);
implement_alternations!(N, AbsNumAlt);
define_nonterminal!(Read, keyword, ReadToken,
                          lparen, LeftParensToken,
                          rparen, RightParensToken);
implement_alternations!(Read, AbsNumAlt);
define_nonterminal!(LineOperations, lineops, LineOps);
implement_alternations!(LineOperations, StatementAlt);
define_nonterminal!(LineOp, slo, SingleLineOp);
implement_alternations!(LineOp, LineOpsAlt);
define_nonterminal!(CountLineOp, line, Number,
                                 sharp, SharpToken,
                                 count, Number);
implement_alternations!(CountLineOp, SingleLineOpAlt);
define_nonterminal!(LineOpList, lineop, LineOp,
                                comma, CommaToken,
                                list, LineOps);
implement_alternations!(LineOpList, LineOpsAlt);
define_nonterminal!(Again, keyword, AgainToken,
                           lparen, LeftParensToken,
                           boolean, Boolean,
                           rparen, RightParensToken,
                           statement, Statement);
implement_alternations!(Again, StatementAlt);
define_nonterminal!(Defer, keyword, DeferToken,
                           lparen, LeftParensToken,
                           boolean, Boolean,
                           rparen, RightParensToken,
                           statement, Statement);
implement_alternations!(Defer, StatementAlt);
define_nonterminal!(Forget, keyword, ForgetToken,
                            lparen, LeftParensToken,
                            boolean, Boolean,
                            rparen, RightParensToken,
                            statement, Statement);
implement_alternations!(Forget, StatementAlt);
define_nonterminal!(Print, keyword, PrintToken,
                           lparen, LeftParensToken,
                           string, String_,
                           rparen, RightParensToken);
implement_alternations!(Print, StatementAlt);
define_nonterminal!(Concat, str1, String_,
                            op, PlusToken,
                            str2, String_);
implement_alternations!(Concat, StringAlt);
define_nonterminal!(U, keyword, UToken,
                       lparen, LeftParensToken,
                       num, AbsNumber,
                       rparen, RightParensToken);
implement_alternations!(U, StringAlt);

// Nonterminals for conversions
macro_rules! define_conversion
{
    ($name: ident, $from: ident, $to: ty, $varname: ident) =>
    {
        pub struct $name<'a>
        {
            pub $varname: $from<'a>
        }
        impl<'a> Graph<'a> for $name<'a>
        {
            fn get_str(&self) -> &'a str { self.$varname.get_str() }
            fn get_id(&self) -> String
            {
                self.$varname.get_id() + concat!('_', stringify!($name))
            }
            fn get_label(&self) -> String
            {
                String::from(concat!("<<I>", stringify!($from), " ⮕ ",
                                     stringify!($to), "</I>>"))
            }
            fn to_dot_recurse(&self) -> String
            {
                format!("  \"{}\" -> \"{}\";\n", self.get_id(),
                        self.$varname.get_id())
                    + self.$varname.to_dot().as_str()
            }
        }
    }
}
define_conversion!(NumToBool, Number, Boolean, num);
implement_alternations!(NumToBool, BoolAlt);
define_conversion!(NumToLineOp, Number, SingleLineOp, num);
implement_alternations!(NumToLineOp, SingleLineOpAlt);
define_conversion!(NumToString, Number, String, num);
implement_alternations!(NumToString, StringAlt);
pub struct StringToNum<'a>
{
    pub string: String_<'a>
}
impl<'a> Graph<'a> for StringToNum<'a>
{
    fn get_str(&self) -> &'a str { self.string.get_str() }
    fn get_id(&self) -> String { self.string.get_id() + "_StringToNum" }
    fn get_label(&self) -> String { String::from("<<I>String ⮕ Number</I>>") }
    fn to_dot_recurse(&self) -> String
    {
        format!("  \"{}\" -> \"{}\";\n", self.get_id(), self.string.get_id())
            + self.string.to_dot().as_str()
    }
}
implement_alternations!(StringToNum, NumberAlt);
