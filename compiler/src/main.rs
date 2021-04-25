#![allow(dead_code)]

pub mod parser;
pub mod desugar;
pub mod graph;

use nom::error::VerboseError;
use std::path;

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
    use std::time::Instant;

    let now = || Instant::now();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Wrong number of arguments, expected 1, got {}", args.len() - 1);
        std::process::exit(3);
    }

    let fpath = path::PathBuf::from(&args[1]);

    let program = std::fs::read_to_string(fpath).expect(&format!("Could not find file {:?}", &args[1]));

    let t0 = now();

    let parsed = match parser::parse_program(&program) {
        Ok((_, p)) => p,
        Err(e) => match e {
            nom::Err::Failure(e) => {
                eprintln!("Parsing error:");
                eprintln!("{}", nom::error::convert_error(program.as_str(), e));
                std::process::exit(1);
            },
            _ => unreachable!()
        }
    };

    let t1 = now();

    let desugared = match desugar::desugar(&parsed) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error while desugaring: {:?}", e);
            std::process::exit(2);
        }
    };

    let t2 = now();

    desugar::pretty_print(&desugared.body);

    println!("\nParsing took {:?}", t1.duration_since(t0));
    println!("Desugaring took {:?}", t2.duration_since(t1));
}
