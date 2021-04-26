use im::Vector;
use ndarray::prelude::*;
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use common::*;
use common::primitives::*;

macro_rules! assert_num_args {
    ($name:expr, $args:expr, $len:expr) => {
        if $args.len() != $len {
            return Err(format!("{} requires {} arguments but got {}", $name, $len, $args.len()));
        }
    };
}

macro_rules! _n {
    ($newn:ident, $n:expr, $l:expr, $v:expr) => {
        let mut $newn = $n;
        if $n < 0 {
            $newn = $l as isize - $n - 2;
        }
        if $n >= $l as isize {
            panic!(
                "Out of bounds access in vector (at {} in vector with length {}: {:?})",
                $n, $l, $v
            );
        }
    };
}

fn _get_nth(builtin: Builtin, args: &[Primitive], n: isize) -> Result<Primitive, String> {
    match &args[0] {
        Primitive::Vector(v) => {
            let l = v.len();
            _n!(n_, n, l, v);
            Ok(v[n_ as usize].clone())
        }
        Primitive::EvaluatedVector(v) => {
            _n!(n_, n, v.len(), v);
            Ok(Primitive::from(v[n_ as usize] as f64))
        }
        _ => Err(format!(
            "{:?} requries a `vector` argument, but got {:?}.",
            builtin, args[0]
        )),
    }
}

pub fn eval_builtin(builtin: Builtin, args: &[Primitive]) -> Result<Primitive, String> {
    match builtin {
        Builtin::First => {
            assert_num_args!("(first v)", &args, 1);
            _get_nth(builtin, args, 0)
        }
        Builtin::Second => {
            assert_num_args!("(second v)", &args, 1);
            _get_nth(builtin, args, 1)
        }
        Builtin::Last => {
            assert_num_args!("(last v)", &args, 1);
            _get_nth(builtin, args, -1)
        }
        Builtin::Rest => {
            assert_num_args!("(rest v)", &args, 1);
            match &args[0] {
                Primitive::Vector(v) => {
                    let mut v = v.clone();
                    v.pop_front();
                    Ok(Primitive::from(v))
                }
                Primitive::EvaluatedVector(v) => {
                    let mut new = Array1::zeros(v.len() - 1);
                    for i in 1..(v.len()) {
                        new[i - 1] = v[i];
                    }
                    Ok(Primitive::EvaluatedVector(new))
                }
                _ => Err(String::from("(rest xs) requires vector argument")),
            }
        }
        Builtin::Get => {
            assert_num_args!("(get collection key)", args, 2);
            if args[0].is_vector() {
                let n = usize::try_from(&args[1]).expect("(get (vector …) n) requires `n` numeric.");
                _get_nth(builtin, args, n as isize)
            } else if let Ok(h) = PHashMap::try_from(args[0].clone()) {
                Ok(h.get(&args[1]).expect(&format!("Key {:?} not found for {:?}", &args[1], &h)).clone())
            } else {
                Err(format!("(get collection key) requires collection to be either vector or hash map, but was {:?}", args[0]))
            }
        }
        Builtin::Put => {
            assert_num_args!("(put collection key value)", args, 3);
            match &args[0] {
                Primitive::Vector(v) => {
                    let n = usize::try_from(&args[1]).expect("(get vector n) requires n integral.");
                    Ok(Primitive::from(v.update(n, args[2].clone())))
                },
                Primitive::EvaluatedVector(v) => {
                    let n = usize::try_from(&args[1]).expect("(get vector n) requires n integral.");

                    if let Ok(f) = f64::try_from(&args[2]) {
                        let mut v = v.clone();
                        v[n] = f;
                        Ok(Primitive::EvaluatedVector(v))
                    } else {
                        let mut new = Vector::from_iter(v.into_iter().map(|x| Primitive::from(*x)));
                        new[n] = args[2].clone();

                        Ok(Primitive::Vector(new))
                    }
                }
                Primitive::HashMap(h) => Ok(Primitive::from(h.update(args[1].clone(), args[2].clone()))),
                _ => Err(format!("(put) requres vector or hash-map, but got {:?}", &args[1]))
            }
        }
        Builtin::Append => {
            assert_num_args!("(append v x)", args, 2);
            match &args[0] {
                Primitive::Vector(v) => {
                    let mut v = v.clone();
                    v.push_back(args[1].clone());
                    Ok(Primitive::Vector(v))
                }
                Primitive::EvaluatedVector(v) => {
                    if let Ok(f) = f64::try_from(&args[1]) {
                        let mut nv = Array1::zeros(v.len() + 1);
                        for i in 0..v.len() {
                            nv[i] = v[i];
                        }
                        nv[v.len()] = f;
                        Ok(Primitive::from(nv))
                    } else {
                        let mut new = Vector::from_iter(v.into_iter().map(|x| Primitive::from(*x)));
                        new.push_back(args[2].clone());
                        Ok(Primitive::Vector(new))
                    }
                },
                _ => Err(format!("(append) requires vector, but got {:?}", args[0]))
            }
        }
        Builtin::Conj => match &args[0] {
            Primitive::Vector(v) => {
                let mut v = v.clone();
                // v.reserve(v.len() + args.len());
                for a in &args[1..] {
                    v.push_front(a.clone())
                }
                Ok(Primitive::from(v))
            }
            Primitive::EvaluatedVector(v) => {
                if args.len() == 1 {
                    return Ok(Primitive::EvaluatedVector(v.clone()))
                }
                let all_num = (&args[1..]).iter().all(|x| x.is_number());
                if all_num {
                    let mut new = Array1::zeros(v.len() + args.len() - 1);
                    let args = &args[1..];
                    for i in 0..args.len() {
                        new[i] = (&args[i]).try_into().expect(&format!("Converting {:?} to f64 failed.", &args[i]));
                    }
                    for i in 0..v.len() {
                        new[args.len() + i] = v[i];
                    }
                    Ok(Primitive::from(new))
                } else {
                    let mut new = Vec::with_capacity(v.len() + args.len() - 1);
                    let args = &args[1..];
                    for i in 0..args.len() {
                        new.push(args[i].clone());
                    }
                    for i in 0..v.len() {
                        new.push(Primitive::from(v[i] as f64));
                    }
                    
                    Ok(Primitive::from(new))
                }
            }
            _ => Err(format!("(conj) requires vector, but got {:?}", args[0]))
        },
        Builtin::Cons => {
            assert_num_args!("(cons x v)", args, 2);
            match &args[1] {
                Primitive::Vector(v) => {
                    let mut v = v.clone();
                    v.push_front(args[1].clone());
                    Ok(Primitive::Vector(v))
                }
                Primitive::EvaluatedVector(v) => {
                    if let Ok(f) = f64::try_from(&args[0]) {
                        let mut new = Array1::zeros(v.len() + 1);
                        for i in 0..v.len() {
                            new[i + 1] = v[i];
                        }
                        new[0] = f;
                        Ok(Primitive::from(new))
                    } else {
                        let mut new = Vec::with_capacity(v.len() + 1);
                        new.push(args[0].clone());
                        for i in 0..v.len() {
                            new.push(Primitive::from(v[i]));
                        }
                        Ok(Primitive::from(new))
                    }
                },
                _ => panic!("(append) requires vector, but got {:?}", args[0]),
            }
        }
        Builtin::IsEmpty => {
            assert_num_args!("(empty? v)", args, 1);
            Ok(Primitive::from(match &args[0] {
                Primitive::Vector(v) => v.len() == 0,
                Primitive::EvaluatedVector(v) => v.len() == 0,
                Primitive::HashMap(h) => h.len() == 0,
                _ => {return Err(format!("(empty?) not defined for {:?}", &args[0]));}
            }))
        }
        Builtin::Vector => {
            let all_num = args.iter().all(|x| x.is_number());
            if all_num {
                Ok(Primitive::from(Array1::from_iter(args.iter().map(|x| x.try_into().unwrap()))))
            } else {
                Ok(Primitive::from(Vec::from_iter(args.iter().cloned())))
            }
        }
        Builtin::HashMap => {
            if args.len() % 2 != 0 {
                return Err(String::from("(hash-map ...) requires an even number of arguments."));
            }
            let iter = args.iter()
                .step_by(2)
                .cloned()
                .zip(args.iter()
                    .skip(1)
                    .step_by(2)
                    .cloned());
            Ok(Primitive::from(PHashMap::from_iter(iter)))
        }
        Builtin::Add | Builtin::Sub | Builtin::Mul | Builtin::Pow => {
            assert_num_args!("(binary-op x y)", args, 2);
            if let Some((l, r)) = integral_pair(&args[0], &args[1]) {
                Ok(Primitive::from(match builtin {
                    Builtin::Add => l + r,
                    Builtin::Sub => l - r,
                    Builtin::Mul => l * r,
                    Builtin::Pow => l.pow(r as u32),
                    _ => unreachable!(),
                }))
            } else if let Some((l, r)) = try_pair::<f64>(&args[0], &args[1]) {
                Ok(Primitive::from(match builtin {
                    Builtin::Add => l + r,
                    Builtin::Sub => l - r,
                    Builtin::Mul => l * r,
                    Builtin::Pow => l.powf(r),
                    _ => unreachable!(),
                }))
            } else {
                Err(format!("(binary-op x y) requires x, y numeric, but they are {:?}, {:?}", &args[0], &args[1]))
            }
        }
        Builtin::Div => {
            assert_num_args!("(/ x y)", args, 2);
            if let Some((l, r)) = try_pair::<f64>(&args[0], &args[1]) {
                Ok(Primitive::from(l / r))
            } else {
                Err(format!("(/ x y) requires x, y numeric, but they are {:?}, {:?}", args[0], args[1]))
            }
        }
        Builtin::IsLess | Builtin::IsEqual | Builtin::IsGreater => {
            assert_num_args!("(cmp x y)", args, 2);
            if let Some((l, r)) = try_pair::<f64>(&args[0], &args[1]) {
                Ok(Primitive::from(match builtin {
                    Builtin::IsLess => l < r,
                    Builtin::IsEqual => l == r,
                    Builtin::IsGreater => l > r,
                    _ => unreachable!()
                }))
            } else {
                Err(format!("(cmp x y) requires x, y numeric, but they are {:?}, {:?}", args[0], args[1]))
            }
        }
        Builtin::And | Builtin::Or => {
            assert_num_args!("(logical x y)", args, 2);
            if let Some((l, r)) = try_pair(&args[0], &args[1]) {
                Ok(Primitive::from(match builtin {
                    Builtin::And => l && r,
                    Builtin::Or =>  l || r,
                    _ => unreachable!()
                }))
            } else {
                Err(format!("(logical-cmp x y) requires x, y logical, but they are {:?}, {:?}", args[0], args[1]))
            }
        }
        Builtin::Sqrt => {
            assert_num_args!("(sqrt x)", args, 1);
            if let Ok(f) = f64::try_from(&args[0]) {
                if f < 0.0 {
                    Err(format!("(sqrt x) requires x > 0, but it is {:?}", args[0]))
                } else {
                    Ok(Primitive::from(f.sqrt()))
                }
            } else {
                Err(format!("(sqrt x) requires x numeric, but it is {:?}", args[0]))
            }
        }
        Builtin::Abs => {
            assert_num_args!("(abs x)", args, 1);
            if let Ok(i) = i128::try_from(&args[0]) {
                Ok(Primitive::from(i.abs()))
            } else if let Ok(f) = f64::try_from(&args[0]) {
                Ok(Primitive::from(f.abs()))
            } else {
                Err(format!("(abs x) requires x numeric, but it is {:?}", args[0]))
            }
        }
        Builtin::Ln => {
            assert_num_args!("(ln x)", args, 1);
            if let Ok(f) = f64::try_from(&args[0]) {
                if f < 0.0 {
                    Err(format!("(ln x) requires x > 0, but it is {:?}", args[0]))
                } else {
                    Ok(Primitive::from(f.ln()))
                }
            } else {
                Err(format!("(ln x) requires x numeric, but it is {:?}", args[0]))
            }
        }
        Builtin::IntRange => {
            assert_num_args!("(int-range a b)", args, 2);
            if let Some(pair) = integral_pair(&args[0], &args[1]) {
                Ok(Primitive::from(Domain::IntRange(pair.0, pair.1)))
            } else {
                Err(format!("(int-range a b) requires a, b integral"))
            }
        }
        Builtin::OneOf => {
            if args.len() == 0 {
                Err(String::from("(one-of x...) requires at least one argument."))
            } else {
                Ok(Primitive::from(Domain::OneOf(args.iter().cloned().collect())))
            }
        }
        _ => panic!("{:?} not implemented", builtin)
    }
}