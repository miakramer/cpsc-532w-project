
mod eref;
pub use eref::*;

use smallvec::SmallVec;
// use serde_json::Value;
use smol_str::SmolStr;
// use std::sync::Arc;

use crate::common::*;


const ETREE_DEFAULT_SIZE: usize = 16;
const CALLABLE_ARG_DEFAULT_SIZE: usize = 4;
const EXPRESSION_LIST_DEFAULT_SIZE: usize = 4;

pub type Args = SmallVec<[ExpressionRef; CALLABLE_ARG_DEFAULT_SIZE]>;
pub type ExpressionList = SmallVec<[ExpressionRef; EXPRESSION_LIST_DEFAULT_SIZE]>;
pub type ProcedureName = SmolStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Variable(VariableName),
    Constant(Primitive),
    Let {
        variable: VariableName,
        binding: ExpressionRef,
        inner: ExpressionRef,
    },
    If {
        predicate: ExpressionRef,
        consequent: ExpressionRef,
        alternative: ExpressionRef,
    },
    Procedure {
        procedure: ProcedureName,
        arguments: ExpressionList,
    },
    Builtin {
        builtin: Builtin,
        arguments: ExpressionList,
    },
    Distribution {
        distribution: DistributionType,
        arguments: ExpressionList,
    },
    // marker used for making sure we have storage-order traversal
    Placeholder,
}


#[derive(Clone, Debug, PartialEq)]
pub struct ExpressionTree {
    elements: SmallVec<[Expression; ETREE_DEFAULT_SIZE]>
}


impl ExpressionTree {
    pub fn n_elements(&self) -> usize {
        self.elements.len()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { elements: SmallVec::with_capacity(capacity) }
    }

    pub fn deref<'a>(&'a self, at: ExpressionRef) -> &'a Expression {
        if idx(at) > self.n_elements() {
            panic!("Attempted to access element @+{:X} in ExpressionTree@{:?}, but tree only has {} elements.", idx(at), self as *const Self, self.n_elements())
        }
        &self.elements[idx(at)]
    }

    pub fn clone_at(&self, at: ExpressionRef) -> ExpressionTree {
        let mut new_tree = Self::with_capacity(self.n_elements() - idx(at));

        self._clone_at(&mut new_tree, at);
        
        new_tree
    }

    fn _clone_at(&self, new: &mut ExpressionTree, at: ExpressionRef) -> ExpressionRef {
        let placeholder = new.placeholder();

        let ex = match self.deref(at) {
            Expression::Variable(n) => {
                Expression::Variable(n.clone())
            }
            Expression::Constant(c) => {
                Expression::Constant(c.clone())
            }
            Expression::If{predicate, consequent, alternative} => {
                let pred = self._clone_at(new, *predicate);
                let cons = self._clone_at(new, *consequent);
                let alt  = self._clone_at(new, *alternative);

                Expression::If{
                    predicate: pred,
                    consequent: cons,
                    alternative: alt,
                }
            }
            Expression::Placeholder => {
                Expression::Placeholder {}
            },
            _ => unimplemented!()
        };

        new.replace(placeholder, ex)
    }

    fn placeholder(&mut self) -> ExpressionRef {
        let idx = self.elements.len();
        self.elements.push(Expression::Placeholder);
        ExpressionRef::from(idx)
    }

    fn add(&mut self, e: Expression) -> ExpressionRef {
        let idx = self.elements.len();
        self.elements.push(e);
        ExpressionRef::from(idx)
    }

    fn replace(&mut self, at: ExpressionRef, with: Expression) -> ExpressionRef {
        self.elements[idx(at)] = with;
        at
    }
}
