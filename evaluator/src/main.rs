#![allow(dead_code)]

mod utilities;

use common::*;

// mod q;

fn main() {
    use std::time::Instant;
    use std::path;
    use std::io::prelude::*;

    let now = || Instant::now();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Wrong number of arguments, expected 1, got {}", args.len() - 1);
        std::process::exit(1);
    }

    let fpath = path::PathBuf::from(&args[1]);
    let mut file = std::fs::File::open(fpath).unwrap();
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf).unwrap();

    let program: ScpGraph = bincode::deserialize(&buf).unwrap();
}
