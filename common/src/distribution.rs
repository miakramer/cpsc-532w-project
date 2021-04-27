use crate::*;
use crate::primitives::*;
use ndarray::prelude::*;
use rand::prelude::ThreadRng;
use std::convert::TryFrom;
use crate::primitives;


macro_rules! assert_num_args {
    ($name:expr, $args:expr, $len:expr) => {
        if $args.len() != $len {
            panic!("{} requires {} arguments but got {}", $name, $len, $args.len());
        }
    };
}


macro_rules! get_arg {
    ($argname:ident, $arg:expr, number, $message:expr) => {
        let $argname = match f64::try_from_ref($arg) {
            Some(f) => *f,
            None => {return Err(String::from($message))},
        };
    };
    ($argname:ident, $arg:expr, integral, $message:expr) => {
        let $argname = match i128::try_from_ref($arg) {
            Some(f) => *f,
            None => {return Err(String::from($message))},
        };
    };
    ($argname:ident, $arg:expr, vector, $message:expr) => {
        let $argname = match Array1::<f32>::try_from($arg) {
            Ok(f) => f,
            Err(_) => {return Err(String::from($message))},
        };
    };
    ($argname:ident, $arg:expr, list, $message:expr) => {
        let $argname: Vec<Primitive> = match &$arg {
            Primitive::Vector(v) => v.iter().cloned().collect(),
            Primitive::EvaluatedVector(v) => v.iter().map(|f| Primitive::from(*f)).collect(),
            _ => {return Err(String::from($message))},
        };
    };
}


use rand::prelude::*;
use rand_distr::*;
use rand_distr::Distribution;


impl Sample for primitives::Distribution {
    type Output = Primitive;

    fn sample(&self, rng: &mut ThreadRng) -> Primitive {
        match self {
            Self::Dirac{center} => Primitive::from(*center),
            Self::Kronecker{center} => Primitive::from(*center),
            Self::UniformDiscrete{a, b} => {
                Primitive::from(Uniform::from(*a..*b).sample(rng))
            }
            Self::Categorical{weights} => {
                let l = weights.len();

                if cfg!(debug_assertions) {
                    print!("Sampling (categorical {:?})…", weights.as_slice());
                }

                let val = rng.gen::<f32>();
                let mut ssf = 0.0;
                for (i, x) in weights.iter().enumerate() {
                    ssf += x;
                    if val <= ssf {
                        if cfg!(debug_assertions) { println!("  -> result: {}", i); }
                        return Primitive::from(i);
                    }
                }
                Primitive::from(l - 1) // rounding error
            }
            Self::MappedCategorical{weights, values} => {
                let l = weights.len();

                if cfg!(debug_assertions) {
                    print!("Sampling (categorical {:?})…", weights.as_slice());
                }

                let val = rng.gen::<f32>();
                let mut ssf = 0.0;
                let mut res = -1;
                for (i, x) in weights.iter().enumerate() {
                    ssf += x;
                    if val <= ssf {
                        if cfg!(debug_assertions) { println!("  -> result: {}", i); }
                        res = i as i128;
                    }
                }
                if res == -1 {
                    res = (l - 1) as i128;
                }
                values[res as usize].clone()
            }
            Self::Bernoulli{p} => {
                Primitive::from(rng.gen::<f64>() <= *p)
            }
            _ => todo!()
        }
    }
}


pub fn build_distribution(dtype: DistributionType, args: &[Primitive]) -> Result<primitives::Distribution, String> {
    match dtype {
        DistributionType::Dirac => {
            assert_num_args!("(dirac center)", args, 1);
            get_arg!(center, &args[0], number, "(dirac center) requries `center` numeric.");
            Ok(primitives::Distribution::Dirac{center})
        }
        DistributionType::Kronecker => {
            assert_num_args!("(kronecker center)", args, 1);
            get_arg!(center, &args[0], integral, "(kronecker center) requries `center` integral.");
            Ok(primitives::Distribution::Kronecker{center})
        }
        DistributionType::UniformContinuous => {
            assert_num_args!("(uniform-continuous a b)", args, 2);
            get_arg!(a, &args[0], number, "(uniform-continuous a b) requires `a` numeric.");
            get_arg!(b, &args[1], number, "(uniform-continuous a b) requires `b` numeric.");

            if b < a {
                return Err(String::from("(uniform-continuous a b) requires a <= b."));
            }

            Ok(primitives::Distribution::UniformContinuous{a, b})
        }
        DistributionType::UniformDiscrete => {
            assert_num_args!("(uniform-continuous a b)", args, 2);
            get_arg!(a, &args[0], integral, "(uniform-discrete a b) requires `a` numeric.");
            get_arg!(b, &args[1], integral, "(uniform-discrete a b) requires `b` numeric.");

            if b < a {
                return Err(String::from("(uniform-discrete a b) requires a <= b."));
            }

            Ok(primitives::Distribution::UniformDiscrete{a, b})
        }
        DistributionType::Categorical => {
            assert_num_args!("(categorical weights)", args, 1);
            get_arg!(weights, &args[0], vector, "(categorical weights) requires `weights` to be a vector of numbers.");
            let mut weights = weights;
            weights /= weights.iter().copied().sum::<f32>();
            if weights.iter().any(|x| *x < 0.0) {
                Err("(categorical weights): all weights must be positive".into())
            } else {
                Ok(primitives::Distribution::Categorical{weights})
            }
        }
        DistributionType::MappedCategorical => {
            assert_num_args!("(map-categorical weights values)", args, 2);
            get_arg!(weights, &args[0], vector, "(map-categorical weights values) requires `weights` to be a vector of numbers.");
            get_arg!(values, &args[1], list, "(map-categorical weights values) requires `values` to be a vector.");
            let mut weights = weights;
            weights /= weights.iter().copied().sum::<f32>();
            if weights.iter().any(|x| *x < 0.0) {
                Err("(map-categorical weights values): all weights must be positive".into())
            } else if weights.len() != values.len() {
                Err("(map-categorical weights values): values and weights must have the same length".into())
            } else {
                Ok(primitives::Distribution::MappedCategorical{weights, values})
            }
        }
        DistributionType::Normal => {
            assert_num_args!("(normal mu sigma)", args, 2);
            get_arg!(mu, &args[0], number, "(normal mu sigma) requires `mu` numeric.");
            get_arg!(sigma, &args[1], number, "(normal mu sigma) requires `sigma` numeric.");
            if sigma <= 0. {
                Err("(normal mu sigma) requires `sigma` > 0".into())
            } else {
                Ok(primitives::Distribution::Normal{mu, sigma})
            }
        }
        DistributionType::Cauchy => {
            assert_num_args!("(cauchy median scale)", args, 2);
            get_arg!(median, &args[0], number, "(cauchy median scale) requires `median` numeric.");
            get_arg!(scale, &args[1], number, "(cauchy median scale) requires `scale` numeric.");
            if scale <= 0. {
                Err("(cauchy median scale) requires `scale` > 0".into())
            } else {
                Ok(primitives::Distribution::Cauchy{median, scale})
            }
        }
        DistributionType::Beta => {
            assert_num_args!("(beta α β)", args, 2);
            get_arg!(alpha, &args[0], number, "(beta α β) requires `α` numeric.");
            get_arg!(beta, &args[1], number, "(beta α β) requires `β` numeric.");
            if alpha <= 0. || beta <= 0. {
                Err("(beta α β) requires all parameters > 0".into())
            } else {
                Ok(primitives::Distribution::Beta{alpha, beta})
            }
        }
        DistributionType::Dirichlet => {
            assert_num_args!("(dirichlet weights)", args, 1);
            get_arg!(weights, &args[0], vector, "(dirichlet weights) requires `weights` numeric.");
            if weights.iter().any(|x| *x > 1. || *x < 0.) {
                Err("(dirichlet weights) requires all weights > 0 and < 1".into())
            } else if (1.0 - weights.iter().sum::<f32>()).abs() > 1e-4 {
                Err("(dirichlet weights) requires weights to sum to 1".into())
            } else {
                let weights = Array1::<f32>::from_iter(weights.iter().copied());
                Ok(primitives::Distribution::Dirichlet{weights})
            }
        }
        DistributionType::Exponential => {
            assert_num_args!("(exponential λ)", args, 1);
            get_arg!(lambda, &args[0], number, "(exponential λ) requires `λ` numeric");
            if lambda <= 0. {
                Err("(exponential λ) requires `λ` positive".into())
            } else {
                Ok(primitives::Distribution::Exponential{lambda})
            }
        }
        DistributionType::Gamma => {
            assert_num_args!("(gamma shape rate)", args, 2);
            get_arg!(shape, &args[0], number, "(gamma shape rate) requires `shape` numeric.");
            get_arg!(rate, &args[1], number, "(gamma shape rate) requires `rate` numeric.");
            if shape <= 0. || rate <= 0. {
                Err("(gamma shape rate) requires `shape` and `rate` positive.".into())
            } else {
                Ok(primitives::Distribution::Gamma{shape, rate})
            }
        }
        DistributionType::Bernoulli => {
            assert_num_args!("(bernoulli p)", args, 1);
            get_arg!(p, &args[0], number, "(bernoulli p) requires `p` numeric.");
            if p < 0.  ||  p > 1. {
                Err("(bernoulli p) requires p to be a probability".into())
            } else {
                Ok(primitives::Distribution::Bernoulli{p})
            }
        }
        DistributionType::Binomial => {
            assert_num_args!("(binomial n p)", args, 2);
            get_arg!(n, &args[0], integral, "(binomial n p) requires `n` integral.");
            get_arg!(p, &args[1], number, "(binomial n p) requires `p` numeric.");
            if p < 0.  ||  p > 1. {
                Err("(binomial n p) requires p to be a probability.".into())
            } else if n < 0 {
                Err("(binomial n p) requires n nonnegative.".into())
            } else {
                Ok(primitives::Distribution::Binomial{n: n as u64, p})
            }
        }
    }
}




// pub fn sample_distribution(
//     distribution: &crate::primitives::Distribution, rng: &mut ThreadRng,
// ) -> Result<Primitive, String> {
//     let args = &distribution.arguments[..];
//     match distribution.distribution {
//         DistributionType::Dirac => {
//             check_params!((dirac center) from args);
//             Ok(Primitive::from(center))
//         }
//         DistributionType::Kronecker => {
//             check_params!((kronecker center) from args);
//             Ok(Primitive::from(center))
//         }
//         DistributionType::UniformContinuous => {
//             check_params!((uniform-continuous a b) from args);
//             Ok(Primitive::from(Uniform::from(a..b).sample(rng)))
//         }
//         DistributionType::UniformDiscrete => {
//             check_params!((uniform-discrete a b) from args);
//             Ok(Primitive::from(Uniform::from(a..b).sample(rng)))
//         }
//         DistributionType::Categorical => {
//             check_params!((categorical weights) from args);
//             let l = weights.len();

//             if cfg!(debug_assertions) {
//                 print!("Sampling (categorical {:?})…", weights.as_slice());
//             }

//             let val = rng.gen::<f64>();
//             let mut ssf = 0.0;
//             for (i, x) in weights.iter().enumerate() {
//                 ssf += x;
//                 if val <= ssf {
//                     if cfg!(debug_assertions) { println!("  -> result: {}", i); }
//                     return Ok(Primitive::from(i));
//                 }
//             }
//             if cfg!(debug_assertions) { println!("  -> result: {}", l - 1); }
//             Ok(Primitive::from(l - 1)) // rounding error
//         }
//         // DistributionType::LogCategorical => {
//         //     check_params!((categorical-logit logits) from args);
//         //     let mut logits = logits;
//         //     // normalize
//         //     let norm = logsumexp(&logits);
//         //     logits -= norm;
//         //     // convert to probabilities
//         //     logits.mapv_inplace(|f| f.exp());
//         //     let norm = logits.sum();
//         //     logits /= norm;
//         //     let weights = logits;
//         //     // sample
//         //     let l = weights.len();
//         //     let val = rng.gen::<f32>();
//         //     let mut ssf: f32 = 0.0;
//         //     for (i, x) in weights.iter().enumerate() {
//         //         ssf += x;
//         //         if val <= ssf {
//         //             return Primitive::from(i);
//         //         }
//         //     }
//         //     return Primitive::from(l - 1); // rounding error
//         // }
//         DistributionType::Normal => {
//             check_params!((normal mu sigma) from args);
//             Ok(Primitive::from(Normal::new(mu, sigma).unwrap().sample(rng)))
//         }
//         // DistributionType::NormalStar => {
//         //     check_params!((normal mu sigma) from args);
//         //     let sigma = (sigma.exp() + 1.).ln();
//         //     Primitive::from(Normal::new(mu, sigma).unwrap().sample(rng))
//         // }
//         DistributionType::Cauchy => {
//             check_params!((cauchy median scale) from args);
//             Ok(Primitive::from(Cauchy::new(median, scale).unwrap().sample(rng)))
//         }
//         DistributionType::Beta => {
//             check_params!((beta alpha beta) from args);
//             Ok(Primitive::from(Beta::new(alpha, beta).unwrap().sample(rng)))
//         }
//         DistributionType::Dirichlet => {
//             check_params!((dirichlet weights) from args);
//             Ok(Primitive::from(
//                 Dirichlet::new(weights.as_slice().unwrap())
//                     .unwrap()
//                     .sample(rng)))
//         }
//         DistributionType::Exponential => {
//             check_params!((exponential lambda) from args);
//             Ok(Primitive::from(Exp::new(lambda).unwrap().sample(rng)))
//         }
//         DistributionType::Gamma => {
//             check_params!((gamma shape rate) from args);
//             Ok(Primitive::from(Gamma::new(shape, 1.0/rate).unwrap().sample(rng)))
//         }
//         // DistributionType::GammaStar => {
//         //     check_params!((gamma shape rate) from args);
//         //     let rate = (rate.exp() + 1.0).ln();
//         //     Primitive::from(Gamma::new(shape, 1.0/rate).unwrap().sample(rng))
//         // }
//         DistributionType::Bernoulli => {
//             check_params!((bernoulli p) from args);
//             Ok(Primitive::from(rng.gen::<f64>() <= p))
//         }
//         DistributionType::Binomial => {
//             check_params!((binomial n p) from args);
//             Ok(Primitive::from(Binomial::new(n as u64, p).unwrap().sample(rng) as i128))
//         }
//     }
// }