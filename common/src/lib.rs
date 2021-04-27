use lazy_static::lazy_static;
use primitives::Primitive;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::collections::HashMap;
pub mod eqmap;
pub mod primitives;
pub mod distribution;
use serde::{Serialize, Deserialize};


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Relation {
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
}

impl Relation {
    pub fn pretty_print(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "≠",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Leq => "≤",
            Self::Geq => "≥"
        }
    }
}


pub type Identifier = SmolStr;


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ExpressionRef {
    pub index: u32,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EvalExpr {
    C(Primitive),
    Begin(Vec<ExpressionRef>),
    Decision{id: Identifier, body: ExpressionRef},
    Stochastic{id: Identifier, body: ExpressionRef},
    If{
        predicate: ExpressionRef,
        consequent: ExpressionRef,
        alternative: ExpressionRef,
    },
    Constrain{
        prob: f64,
        relation: Relation,
        left: ExpressionRef,
        right: ExpressionRef,
    },
    Builtin{
        builtin: Builtin,
        args: Vec<ExpressionRef>,
    },
    Distribution{
        distribution: DistributionType,
        args: Vec<ExpressionRef>,
    },
    Placeholder,
    Deleted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvaluatedTree {
    pub expressions: SmallVec<[EvalExpr; 8]>,
}

impl EvaluatedTree {
    pub fn new() -> Self {
        Self {
            expressions: SmallVec::new(),
        }
    }

    pub fn root(&self) -> ExpressionRef {
        ExpressionRef { index: 0 }
    }

    pub fn push(&mut self, expr: EvalExpr) -> ExpressionRef {
        let l = self.expressions.len() as u32;
        self.expressions.push(expr);
        ExpressionRef { index: l }
    }

    pub fn placeholder(&mut self) -> ExpressionRef {
        let ret = self.push(EvalExpr::Placeholder);
        // println!(" -> Creating placeholder @ {}", ret.index);
        ret
    }

    pub fn replace(&mut self, at: ExpressionRef, with: EvalExpr) -> ExpressionRef {
        match self.expressions[at.index as usize] {
            EvalExpr::Placeholder => (),
            EvalExpr::Deleted => (),
            _ => panic!("Replacing non-placeholder value: {:?}\n -> with {:?}", &self.expressions[at.index as usize], &with)
        };
        // println!(" -> Replacing placeholder @ {}", at.index);
        self.expressions[at.index as usize] = with;
        at
    }

    pub fn delete(&mut self, at: ExpressionRef) {
        self.expressions[at.index as usize] = EvalExpr::Deleted;
    }

    pub fn deref<'a>(&'a self, at: ExpressionRef) -> &EvalExpr {
        &self.expressions[at.index as usize]
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VariableKind {
    Stochastic,
    Decision,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VarRef {
    pub kind: VariableKind,
    pub id: u32
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variable {
    pub kind: VariableKind,
    pub name: Identifier,
    pub definition: EvaluatedTree,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variables {
    pub variables: Vec<Variable>,
}

impl Variables {
    pub fn deref<'a>(&'a self, at: &VarRef) -> &'a Variable {
        &self.variables[at.id as usize]
    }

    pub fn new() -> Self {
        Self { variables: Vec::new() }
    }

    pub fn has_name(&self, ident: &Identifier) -> bool {
        for var in &self.variables {
            if &var.name == ident {
                return true;
            }
        }
        false
    }

    pub fn get_by_name(&self, ident: &Identifier) -> Option<VarRef> {
        for (i, var) in self.variables.iter().enumerate() {
            if &var.name == ident {
                return Some(VarRef{id: i as u32, kind: var.kind});
            }
        }
        None
    }

    pub fn push(&mut self, kind: VariableKind, name: Identifier, definition: EvaluatedTree) -> VarRef {
        let id = self.variables.len() as u32;
        self.variables.push(Variable{kind, name, definition});
        VarRef{kind, id}
    }

    pub fn get_ref(&self, of: &Variable) -> VarRef {
        for (i, var) in self.variables.iter().enumerate() {
            if var.name == of.name {
                return VarRef{id: i as u32, kind: var.kind};
            }
        }
        unreachable!()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Predicate {
    pub pred: EvaluatedTree,
    pub negated: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraint {
    pub probability: f64,
    pub relation: Relation,
    pub left: EvaluatedTree,
    pub right: EvaluatedTree,
    pub predicate: SmallVec<[Predicate; 8]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub this: VarRef,
    pub depends_on: SmallVec<[VarRef; 8]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScpGraph {
    pub variables: Variables,
    pub dependencies: Vec<Dependency>,
    pub constraints: Vec<Constraint>,
    pub body: EvaluatedTree,
}

impl ScpGraph {
    pub fn dependencies_of<'a>(&'a self, var: VarRef) -> &'a Dependency {
        for d in &self.dependencies {
            if d.this == var {
                return d;
            }
        }
        unreachable!()
    }

    pub fn get_ref(&self, of: &Variable) -> VarRef {
        self.variables.get_ref(of)
    }
}



lazy_static! {
    static ref BUILTINS: HashMap<&'static str, Builtin> = {
        let mut builtins = HashMap::new();

        builtins.insert("one-of", Builtin::OneOf);
        builtins.insert("int-range", Builtin::IntRange);

        builtins.insert("+", Builtin::Add);
        builtins.insert("-", Builtin::Sub);
        builtins.insert("*", Builtin::Mul);
        builtins.insert("/", Builtin::Div);
        builtins.insert("pow", Builtin::Pow);
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
        // distributions.insert("uniform-continuous", DistributionType::UniformContinuous);
        distributions.insert("uniform-discrete", DistributionType::UniformDiscrete);
        // distributions.insert("uniform", DistributionType::UniformContinuous);
        distributions.insert("categorical", DistributionType::Categorical);
        distributions.insert("map-categorical", DistributionType::MappedCategorical);
        // distributions.insert("normal", DistributionType::Normal);
        // distributions.insert("cauchy", DistributionType::Cauchy);
        // distributions.insert("beta", DistributionType::Beta);
        // distributions.insert("dirichlet", DistributionType::Dirichlet);
        // distributions.insert("gamma", DistributionType::Gamma);
        // distributions.insert("exponential", DistributionType::Exponential);
        // distributions.insert("discrete", DistributionType::Categorical);
        distributions.insert("flip", DistributionType::Bernoulli);

        distributions
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DistributionType {
    Dirac,
    Kronecker,
    UniformContinuous,
    UniformDiscrete,
    Categorical,
    MappedCategorical,
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
