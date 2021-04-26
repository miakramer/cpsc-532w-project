use lazy_static::lazy_static;
use std::collections::HashMap;
pub mod eqmap;
pub mod primitives;
pub mod distribution;

lazy_static! {
    static ref BUILTINS: HashMap<&'static str, Builtin> = {
        let mut builtins = HashMap::new();

        builtins.insert("one-of", Builtin::OneOf);
        builtins.insert("int-range", Builtin::IntRange);

        builtins.insert("+", Builtin::Add);
        builtins.insert("-", Builtin::Sub);
        builtins.insert("*", Builtin::Mul);
        builtins.insert("/", Builtin::Div);
        builtins.insert("sqrt", Builtin::Sqrt);
        builtins.insert("first", Builtin::First);
        builtins.insert("peek", Builtin::First);
        builtins.insert("second", Builtin::Second);
        builtins.insert("last", Builtin::Last);
        builtins.insert("rest", Builtin::Rest);
        builtins.insert("append", Builtin::Append);
        builtins.insert("get", Builtin::Get);
        builtins.insert("put", Builtin::Put);
        builtins.insert("remove", Builtin::Remove);
        builtins.insert("conj", Builtin::Conj);
        builtins.insert("cons", Builtin::Cons);
        builtins.insert("empty?", Builtin::IsEmpty);
        builtins.insert("vector", Builtin::Vector);
        builtins.insert("hash-map", Builtin::HashMap);
        // builtins.insert("mat-add", Builtin::MatAdd);
        // builtins.insert("mat-mul", Builtin::MatMul);
        // builtins.insert("mat-transpose", Builtin::MatTranspose);
        // builtins.insert("mat-tanh", Builtin::MatTanh);
        // builtins.insert("mat-repmat", Builtin::MatRepmat);
        builtins.insert("<?", Builtin::IsLess);
        builtins.insert(">?", Builtin::IsGreater);
        builtins.insert("=?", Builtin::IsEqual);
        builtins.insert("and", Builtin::And);
        builtins.insert("or", Builtin::Or);
        builtins.insert("abs", Builtin::Abs);
        builtins.insert("log", Builtin::Ln);

        builtins
    };
    static ref DISTRIBUTIONS: HashMap<&'static str, DistributionType> = {
        let mut distributions = HashMap::new();

        distributions.insert("dirac", DistributionType::Dirac);
        distributions.insert("kronecker", DistributionType::Kronecker);
        distributions.insert("uniform-continuous", DistributionType::UniformContinuous);
        distributions.insert("uniform", DistributionType::UniformContinuous);
        distributions.insert("categorical", DistributionType::Categorical);
        distributions.insert("normal", DistributionType::Normal);
        distributions.insert("cauchy", DistributionType::Cauchy);
        distributions.insert("beta", DistributionType::Beta);
        distributions.insert("dirichlet", DistributionType::Dirichlet);
        distributions.insert("gamma", DistributionType::Gamma);
        distributions.insert("exponential", DistributionType::Exponential);
        distributions.insert("discrete", DistributionType::Categorical);
        distributions.insert("flip", DistributionType::Bernoulli);

        distributions
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Builtin {
    OneOf,
    IntRange,

    First,
    Second,
    Last,
    Rest,
    Get,
    Put,
    Append,
    Remove,
    Vector,
    HashMap,
    Cons,
    Conj,
    IsEmpty,

    Add,
    Sub,
    Mul,
    Div,
    Sqrt,
    Pow,
    Abs,
    Ln,

    IsLess,
    IsEqual,
    IsGreater,

    And,
    Or,
}

impl Builtin {
    pub fn maybe_match(name: &str) -> Option<Self> {
        BUILTINS.get(name).and_then(|x| Some(*x))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DistributionType {
    Dirac,
    Kronecker,
    UniformContinuous,
    UniformDiscrete,
    Categorical,
    // LogCategorical,

    Normal,
    // NormalStar,
    Cauchy,
    Beta,
    Dirichlet,
    Exponential,
    Gamma,
    // GammaStar,

    Binomial,
    Bernoulli,
}


impl DistributionType {
    pub fn maybe_match(name: &str) -> Option<DistributionType> {
        DISTRIBUTIONS.get(name).and_then(|x| Some(x.clone()))
    }
}
