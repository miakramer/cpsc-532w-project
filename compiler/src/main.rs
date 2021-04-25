#![allow(dead_code)]

#[macro_use]
extern crate nom;
pub mod parser;
pub mod desugar;
pub mod graph;

use nom::error::VerboseError;

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
    (defn f [x] (+ x 2))
    (defn g [x] (- x (f x)))
    (let [d (decision (one-of 1 (f 1)))
          dist (normal 1 1.5)
          val (sample dist)
          x 5
          _ (+ 2 (g 3))]
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
            eprintln!("Error while desugaring: {:?}", e);
            std::process::exit(2);
        }
    };

    desugar::pretty_print(&desugared.body);
}
