#![allow(dead_code)]

pub mod expression;
pub mod primitive;
pub mod eqmap;
mod utilities;
pub mod tree;
pub mod distribution;

pub mod common {
    use smol_str::SmolStr;

    pub type VariableName = SmolStr;

    pub use crate::primitive::*;
}


fn main() {
    println!("Hello, world!");
}
