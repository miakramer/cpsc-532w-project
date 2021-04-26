use crate::*;
use crate::primitives::*;
use ndarray::prelude::*;
use std::convert::TryFrom;


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
}


pub fn build_distribution(dtype: DistributionType, args: &[Primitive]) -> Result<Distribution, String> {
    match dtype {
        DistributionType::Dirac => {
            assert_num_args!("(dirac center)", args, 1);
            get_arg!(center, &args[0], number, "(dirac center) requries `center` numeric.");
            Ok(Distribution::Dirac{center})
        }
        DistributionType::Kronecker => {
            assert_num_args!("(kronecker center)", args, 1);
            get_arg!(center, &args[0], integral, "(kronecker center) requries `center` integral.");
            Ok(Distribution::Kronecker{center})
        }
        DistributionType::UniformContinuous => {
            assert_num_args!("(uniform-continuous a b)", args, 2);
            get_arg!(a, &args[0], number, "(uniform-continuous a b) requires `a` numeric.");
            get_arg!(b, &args[1], number, "(uniform-continuous a b) requires `b` numeric.");

            if b < a {
                return Err(String::from("(uniform-continuous a b) requires a <= b."));
            }

            Ok(Distribution::UniformContinuous{a, b})
        }
        DistributionType::UniformDiscrete => {
            assert_num_args!("(uniform-continuous a b)", args, 2);
            get_arg!(a, &args[0], integral, "(uniform-discrete a b) requires `a` numeric.");
            get_arg!(b, &args[1], integral, "(uniform-discrete a b) requires `b` numeric.");

            if b < a {
                return Err(String::from("(uniform-discrete a b) requires a <= b."));
            }

            Ok(Distribution::UniformDiscrete{a, b})
        }
        DistributionType::Categorical => {
            assert_num_args!("(categorical weights)", args, 1);
            get_arg!(weights, &args[0], vector, "(categorical weights) requires `weights` to be a vector of numbers.");
            let mut weights = weights;
            weights /= weights.iter().copied().sum::<f32>();
            if weights.iter().any(|x| *x < 0.0) {
                Err("(categorical weights): all weights must be positive".into())
            }
            else {
                Ok(Distribution::Categorical{weights})
            }            
        }
        DistributionType::Normal => {
            assert_num_args!("(normal mu sigma)", args, 2);
            get_arg!(mu, &args[0], number, "(normal mu sigma) requires `mu` numeric.");
            get_arg!(sigma, &args[1], number, "(normal mu sigma) requires `sigma` numeric.");
            if sigma <= 0. {
                Err("(normal mu sigma) requires `sigma` > 0".into())
            } else {
                Ok(Distribution::Normal{mu, sigma})
            }
        }
        DistributionType::Cauchy => {
            assert_num_args!("(cauchy median scale)", args, 2);
            get_arg!(median, &args[0], number, "(cauchy median scale) requires `median` numeric.");
            get_arg!(scale, &args[1], number, "(cauchy median scale) requires `scale` numeric.");
            if scale <= 0. {
                Err("(cauchy median scale) requires `scale` > 0".into())
            } else {
                Ok(Distribution::Cauchy{median, scale})
            }
        }
        DistributionType::Beta => {
            assert_num_args!("(beta α β)", args, 2);
            get_arg!(alpha, &args[0], number, "(beta α β) requires `α` numeric.");
            get_arg!(beta, &args[1], number, "(beta α β) requires `β` numeric.");
            if alpha <= 0. || beta <= 0. {
                Err("(beta α β) requires all parameters > 0".into())
            } else {
                Ok(Distribution::Beta{alpha, beta})
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
                Ok(Distribution::Dirichlet{weights})
            }
        }
        DistributionType::Exponential => {
            assert_num_args!("(exponential λ)", args, 1);
            get_arg!(lambda, &args[0], number, "(exponential λ) requires `λ` numeric");
            if lambda <= 0. {
                Err("(exponential λ) requires `λ` positive".into())
            } else {
                Ok(Distribution::Exponential{lambda})
            }
        }
        DistributionType::Gamma => {
            assert_num_args!("(gamma shape rate)", args, 2);
            get_arg!(shape, &args[0], number, "(gamma shape rate) requires `shape` numeric.");
            get_arg!(rate, &args[1], number, "(gamma shape rate) requires `rate` numeric.");
            if shape <= 0. || rate <= 0. {
                Err("(gamma shape rate) requires `shape` and `rate` positive.".into())
            } else {
                Ok(Distribution::Gamma{shape, rate})
            }
        }
        DistributionType::Bernoulli => {
            assert_num_args!("(bernoulli p)", args, 1);
            get_arg!(p, &args[0], number, "(bernoulli p) requires `p` numeric.");
            if p < 0.  ||  p > 1. {
                Err("(bernoulli p) requires p to be a probability".into())
            } else {
                Ok(Distribution::Bernoulli{p})
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
                Ok(Distribution::Binomial{n: n as u64, p})
            }
        }
    }
}
