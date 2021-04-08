
use libc::c_double;

use num_traits::Float;

extern "C" {
    fn lgamma(x: c_double) -> c_double;
}

pub trait BasicFloat {
    fn as_f64(self) -> f64;
    fn as_f32(self) -> f32;
    fn to_self_f32(f: f32) -> Self;
    fn to_self_f64(f: f64) -> Self;
}

impl BasicFloat for f32 {
    fn as_f32(self) -> f32 {self}
    fn as_f64(self) -> f64 {self as f64}
    fn to_self_f32(f: f32) -> Self { f }
    fn to_self_f64(f: f64) -> Self { f as f32 }
}

impl BasicFloat for f64 {
    fn as_f32(self) -> f32 {self as f32}
    fn as_f64(self) -> f64 {self}
    fn to_self_f32(f: f32) -> Self { f as f64 }
    fn to_self_f64(f: f64) -> Self { f }
}

pub fn ln_gamma<F : BasicFloat + Float>(f: F) -> F {
    BasicFloat::to_self_f64(unsafe { lgamma(f.as_f64()) })
}

pub fn ln<F : Float>(f: F) -> F {
    f.ln()
}

pub fn digamma<F : special::Gamma>(f: F) -> F {
    f.digamma()
}
