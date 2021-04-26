use smallvec::SmallVec;

use super::{Expr, VarRef};
use super::ExpressionRef;
use crate::parser::C;

type ERef = ExpressionRef;

#[derive(Clone, Copy, Debug)]
pub enum PredicateExpr {
    And(ERef, ERef),
    Or(ERef, ERef),
    Not(ERef),
    Eq(ERef, ERef),
    Ne(ERef, ERef),
    Lt(ERef, ERef),
    Gt(ERef, ERef),
    Le(ERef, ERef),
    Ge(ERef, ERef),

    Variable(VarRef),
    Constant(C),
}

#[derive(Clone, Debug)]
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


pub type LinkExpr = Expr<VarRef>;
