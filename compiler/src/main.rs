#![allow(dead_code)]

#[macro_use]
extern crate nom;
pub mod parser;
pub mod desugar;

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
    let prog1 = r#"
    (proclaim-threshold 0.8)
    (let [d (decision (one-of 1 2))
          dist (normal 1 1.5)
          val (sample dist)]
        (constrain = val dist))
    "#;

    let parsed = match parser::parse_program(prog1) {
        Ok((_, p)) => p,
        Err(e) => match e {
            nom::Err::Failure(e) => {
                eprintln!("Parsing error:");
                eprintln!("{}", nom::error::convert_error(prog1, e));
                std::process::exit(1);
            },
            _ => unreachable!()
        }
    };

    let desugared = match desugar::desugar(&parsed) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(2);
        }
    };

    println!("{:?}", desugared);
}
