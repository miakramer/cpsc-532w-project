use ndarray::prelude::*;
use im::Vector;
use crate::eqmap::*;
use num_traits::*;
use std::convert::TryFrom;
use serde::{Serialize, Deserialize};
use rand::prelude::*;

pub type PHashMap = EqMap<Primitive, Primitive>;

pub trait Support {
    fn cardinality(&self) -> usize;
}

pub trait Sample {
    type Output;
    fn sample(&self, rng: &mut ThreadRng) -> Self::Output;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Distribution {
    Dirac{center: f64},
    Kronecker{center: i128},
    UniformContinuous{a: f64, b: f64},
    UniformDiscrete{a: i128, b: i128},
    Categorical{weights: Array1<f32>},
    MappedCategorical{weights: Array1<f32>, values: Vec<Primitive>},
    Normal{mu: f64, sigma: f64},
    Cauchy{median: f64, scale: f64},
    Beta{alpha: f64, beta: f64},
    Dirichlet{weights: Array1<f32>},
    Exponential{lambda: f64},
    Gamma{shape: f64, rate: f64},
    Bernoulli{p: f64},
    Binomial{n: u64, p: f64},
}


impl Support for Distribution {
    fn cardinality(&self) -> usize {
        match self {
            Self::Dirac{center: _} => 1,
            Self::Kronecker{center: _} => 1,
            Self::UniformDiscrete{a, b} => (b - a) as usize,
            Self::Categorical{weights} => weights.len(),
            Self::MappedCategorical{weights, values: _} => weights.len(),
            Self::Bernoulli{p: _} => 2,
            _ => todo!()
        }
    }
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Domain {
    OneOf(Vec<Primitive>),
    IntRange(i128, i128),
}


impl Support for Domain {
    fn cardinality(&self) -> usize {
        match self {
            Self::OneOf(v) => v.len(),
            Self::IntRange(a, b) => (b - a) as usize,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Primitive {
    Boolean(bool),
    Float(f64),
    Int(i128),
    // String(String),
    Vector(Vector<Primitive>),
    HashMap(PHashMap),
    Distribution(Distribution),
    Domain(Domain),
    // optimizations
    EvaluatedVector(Array1<f64>),
}


impl PartialEq for Primitive {
    fn eq(&self, other: &Primitive) -> bool {
        match self {
            Self::Boolean(l) => {
                if let Primitive::Boolean(r) = other {
                    l == r
                } else { false }
            }
            Self::Int(l) => {
                if let Ok(r) = i128::try_from(other) {
                    *l == r
                } else { false }
            }
            Self::Float(l) => {
                if let Ok(r) = f64::try_from(other) {
                    *l == r
                } else { false }
            }
            // Self::String(l) => {
            //     if let Self::String(r) = other { l == r} else { false }
            // }
            Self::Vector(l) => {
                if let Self::Vector(r) = other { l == r} else { false }
            }
            Self::HashMap(l) => {
                if let Self::HashMap(r) = other { l == r} else { false }
            }
            // Self::DistributionType(l) => {
            //     if let Self::DistributionType(r) = other { l == r} else { false }
            // }
            Self::Distribution(l) => {
                if let Self::Distribution(r) = other { l == r} else { false }
            }
            // Self::Builtin(l) => {
            //     if let Self::Builtin(r) = other { l == r} else { false }
            // }
            Self::EvaluatedVector(l) => {
                if let Self::EvaluatedVector(r) = other { l == r} else { false }
            }
            Self::Domain(l) => {
                if let Self::Domain(r) = other { l == r } else { false }
            }
            // Self::Observed => {
            //     if let Self::Observed = other { true } else { false }
            // }
        }
    }
}


impl Primitive {
    pub fn is_vector(&self) -> bool  {
        match self {
            Self::Vector(_) | Self::EvaluatedVector(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Self::Boolean(_) | Self::Int(_) | Self::Float(_) => true,
            _ => false,
        }
    }
}

macro_rules! _primfrom {
    ($match:expr, $type:ty) => {
        impl From<$type> for Primitive {
            fn from(t: $type) -> Self {
                $match(t)
            }
        }
    };
    ($match:expr, $type:ty, $astype:ty) => {
        impl From<$type> for Primitive {
            fn from(t: $type) -> Self {
                $match(t as $astype)
            }
        }
    };
    (int $type:ty) => {
        _primfrom!(Primitive::Int, $type, i128);
    };
    (int $type:ty, $($types:ty),+) => {
        _primfrom!(Primitive::Int, $type, i128);
        _primfrom!(int $($types),+);
    };
}

_primfrom!(Primitive::Float, f64, f64);
_primfrom!(Primitive::Float, f32, f64);
_primfrom!(int u8,u16,u32,u64,i8,i16,i32,i64,i128,usize,isize);

// _primfrom!(Primitive::String, String);
_primfrom!(Primitive::Vector, Vector<Primitive>);
_primfrom!(Primitive::HashMap, PHashMap);
_primfrom!(Primitive::EvaluatedVector, Array1<f64>);
_primfrom!(Primitive::Boolean, bool);
_primfrom!(Primitive::Domain, Domain);
_primfrom!(Primitive::Distribution, Distribution);
// _primfrom!(Primitive::DistributionType, DistributionType);

// impl From<&str> for Primitive {
//     fn from(s: &str) -> Self {
//         Primitive::String(String::from(s))
//     }
// }

impl From<Vec<Primitive>> for Primitive {
    fn from(v: Vec<Primitive>) -> Self {
        Primitive::Vector(Vector::from(v))
    }
}

impl<T : Copy + AsPrimitive<f64> + Float> From<Vec<T>> for Primitive {
    fn from(t: Vec<T>) -> Primitive {
        t.into_iter()
            .map(|x| x.as_())
            .collect::<Array1<f64>>()
            .into()
    }
}

// impl<T : Copy + Into<Primitive>> From<&T> for Primitive {
//     fn from(t: &T) -> Primitive {
//         (*t).into()
//     }
// }

// impl From<&serde_json::Number> for Primitive {
//     fn from(n: &serde_json::Number) -> Primitive {
//         if n.is_u64() {
//             Primitive::from(n.as_u64().unwrap() as i128)
//         } else if n.is_i64() {
//             Primitive::from(n.as_i64().unwrap() as i128)
//         } else if n.is_f64() {
//             Primitive::from(n.as_f64().unwrap())
//         } else {
//             panic!("Unknown number type found in JSON: {:?}", n);
//         }
//     }
// }


pub trait TryFromRef<T : ?Sized> {
    fn try_from_ref(t: &T) -> Option<&Self>;
}

macro_rules! _tryasref {
    ($type:ty, $match:path) => {
        impl TryFromRef<Primitive> for $type {
            fn try_from_ref(p: &Primitive) -> Option<&$type> {
                match p {
                    $match(x) => Some(x),
                    _ => None,
                }
            }
        }
    };
    ($type:ty, $conv:ident, $primary:path, $alt:path) => {
        impl TryFromRef<Primitive> for $type {
            fn try_from_ref(p: &Primitive) -> Option<&$type> {
                match p {
                    $primary(x) => Some(x),
                    $alt(x) => Some(&x.$conv()),
                    _ => None,
                }
            }
        }
    };
}

_tryasref!(bool, Primitive::Boolean);
_tryasref!(f64, Primitive::Float);
_tryasref!(i128, Primitive::Int);
// _tryasref!(String, Primitive::String);
// _tryasref!(str, Primitive::String);
_tryasref!(Vector<Primitive>, Primitive::Vector);
_tryasref!(Distribution, Primitive::Distribution);
// _tryasref!(Builtin, Primitive::Builtin);

// _tryasref!(f64, to_f64, Primitive::Float, Primitive::Int);
// _tryasref!(i128, to_i128, Primitive::Float);


impl<T> TryFrom<&Primitive> for Array1<T>
where
    T : FromPrimitive + Float + TryFromRef<Primitive> + Copy
{
    type Error = ();

    fn try_from(p: &Primitive) -> Result<Array1<T>, Self::Error> {
        match p {
            Primitive::EvaluatedVector(v) => {
                let mut out = Array1::zeros(v.len());
                for (i, el) in v.iter().enumerate() {
                    if let Some(el) = T::from_f64(*el) {
                        out[i] = el;
                    } else {
                        return Err(())
                    }
                }
                Ok(out)
            }
            Primitive::Vector(v) => {
                let mut out = Array1::zeros(v.len());
                for (i, el) in v.iter().enumerate() {
                    if let Some(el) = T::try_from_ref(el) {
                        out[i] = *el;
                    } else {
                        return Err(())
                    }
                }
                Ok(out)
            }
            _ => Err(())
        }
    }
}


pub trait TryAsRef<T> {
    fn try_as_ref(&self) -> Option<&T>;
}

impl<T : TryFromRef<Primitive>> TryAsRef<T> for Primitive {
    fn try_as_ref(&self) -> Option<&T> {
        T::try_from_ref(self)
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum PrimitiveError {
    CannotConvert,
    Incompatible,
}

macro_rules! _tryfrom {
    ($match1:path, $match2:path, $type:ty, $totype:ident) => {
        impl TryFrom<&Primitive> for $type {
            type Error = PrimitiveError;

            fn try_from(p: &Primitive) -> Result<$type, Self::Error> {
                match p {
                    $match1(x) => match x.$totype() {
                        Some(y) => Ok(y),
                        None => Err(PrimitiveError::CannotConvert),
                    },
                    $match2(x) => match x.$totype() {
                        Some(y) => Ok(y),
                        None => Err(PrimitiveError::CannotConvert),
                    },
                    Primitive::Boolean(x) => match (*x as u8).$totype() {
                        Some(y) => Ok(y),
                        None => Err(PrimitiveError::CannotConvert),
                    },
                    _ => Err(PrimitiveError::Incompatible),
                }
            }
        }
        impl TryFrom<Primitive> for $type {
            type Error = PrimitiveError;

            fn try_from(p: Primitive) -> Result<$type, Self::Error> {
                TryFrom::try_from(&p)
            }
        }
    };
    (numeric $type:ty, $totype:ident) => {
        _tryfrom!(Primitive::Int, Primitive::Float, $type, $totype);
    };
    ($match:path, $type:ty) => {
        impl TryFrom<Primitive> for $type {
            type Error = PrimitiveError;

            fn try_from(p: Primitive) -> Result<$type, Self::Error> {
                match p {
                    $match(x) => Ok(x.into()),
                    _ => Err(PrimitiveError::Incompatible),
                }
            }
        }
    };
}

_tryfrom!(numeric u8, to_u8);
_tryfrom!(numeric u16, to_u16);
_tryfrom!(numeric u32, to_u32);
_tryfrom!(numeric u64, to_u64);
_tryfrom!(numeric usize, to_usize);
_tryfrom!(numeric i8, to_i8);
_tryfrom!(numeric i16, to_i16);
_tryfrom!(numeric i32, to_i32);
_tryfrom!(numeric i64, to_i64);
_tryfrom!(numeric isize, to_isize);
_tryfrom!(numeric i128, to_i128);
_tryfrom!(numeric f64, to_f64);
_tryfrom!(numeric f32, to_f32);

// _tryfrom!(Primitive::String, String);
_tryfrom!(Primitive::Vector, Vector<Primitive>);
_tryfrom!(Primitive::Distribution, Distribution);
// _tryfrom!(Primitive::Builtin, Builtin);
_tryfrom!(Primitive::HashMap, EqMap<Primitive, Primitive>);
_tryfrom!(Primitive::EvaluatedVector, Array1<f64>);

impl TryFrom<&Primitive> for bool {
    type Error = PrimitiveError;

    fn try_from(p: &Primitive) -> Result<bool, Self::Error> {
        match p {
            Primitive::Boolean(x) => Ok(*x),
            _ => Err(PrimitiveError::Incompatible),
        }
    }
}

impl TryFrom<Primitive> for bool {
    type Error = PrimitiveError;

    fn try_from(p: Primitive) -> Result<bool, Self::Error> {
        match p {
            Primitive::Boolean(x) => Ok(x),
            _ => Err(PrimitiveError::Incompatible),
        }
    }
}

impl TryFrom<&Primitive> for Array1<f32> {
    type Error = PrimitiveError;

    fn try_from(p: &Primitive) -> Result<Self, Self::Error> {
        match p {
            Primitive::EvaluatedVector(v) => Ok(v.mapv(|x| x as f32)),
            _ => Err(PrimitiveError::CannotConvert),
        }
    }
}

pub fn integral_pair(p1: &Primitive, p2: &Primitive) -> Option<(i128, i128)> {
    let l = match p1 {
        Primitive::Int(i) => *i,
        Primitive::Boolean(b) => *b as i128,
        _ => {return None;}
    };
    let r = match p2 {
        Primitive::Int(i) => *i,
        Primitive::Boolean(b) => *b as i128,
        _ => {return None;}
    };
    Some((l, r))
}


pub fn try_pair<'a, T : TryFrom<&'a Primitive>>(p1: &'a Primitive, p2: &'a Primitive) -> Option<(T, T)> {
    if let Ok(left) = T::try_from(p1) {
        if let Ok(right) = T::try_from(p2) {
            return Some((left, right));
        }
    }
    None
}

// use ansi_term::Colour::*;

// impl Primitive {
//     pub fn pretty_print(&self, indentation: usize) {
//         match self {
//             Primitive::Observed => print!("{{}}{}", Green.paint("::Observed")),
//             Primitive::Int(i) => print!("{}{}", i, Green.paint("::Int")),
//             Primitive::Float(f) => print!("{}{}", f, Green.paint("::Float")),
//             Primitive::Boolean(b) => print!("{}{}", b, Green.paint("::Boolean")),
//             Primitive::Builtin(b) => print!("{}{}", b.name(), Green.paint("::Builtin")),
//             // Primitive::DistributionType(d) => print!("{:?}{}", d, Green.paint("::DistributionType")),
//             Primitive::Distribution(_d) => {
//                 // print!("({}", d.distribution.name());
//                 // for arg in &d.arguments {
//                 //     print!(" ");
//                 //     arg.pretty_print(indentation);
//                 // }
//                 // print!("){}", Green.paint("::Distribution"))
//             },
//             Primitive::Vector(v) => {
//                 print!("[");
//                 for el in v {
//                     print!(" ");
//                     el.pretty_print(indentation + 1);
//                 }
//                 print!("]{}", Green.paint("::Vector"));
//             },
//             Primitive::EvaluatedVector(v) => {
//                 print!("[{{evaluated}}");
//                 for el in v {
//                     print!(" {}", el);
//                 }
//                 print!("]{}", Green.paint("::Vector"));
//             }
//             Primitive::HashMap(e) => {
//                 print!("(hash-map");
//                 for (k, v) in e {
//                     print!("  ");
//                     k.pretty_print(indentation + 1);
//                     print!(" ");
//                     v.pretty_print(indentation + 1);
//                     print!("){}", Green.paint("::HashMap"))
//                 }
//             }
//             Primitive::String(s) => {
//                 print!("{:?}{}", s, Green.paint("::String"))
//             }
//         }
//     }
// }