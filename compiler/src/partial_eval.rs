

pub use crate::desugar::*;
use common::{*, primitives::*, distribution::build_distribution};

mod eval;
use eval::*;



fn fresh(state: &mut u32, varkind: VariableKind) -> Identifier {
    use std::io::Write;

    let mut buf = [0u8; 22];
    {
        let mut br = &mut buf[..];
        write!(br, "@{}{}", match varkind {
            VariableKind::Stochastic => 'S',
            VariableKind::Decision => 'D',
        }, state).unwrap();
    }
    let mut i = 0;
    while buf[i] != 0 {
        i += 1;
    }
    let s = std::str::from_utf8(&buf[..i]).unwrap();

    *state += 1;

    Identifier::from(s)
}

fn append_fresh(state: &mut u32, base: &Identifier) -> Identifier {
    use std::fmt::Write;

    let mut new = String::with_capacity(5 + base.len());
    new.push_str(base.as_str());
    write!(&mut new, "@{}", state).unwrap();

    new.into()
}

pub fn partial_eval(src: &ExpressionTree<Identifier>) -> Result<EvaluatedTree, PartialEvalErr> {
    let mut name_state = 0;
    let mut out = EvaluatedTree::new();
    _partial_eval(src, src.root(), &mut out, &im::HashMap::new(), &mut name_state, None)?;
    Ok(out)
}

pub enum PartialEvalErr {
    Bubble(String),
    Undefined(Identifier),
    Placeholder,
    Observe,
    InvalidProbability,
}

fn bind(bindings: &im::HashMap<Identifier, ExpressionRef>, this: Identifier, to: ExpressionRef) -> im::HashMap<Identifier, ExpressionRef> {
    let mut bindings = bindings.clone();
    bindings.insert(this, to);
    bindings
}

fn _partial_eval(src: &ExpressionTree<Identifier>, at: ExpressionRef, to: &mut EvaluatedTree, bindings: &im::HashMap<Identifier, ExpressionRef>, name_state: &mut u32, recycle: Option<ExpressionRef>) -> Result<ExpressionRef, PartialEvalErr> {
    let placeholder = if let Some(p) = recycle {
        p
    } else {
        to.placeholder()
    };
    match src.deref(at) {
        Expr::C(c) => {Ok(to.replace(placeholder, EvalExpr::C(c.clone().into())))},
        Expr::V(v) => {
            if let Some(b) = bindings.get(v) {
                Ok(_clone_at(to, *b, placeholder))
            } else {
                Err(PartialEvalErr::Undefined(v.clone()))
            }
        }
        Expr::Begin(b) => {
            let mut new = Vec::with_capacity(b.len());
            for expr in b {
                new.push(_partial_eval(src, *expr, to, bindings, name_state, None)?);
            }
            if new.iter().all(|er| {
                if let EE::C(p) = to.deref(*er) {
                    is_const(p)
                } else {
                    false
                }
            }) {
                let expr = to.deref(*new.last().unwrap()).clone();
                for expr in new {
                    to.replace(expr, EE::Deleted);
                }
                if let EE::C(p) = expr {
                    Ok(to.replace(placeholder, EE::C(p)))
                } else {
                    unreachable!()
                }
            } else {
                Ok(to.replace(placeholder, EE::Begin(new)))
            }
        }
        Expr::If{predicate, consequent, alternative} => {
            let predicate = _partial_eval(src, *predicate, to, bindings, name_state, None)?;
            let consequent = _partial_eval(src, *consequent, to, bindings, name_state, None)?;
            let alternative = _partial_eval(src, *alternative, to, bindings, name_state, None)?;
            Ok(to.replace(placeholder, EE::If{predicate, consequent, alternative}))
        }
        Expr::Let{name, value, body} => {
            let name = name.clone();
            let value = _partial_eval(src, *value, to, bindings, name_state, None)?;
            _partial_eval(src, *body, to, &bind(bindings, name, value), name_state, Some(placeholder))
        }
        Expr::Sample(e) => {
            let id = fresh(name_state, VariableKind::Stochastic);
            let body = _partial_eval(src, *e, to, bindings, name_state, None)?;
            Ok(to.replace(placeholder, EE::Stochastic{id, body}))
        }
        Expr::Decision(e) => {
            let id = fresh(name_state, VariableKind::Decision);
            let body = _partial_eval(src, *e, to, bindings, name_state, None)?;
            Ok(to.replace(placeholder, EE::Decision{id, body}))
        }
        Expr::Builtin{builtin, args} => {
            let builtin = *builtin;
            let mut new = Vec::with_capacity(args.len());
            for expr in args {
                new.push(_partial_eval(src, *expr, to, bindings, name_state, None)?);
            }
            if new.iter().all(|e| {
                if let EE::C(p) = to.deref(*e) {
                    is_const(p)
                } else {
                    false
                }
            }) {
                let args: Vec<Primitive> = new.iter().map(|e| match to.deref(*e) {
                    EE::C(p) => p.clone(),
                    _ => unreachable!()
                }).collect();
                let result = match eval_builtin(builtin, args.as_slice()) {
                    Ok(o) => o,
                    Err(e) => {return Err(PartialEvalErr::Bubble(e))}
                };
                for expr in new {
                    to.delete(expr);
                }
                Ok(to.replace(placeholder, EE::C(result)))
            } else {
                Ok(to.replace(placeholder, EE::Builtin{builtin, args: new}))
            }
        }
        Expr::Distribution{distribution, args} => {
            let distribution = *distribution;
            let mut new = Vec::with_capacity(args.len());
            for expr in args {
                new.push(_partial_eval(src, *expr, to, bindings, name_state, None)?);
            }
            if new.iter().all(|e| {
                if let EE::C(p) = to.deref(*e) {
                    is_const(p)
                } else {
                    false
                }
            }) {
                let args: Vec<Primitive> = new.iter().map(|e| match to.deref(*e) {
                    EE::C(p) => p.clone(),
                    _ => unreachable!()
                }).collect();
                let result = match build_distribution(distribution, args.as_slice()) {
                    Ok(o) => o,
                    Err(e) => {return Err(PartialEvalErr::Bubble(e))}
                };
                for expr in new {
                    to.delete(expr);
                }
                Ok(to.replace(placeholder, EE::C(Primitive::from(result))))
            } else {
                Ok(to.replace(placeholder, EE::Distribution{distribution, args: new}))
            }
        }
        Expr::Constrain{prob, relation, left, right} => {
            let (relation, left, right, prob) = (*relation, *left, *right, *prob);
            if prob < 0. || prob > 1. {
                return Err(PartialEvalErr::InvalidProbability);
            }
            let left = _partial_eval(src, left, to, bindings, name_state, None)?;
            let right = _partial_eval(src, right, to, bindings, name_state, None)?;
            Ok(to.replace(placeholder, EE::Constrain{prob, relation, left, right}))
        }
        Expr::Placeholder => {Err(PartialEvalErr::Placeholder)}
        Expr::Observe{observable: _, observed: _} => {
            Err(PartialEvalErr::Observe)
        }
    }
}

pub type EE = EvalExpr;

fn _clone_at(tree: &mut EvaluatedTree, src_at: ExpressionRef, placeholder: ExpressionRef) -> ExpressionRef {
    let copy = match tree.deref(src_at) {
        EE::C(c) => EE::C(c.clone()),
        EE::Begin(v) => {
            let mut new = v.clone();
            for expr in new.iter_mut() {
                let inner = tree.placeholder();
                *expr = _clone_at(tree, *expr, inner);
            }
            EE::Begin(new)
        },
        EE::Decision{id, body} => {
            let (id, body) = (id.clone(), *body);
            let inner = tree.placeholder();
            EE::Decision{id, body: _clone_at(tree, body, inner)}
        },
        EE::Stochastic{id, body} => {
            let (id, body) = (id.clone(), *body);
            let inner = tree.placeholder();
            EE::Stochastic{id, body: _clone_at(tree, body, inner)}
        },
        EE::If{predicate, consequent, alternative} => {
            let (predicate, consequent, alternative) = (*predicate, *consequent, *alternative);
            let inner_p = tree.placeholder();
            let predicate = _clone_at(tree, predicate, inner_p);
            let inner_c = tree.placeholder();
            let consequent = _clone_at(tree, consequent, inner_c);
            let inner_a = tree.placeholder();
            let alternative = _clone_at(tree, alternative, inner_a);
            EE::If{predicate, consequent, alternative}
        }
        EE::Constrain{prob, relation, left, right} => {
            let (relation, left, right, prob) = (*relation, *left, *right, *prob);
            let inner_l = tree.placeholder();
            let left = _clone_at(tree, left, inner_l);
            let inner_r = tree.placeholder();
            let right = _clone_at(tree, right, inner_r);
            EE::Constrain{prob, relation, left, right}
        }
        EE::Builtin{builtin, args} => {
            let builtin = *builtin;
            let mut args = args.clone();
            for expr in args.iter_mut() {
                let inner = tree.placeholder();
                *expr = _clone_at(tree, *expr, inner);
            }
            EE::Builtin{builtin, args}
        }
        EE::Distribution{distribution, args} => {
            let distribution = *distribution;
            let mut args = args.clone();
            for expr in args.iter_mut() {
                let inner = tree.placeholder();
                *expr = _clone_at(tree, *expr, inner);
            }
            EE::Distribution{distribution, args}
        }
        _ => todo!()
    };
    tree.replace(placeholder, copy)
}


pub fn pretty_print(tree: &EvaluatedTree) {
    pretty_print_at(tree, ExpressionRef{index: 0}, 0)
}

#[inline(always)]
fn indent(indentation: usize) {
    use std::iter::repeat;
    use std::iter::FromIterator;
    print!("{}", String::from_iter(repeat(' ').take(indentation * 2)))
}

pub(crate) fn pretty_print_at(tree: &EvaluatedTree, at: ExpressionRef, indentation: usize) {
    indent(indentation);
    match tree.deref(at) {
        EE::C(c) => println!("{:?}", c),
        EE::VarRef(r) => {
            println!("(ref {})", r);
        }
        EE::Begin(e) => {
            println!("(begin");
            for expr in e {
                pretty_print_at(tree, *expr, indentation+1);
            }
            indent(indentation);
            println!(") ; end begin")
        }
        EE::If{predicate, consequent, alternative} => {
            println!("(if");
            pretty_print_at(tree, *predicate, indentation+1);
            pretty_print_at(tree, *consequent, indentation+1);
            pretty_print_at(tree, *alternative, indentation+1);
            indent(indentation);
            println!(") ; end if")
        }
        EE::Decision{id, body} => {
            println!("(decision {{id={}}}", id);
            pretty_print_at(tree, *body, indentation+1);
            indent(indentation);
            println!("); end decision");
        }
        EE::Stochastic{id, body} => {
            println!("(sample {{id={}}}", id);
            pretty_print_at(tree, *body, indentation+1);
            indent(indentation);
            println!("); end sample");
        }
        EE::Constrain{prob, relation, left, right} => {
            println!("(constrain with-prob={} {}", prob, relation.pretty_print());
            pretty_print_at(tree, *left, indentation+1);
            pretty_print_at(tree, *right, indentation+1);
            indent(indentation);
            println!(") ; end constrain")
        }
        EE::Builtin{builtin, args} => {
            println!("({:?}", builtin);
            for arg in args {
                pretty_print_at(tree, *arg, indentation+1);
            }
            indent(indentation);
            println!(") ; end builtin")
        }
        EE::Distribution{distribution, args} => {
            println!("({:?}", distribution);
            for arg in args {
                pretty_print_at(tree, *arg, indentation+1);
            }
            indent(indentation);
            println!(") ; end distribution")
        }
        EE::Placeholder => {
            println!("({{placeholder}})")
        }
        EE::Deleted => {
            println!("({{deleted}})")
        }
    }
}