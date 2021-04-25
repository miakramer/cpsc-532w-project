#![allow(dead_code)]

#[macro_use]
extern crate nom;
pub mod parser;

use nom::error::VerboseError;
use parser::*;

fn prettyprint<'a, T : std::fmt::Debug>(i: &'static str, val: nom::IResult<&'a str, T, VerboseError<&str>>) {
    match val {
        Ok(v) => println!("✓ {:?}", v.1),
        Err(e) => match e {
            nom::Err::Error(e) => println!("(recoverable) {}", nom::error::convert_error(i, e)),
            nom::Err::Incomplete(n) => println!("(incomplete) {:?}", n),
            nom::Err::Failure(e) => println!("(failure) {}", nom::error::convert_error(i, e)),
        }
    }
}

pub fn main() {
    println!("{:?}", c("1234"));
    println!("{:?}", c("12.34"));
    println!("{:?}", c("true"));

    println!("{:?}", identifier("foobar"));
    println!("{:?}", identifier("_"));

    println!("{:?}", domain_name("one-of"));

    println!("{:?}", proclaim_threshold("(proclaim-threshold 0.5)"));

    println!("{:?}", parse_expr("(begin 1 2 3)"));
    println!("{:?}", parse_expr("(sample bar)"));
    let i = "(if true\n    (sample bar)\n    (observe foo baz))";
    prettyprint(i, parse_expr(i));

    let i = r#"(let
        [v1 (sample e) v2 (sample f)] (begin foo bar))"#;
    prettyprint(i, parse_expr(i));

    let i = "(foo-bar (sample e) (observe f g))";
    prettyprint(i, parse_expr(i));

    let i = "(defn f [a b c] (foo a b c))";
    prettyprint(i, parse_defn(i));

    let i = r#"
    (proclaim-threshold 0.8)

    (defn f [a b c] (foo a b c))

    (defn g [_] (abs -1.5))

    (apply f (vector 1 2 3))
    "#;
    prettyprint(i, parse_program(i));
}
