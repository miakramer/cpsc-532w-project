pub use crate::parser::{ProclaimThreshold, Relation, C};
pub use crate::desugar::*;
// use common::DistributionType;
use smallvec::SmallVec;


#[derive(Clone, Copy, Debug)]
pub enum VarRef {
    Stochastic(u32),
    Decision(u32),
    Constant(C),
}


#[derive(Clone, Copy, Debug)]
pub enum PredicateExpr {
    And(u32, u32),
    Or(u32, u32),
    Not(u32),
    Eq(u32, u32),
    Ne(u32, u32),
    Lt(u32, u32),
    Gt(u32, u32),
    Le(u32, u32),
    Ge(u32, u32),

    Stochastic(u32),
    Decision(u32),
    Constant(C),
}


pub struct Predicate {
    pub exprs: SmallVec<[PredicateExpr; 8]>,
    pub not: bool, // invert expression
}

impl Predicate {
    pub fn not(&self) -> Self {
        let exprs = self.exprs.clone();
        Predicate { exprs, not: !self.not }
    }
}

pub struct Constraint {
    pub relation: Relation,
    pub left: VarRef,
    pub right: VarRef,
    pub when: Predicate,
}