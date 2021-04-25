use common::DistributionType;
use smol_str::SmolStr;

use crate::parser;
pub use crate::parser::{ProclaimThreshold, Relation, C};

pub type Identifier = SmolStr;

#[derive(Clone, Copy, Debug)]
pub struct ExpressionRef {
    index: u32,
}

const SMALL_ARG_SIZE: usize = 4;

#[derive(Clone, Debug)]
pub enum Expr {
    C(C),
    V(Identifier),
    Begin(Vec<ExpressionRef>),
    If {
        predicate: ExpressionRef,
        consequent: ExpressionRef,
        alternative: ExpressionRef,
    },
    Let {
        name: Identifier,
        value: ExpressionRef,
        body: ExpressionRef,
    },
    Sample(ExpressionRef),
    Observe {
        observable: ExpressionRef,
        observed: ExpressionRef,
    },
    Decision(ExpressionRef),
    Constrain {
        relation: Relation,
        left: ExpressionRef,
        right: ExpressionRef,
    },
    Builtin {
        builtin: common::Builtin,
        args: Vec<ExpressionRef>,
    },
    Distribution {
        distribution: DistributionType,
        args: Vec<ExpressionRef>,
    },
    Procedure {
        name: Identifier,
        args: Vec<ExpressionRef>,
    },
}

#[derive(Clone, Debug)]
pub struct ExpressionTree {
    expressions: Vec<Expr>,
}

impl ExpressionTree {
    pub fn new() -> Self {
        Self {
            expressions: Vec::new(),
        }
    }

    pub fn push(&mut self, expr: Expr) -> ExpressionRef {
        let l = self.expressions.len() as u32;
        self.expressions.push(expr);
        ExpressionRef { index: l }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    proclaim: ProclaimThreshold,
    defns: Vec<Defn>,
    body: ExpressionTree,
}

#[derive(Clone, Debug)]
pub struct Defn {
    name: Identifier,
    args: Vec<Identifier>,
    body: ExpressionTree,
}

#[derive(Debug)]
pub enum DesugarError {
    UndefinedProcedure(SmolStr),
}

pub fn desugar(raw: &parser::Program) -> Result<Program, DesugarError> {
    let mut prog = Program {
        proclaim: raw.proclaim,
        defns: Vec::with_capacity(raw.defns.len()),
        body: ExpressionTree::new(),
    };

    let mut defn_names = Vec::new();
    let mut name_state = 0u32;

    for defn in &raw.defns {
        let mut expr = ExpressionTree::new();
        desugar_exprs(&mut expr, &defn.body, &defn_names, &mut name_state)?;

        let name = ident(&defn.name, &mut name_state);

        prog.defns.push(Defn {
            name: name.clone(),
            args: defn
                .args
                .iter()
                .map(|i| ident(i, &mut name_state))
                .collect(),
            body: expr,
        });

        defn_names.push(name);
    }

    desugar_exprs(&mut prog.body, &raw.body, &defn_names, &mut name_state)?;

    Ok(prog)
}

fn ident(source: &parser::Identifier, state: &mut u32) -> Identifier {
    use std::io::Write;

    match source {
        parser::Identifier::Newvar => {
            let mut buf = [0u8; 22];
            {
                let mut br = &mut buf[..];
                write!(br, "${}", state).unwrap();
            }
            let s = std::str::from_utf8(&buf[..]).unwrap();
        
            Identifier::from(s)
        },
        parser::Identifier::Ident(i) => i.clone()
    }
}

fn desugar_exprs(tree: &mut ExpressionTree, source: &parser::Expr, names: &Vec<Identifier>, name_state: &mut u32) -> Result<ExpressionRef, DesugarError> {
    match source {
        parser::Expr::C(c) => Ok(tree.push(Expr::C(*c))),
        parser::Expr::V(i) => Ok(tree.push(Expr::V(ident(i, name_state)))),
        parser::Expr::Begin(exprs) => {
            let mut refs = Vec::with_capacity(exprs.len());
            for expr in exprs {
                refs.push(desugar_exprs(tree, expr, names, name_state)?);
            }
            Ok(tree.push(Expr::Begin(refs)))
        },
        parser::Expr::If{predicate, consequent, alternative} => {
            let predicate = desugar_exprs(tree, predicate, names, name_state)?;
            let consequent = desugar_exprs(tree, consequent, names, name_state)?;
            let alternative = desugar_exprs(tree, alternative, names, name_state)?;
            Ok(tree.push(Expr::If{predicate, consequent, alternative}))
        },
        parser::Expr::Let{bindings, body} => {
            desugar_exprs_let(tree, names, name_state, &bindings[..], body)
        },
        parser::Expr::Sample(e) => {
            let e = desugar_exprs(tree, e, names, name_state)?;
            Ok(tree.push(Expr::Sample(e)))
        },
        parser::Expr::Observe{observable, observed} => {
            let observable = desugar_exprs(tree, observable, names, name_state)?;
            let observed = desugar_exprs(tree, observed, names, name_state)?;
            Ok(tree.push(Expr::Observe{observable, observed}))
        }
        parser::Expr::Decision(d) => {
            let d = desugar_exprs(tree, d, names, name_state)?;
            Ok(tree.push(Expr::Decision(d)))
        }
        parser::Expr::Constrain{relation, left, right} => {
            let left = desugar_exprs(tree, left, names, name_state)?;
            let right = desugar_exprs(tree, right, names, name_state)?;
            Ok(tree.push(Expr::Constrain{relation: *relation, left, right}))
        }
        parser::Expr::Apply{head, body} => {
            let name = ident(head, name_state);
            let mut args = Vec::with_capacity(body.len());
            for e in body {
                args.push(desugar_exprs(tree, e, names, name_state)?);
            }
            if let Some(builtin) = common::Builtin::maybe_match(name.as_str()) {
                Ok(tree.push(Expr::Builtin{builtin, args}))
            } else if let Some(distribution) = DistributionType::maybe_match(name.as_str()) {
                Ok(tree.push(Expr::Distribution{distribution, args}))
            } else if names.contains(&name) {
                Ok(tree.push(Expr::Procedure{name, args}))
            } else {
                Err(DesugarError::UndefinedProcedure(name))
            }
        }
    }
}

fn desugar_exprs_let(tree: &mut ExpressionTree, names: &Vec<Identifier>, name_state: &mut u32, bindings: &[(parser::Identifier, parser::Expr)], body: &parser::Expr) -> Result<ExpressionRef, DesugarError> {
    if bindings.len() == 0 {
        panic!("Shouldn't happen")
    } else if bindings.len() == 1 {
        let (i, e) = &bindings[0];
        let value = desugar_exprs(tree, e, names, name_state)?;
        let name = ident(i, name_state);
        let body = desugar_exprs(tree, body, names, name_state)?;
        Ok(tree.push(Expr::Let{name, value, body}))
    } else {
        let (i, e) = &bindings[0];
        let name = ident(i, name_state);
        let value = desugar_exprs(tree, e, names, name_state)?;
        let body = desugar_exprs_let(tree, names, name_state, &bindings[1..], body)?;
        Ok(tree.push(Expr::Let{name, value, body}))
    }
}
