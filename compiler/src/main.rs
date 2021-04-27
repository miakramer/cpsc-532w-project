#![allow(dead_code)]

pub mod parser;
pub mod desugar;
pub mod partial_eval;
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
    use std::io::Write;

    let now = || Instant::now();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Wrong number of arguments, expected 2, got {}", args.len() - 1);
        std::process::exit(1);
    }

    let fpath = path::PathBuf::from(&args[1]);
    let opath = path::PathBuf::from(&args[2]);

    let program = std::fs::read_to_string(fpath).expect(&format!("Could not find file {:?}", &args[1]));

    let t0 = now();

    let parsed = match parser::parse_program(&program) {
        Ok((_, p)) => p,
        Err(e) => match e {
            nom::Err::Failure(e) => {
                eprintln!("Parsing error:");
                eprintln!("{}", nom::error::convert_error(program.as_str(), e));
                std::process::exit(2);
            },
            _ => unreachable!()
        }
    };

    let t1 = now();

    // println!("{:?}", parsed.body);

    let desugared = match desugar::desugar(&parsed) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error while desugaring: {:?}", e);
            std::process::exit(3);
        }
    };

    let t2 = now();

    println!("\n    Desugared:\n");
    // println!("{:?}", desugared.body);
    desugar::pretty_print(&desugared.body);

    let t3 = now();

    let evald = match partial_eval::partial_eval(&desugared.body) {
        Ok(e) => e,
        Err(e) => {
            eprint!("Error while partially evaluating: ");
            match e {
                partial_eval::PartialEvalErr::Bubble(s) => {
                    eprintln!("{}", s);
                }
                partial_eval::PartialEvalErr::Undefined(i) => {
                    eprintln!("Name {:?} undefined", i);
                }
                partial_eval::PartialEvalErr::Placeholder => {
                    eprintln!("Encountered placeholder value.");
                }
                partial_eval::PartialEvalErr::Observe => {
                    eprintln!("Observe not yet supported.");
                }
                partial_eval::PartialEvalErr::InvalidProbability => {
                    eprintln!("Probabilities must be in [0, 1]");
                }
            }
            std::process::exit(4);
        }
    };

    let t4 = now();

    println!("\n    Partially evaluated:\n");

    // println!("{:?}", &evald);
    partial_eval::pretty_print(&evald);

    
    let t5 = now();
    let g = graph::compile_graph(&evald);
    let t6 = now();

    println!("\n==============\n    Graph:\n==============\n");
    graph::pretty_print(&g);

    println!("\nParsing took            {:?}", t1.duration_since(t0));
    println!("Desugaring took         {:?}", t2.duration_since(t1));
    println!("Partial evaluation took {:?}", t4.duration_since(t3));
    println!("Graph compilation took  {:?}", t6.duration_since(t5));

    println!("Saving to {:?}…", &opath);
    let serialized = bincode::serialize(&g).unwrap();
    let mut f = std::fs::File::create(opath).unwrap();
    f.write_all(&serialized).unwrap();
    println!("Done.");
}
