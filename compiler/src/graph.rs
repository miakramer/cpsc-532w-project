pub use crate::parser::{ProclaimThreshold, Relation, C};
pub use crate::desugar::*;
// use common::DistributionType;
use smallvec::SmallVec;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariableKind {
    Stochastic,
    Decision,
}


#[derive(Clone, Copy, Debug)]
pub struct VarRef {
    kind: VariableKind,
    id: u32
}


#[derive(Clone, Debug, Hash)]
pub struct Variable {
    kind: VariableKind,
    name: Identifier,
    offset: u32,
}


impl Variable {
    pub fn format(&self) -> String {
        format!("{}${}", self.name, self.offset)
    }
}


#[derive(Clone, Debug)]
pub struct Variables {
    variables: Vec<Variable>
}

impl Variables {
    pub fn new() -> Self {
        Self { variables: Vec::new() }
    }

    pub fn push(&mut self, name: Identifier, kind: VariableKind) -> VarRef {
        let id = self.variables.len() as u32;
        let offset = self.offset_for_name(&name);
        self.variables.push(Variable{
            name,
            kind,
            offset,
        });
        VarRef {
            kind,
            id
        }
    }

    fn offset_for_name(&self, name: &Identifier) -> u32 {
        for var in self.variables.iter().rev() {
            if &var.name == name {
                return var.offset + 1;
            }
        }
        0
    }
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

    VarRef(VarRef),
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

#[derive(Clone, Debug)]
pub struct Constraint {
    pub relation: Relation,
    pub left: VarRef, // will be some sort of expression
    pub right: VarRef, // also here
    pub when: Predicate,
}

#[derive(Clone, Debug)]
pub struct Dependency {
    pub this: VarRef,
    pub depends_on: VarRef,
}

#[derive(Clone, Debug)]
pub struct ScpGraph {
    pub variables: Variables,
    pub dependencies: Vec<Dependency>,
    pub links: Vec<()>, // will be some sort of expression
    pub constraints: Vec<Constraint>,
}
