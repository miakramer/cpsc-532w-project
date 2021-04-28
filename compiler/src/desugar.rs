use common::*;
use smol_str::SmolStr;

use crate::parser;


const SMALL_ARG_SIZE: usize = 4;

#[derive(Clone, Debug)]
pub enum Expr<Var> {
    C(C),
    V(Var),
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
        prob: f64,
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
    Placeholder
}

#[derive(Clone, Debug)]
pub struct ExpressionTree<Var> {
    expressions: Vec<Expr<Var>>,
}

impl<Var : std::fmt::Debug> ExpressionTree<Var> {
    pub fn new() -> Self {
        Self {
            expressions: Vec::new(),
        }
    }

    pub fn root(&self) -> ExpressionRef {
        ExpressionRef { index: 0 }
    }

    pub fn push(&mut self, expr: Expr<Var>) -> ExpressionRef {
        let l = self.expressions.len() as u32;
        self.expressions.push(expr);
        ExpressionRef { index: l }
    }

    pub fn placeholder(&mut self) -> ExpressionRef {
        let ret = self.push(Expr::Placeholder);
        // println!(" -> Creating placeholder @ {}", ret.index);
        ret
    }

    pub fn replace(&mut self, at: ExpressionRef, with: Expr<Var>) -> ExpressionRef {
        match self.expressions[at.index as usize] {
            Expr::Placeholder => (),
            _ => panic!("Replacing non-placeholder value: {:?}\n -> with {:?}", &self.expressions[at.index as usize], &with)
        };
        // println!(" -> Replacing placeholder @ {}", at.index);
        self.expressions[at.index as usize] = with;
        at
    }

    pub fn deref<'a>(&'a self, at: ExpressionRef) -> &Expr<Var> {
        &self.expressions[at.index as usize]
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    // pub proclaim: ProclaimThreshold,
    pub body: ExpressionTree<Identifier>,
}

#[derive(Clone, Debug)]
pub struct Defn {
    name: Identifier,
    args: Vec<Identifier>,
    body: ExpressionTree<Identifier>,
}

#[derive(Debug)]
pub enum DesugarError {
    UndefinedProcedure(SmolStr),
    UnscopedVariable(SmolStr),
}

pub fn desugar(raw: &parser::Program) -> Result<Program, DesugarError> {
    let mut prog = Program {
        // proclaim: raw.proclaim,
        body: ExpressionTree::new(),
    };

    let mut defns = Vec::with_capacity(raw.defns.len());

    let mut name_state = 0u32;

    for defn in &raw.defns {
        let mut expr = ExpressionTree::new();
        desugar_exprs(&mut expr, &defn.body,&defns, &mut name_state)?;

        let name = ident(&defn.name, &mut name_state);

        defns.push(Defn {
            name: name.clone(),
            args: defn
                .args
                .iter()
                .map(|i| ident(i, &mut name_state))
                .collect(),
            body: expr,
        });

        // println!("\nFinished desugaring procedure {}:", name);
        // pretty_print(&prog.defns.last().unwrap().body);
    }

    // println!();
    desugar_exprs(&mut prog.body, &raw.body, &defns, &mut name_state)?;

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
            let mut i = 0;
            while buf[i] != 0 {
                i += 1;
            }
            let s = std::str::from_utf8(&buf[..i]).unwrap();

            *state += 1;
        
            Identifier::from(s)
        },
        parser::Identifier::Ident(i) => i.clone()
    }
}

fn procedure_by_name<'a>(name: &Identifier, procedures: &'a [Defn]) -> Option<&'a Defn> {
    for proc in procedures.iter() {
        if &proc.name == name {
            return Some(proc);
        }
    }
    None
}

fn desugar_exprs(tree: &mut ExpressionTree<Identifier>, source: &parser::Expr, procedures: &[Defn], name_state: &mut u32) -> Result<ExpressionRef, DesugarError> {
    let placeholder = tree.placeholder();
    match source {
        parser::Expr::C(c) => Ok(tree.replace(placeholder, Expr::C(*c))),
        parser::Expr::V(i) => Ok(tree.replace(placeholder, Expr::V(ident(i, name_state)))),
        parser::Expr::Begin(exprs) => {
            let mut refs = Vec::with_capacity(exprs.len());
            for expr in exprs {
                refs.push(desugar_exprs(tree, expr, procedures, name_state)?);
            }
            Ok(tree.replace(placeholder, Expr::Begin(refs)))
        },
        parser::Expr::If{predicate, consequent, alternative} => {
            let predicate = desugar_exprs(tree, predicate, procedures, name_state)?;
            let consequent = desugar_exprs(tree, consequent, procedures, name_state)?;
            let alternative = desugar_exprs(tree, alternative, procedures, name_state)?;
            Ok(tree.replace(placeholder, Expr::If{predicate, consequent, alternative}))
        },
        parser::Expr::Let{bindings, body} => {
            desugar_exprs_let(tree, placeholder, procedures, name_state, &bindings[..], body)
        },
        parser::Expr::Sample(e) => {
            let e = desugar_exprs(tree, e, procedures, name_state)?;
            Ok(tree.replace(placeholder, Expr::Sample(e)))
        },
        parser::Expr::Observe{observable, observed} => {
            let observable = desugar_exprs(tree, observable, procedures, name_state)?;
            let observed = desugar_exprs(tree, observed, procedures, name_state)?;
            Ok(tree.replace(placeholder, Expr::Observe{observable, observed}))
        }
        parser::Expr::Decision(d) => {
            let d = desugar_exprs(tree, d, procedures, name_state)?;
            Ok(tree.replace(placeholder, Expr::Decision(d)))
        }
        parser::Expr::Constrain{prob, relation, left, right} => {
            let left = desugar_exprs(tree, left, procedures, name_state)?;
            let right = desugar_exprs(tree, right, procedures, name_state)?;
            Ok(tree.replace(placeholder, Expr::Constrain{prob: *prob, relation: *relation, left, right}))
        }
        parser::Expr::Apply{head, body} => {
            let name = ident(head, name_state);
            let mut args = Vec::with_capacity(body.len());
            for e in body {
                args.push(desugar_exprs(tree, e, procedures, name_state)?);
            }
            if let Some(builtin) = common::Builtin::maybe_match(name.as_str()) {
                Ok(tree.replace(placeholder, Expr::Builtin{builtin, args}))
            } else if let Some(distribution) = DistributionType::maybe_match(name.as_str()) {
                Ok(tree.replace(placeholder, Expr::Distribution{distribution, args}))
            } else if let Some(proc) = procedure_by_name(&name, procedures) {
                desugar_exprs_proc(tree, placeholder, proc.body.root(), body, proc, procedures, name_state, &im::Vector::new())
            } else {
                Err(DesugarError::UndefinedProcedure(name))
            }
        }
    }
}

fn desugar_exprs_let(tree: &mut ExpressionTree<Identifier>, placeholder: ExpressionRef, procedures: &[Defn], name_state: &mut u32, bindings: &[(parser::Identifier, parser::Expr)], body: &parser::Expr) -> Result<ExpressionRef, DesugarError> {
    if bindings.len() == 0 {
        panic!("Shouldn't happen")
    } else if bindings.len() == 1 {
        // let placeholder = tree.placeholder();
        let (i, e) = &bindings[0];
        let value = desugar_exprs(tree, e, procedures, name_state)?;
        let name = ident(i, name_state);
        let body = desugar_exprs(tree, body, procedures, name_state)?;
        Ok(tree.replace(placeholder, Expr::Let{name, value, body}))
    } else {
        let (i, e) = &bindings[0];
        let name = ident(i, name_state);
        let value = desugar_exprs(tree, e, procedures, name_state)?;
        let inner_placeholder = tree.placeholder();
        let body = desugar_exprs_let(tree, inner_placeholder, procedures, name_state, &bindings[1..], body)?;
        Ok(tree.replace(placeholder, Expr::Let{name, value, body}))
    }
}

fn get_arg_ord(arg: &Identifier, args: &[Identifier]) -> Option<usize> {
    args.into_iter().enumerate().skip_while(|(_i, x)| *x != arg).map(|(i, _x)| i).next()
}

fn push_name(name: &Identifier, names: &im::Vector<Identifier>) -> im::Vector<Identifier> {
    let mut names = names.clone();
    names.push_back(name.clone());
    names
}

fn desugar_exprs_proc(tree: &mut ExpressionTree<Identifier>, placeholder: ExpressionRef, at: ExpressionRef, call: &[parser::Expr], proc: &Defn, procedures: &[Defn], name_state: &mut u32, scoped_names: &im::Vector<Identifier>) -> Result<ExpressionRef, DesugarError> {
    // let placeholder = tree.placeholder();
    let expr = proc.body.deref(at);
    if let Expr::C(c) = expr {
        Ok(tree.replace(placeholder, Expr::C(*c)))
    } else if let Expr::V(v) = expr {
        if let Some(o) = get_arg_ord(v, &proc.args) {
            desugar_exprs(tree, &call[o], procedures, name_state)
        } else if scoped_names.contains(v) {
            Ok(tree.replace(placeholder, Expr::V(v.clone())))
        } else {
            Err(DesugarError::UnscopedVariable(v.clone()))
        }
    } else if let Expr::Begin(exprs) = expr {
        let mut new_exprs = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let inner_placeholder = tree.placeholder();
            new_exprs.push(desugar_exprs_proc(tree, inner_placeholder, *expr, call, proc, procedures, name_state, scoped_names)?);
        }
        Ok(tree.replace(placeholder, Expr::Begin(new_exprs)))
    } else if let Expr::If{predicate, consequent, alternative} = expr {
        let inner_placeholder = tree.placeholder();
        let predicate = desugar_exprs_proc(tree, inner_placeholder, *predicate, call, proc, procedures, name_state, scoped_names)?;
        let consequent = desugar_exprs_proc(tree, inner_placeholder, *consequent, call, proc, procedures, name_state, scoped_names)?;
        let alternative = desugar_exprs_proc(tree, inner_placeholder, *alternative, call, proc, procedures, name_state, scoped_names)?;
        Ok(tree.replace(placeholder, Expr::If{predicate, consequent, alternative}))
    } else if let Expr::Let{name, value, body} = expr {
        let inner_placeholder = tree.placeholder();
        let value = desugar_exprs_proc(tree,  inner_placeholder,*value, call, proc, procedures, name_state, scoped_names)?;
        let scoped_names = push_name(name, scoped_names);
        let body = desugar_exprs_proc(tree, inner_placeholder, *body, call, proc, procedures, name_state, &scoped_names)?;
        Ok(tree.replace(placeholder, Expr::Let{name: name.clone(), value, body}))
    } else if let Expr::Sample(s) = expr {
        let inner_placeholder = tree.placeholder();
        let s = desugar_exprs_proc(tree, inner_placeholder, *s, call, proc, procedures, name_state, scoped_names)?;
        Ok(tree.replace(placeholder, Expr::Sample(s)))
    } else if let Expr::Observe{observable, observed} = expr {
        let inner_placeholder = tree.placeholder();
        let observable = desugar_exprs_proc(tree, inner_placeholder, *observable, call, proc, procedures, name_state, scoped_names)?;
        let observed = desugar_exprs_proc(tree, inner_placeholder, *observed, call, proc, procedures, name_state, scoped_names)?;
        Ok(tree.replace(placeholder, Expr::Observe{observable, observed}))
    } else if let Expr::Decision(d) = expr {
        let inner_placeholder = tree.placeholder();
        let d = desugar_exprs_proc(tree, inner_placeholder, *d, call, proc, procedures, name_state, scoped_names)?;
        Ok(tree.replace(placeholder, Expr::Decision(d)))
    } else if let Expr::Constrain{prob, relation, left, right} = expr {
        let inner_placeholder = tree.placeholder();
        let left = desugar_exprs_proc(tree, inner_placeholder, *left, call, proc, procedures, name_state, scoped_names)?;
        let right = desugar_exprs_proc(tree, inner_placeholder, *right, call, proc, procedures, name_state, scoped_names)?;
        Ok(tree.replace(placeholder, Expr::Constrain{prob: *prob, relation: *relation, left, right}))
    } else if let Expr::Builtin{builtin, args} = expr {
        let mut new_args = Vec::with_capacity(args.len());
        for arg in args {
            let inner_placeholder = tree.placeholder();
            new_args.push(desugar_exprs_proc(tree, inner_placeholder, *arg, call, proc, procedures, name_state, scoped_names)?)
        }
        Ok(tree.replace(placeholder, Expr::Builtin{builtin: *builtin, args: new_args}))
    } else if let Expr::Distribution{distribution, args} = expr {
        let mut new_args = Vec::with_capacity(args.len());
        for arg in args {
            let inner_placeholder = tree.placeholder();
            new_args.push(desugar_exprs_proc(tree, inner_placeholder, *arg, call, proc, procedures, name_state, scoped_names)?)
        }
        Ok(tree.replace(placeholder, Expr::Distribution{distribution: *distribution, args: new_args}))
    } else {
        panic!("Encountered placeholder value")
    }
}

pub fn pretty_print(tree: &ExpressionTree<Identifier>) {
    pretty_print_at(tree, ExpressionRef{index: 0}, 0)
}

#[inline(always)]
fn indent(indentation: usize) {
    use std::iter::repeat;
    use std::iter::FromIterator;
    print!("{}", String::from_iter(repeat(' ').take(indentation * 2)))
}

fn pretty_print_at(tree: &ExpressionTree<Identifier>, at: ExpressionRef, indentation: usize) {
    indent(indentation);
    match tree.deref(at) {
        Expr::C(c) => println!("{:?}", c),
        Expr::V(v) => println!("{:?}", v),
        Expr::Begin(e) => {
            println!("(begin");
            for expr in e {
                pretty_print_at(tree, *expr, indentation+1);
            }
            indent(indentation);
            println!(") ; end begin")
        }
        Expr::If{predicate, consequent, alternative} => {
            println!("(if");
            pretty_print_at(tree, *predicate, indentation+1);
            pretty_print_at(tree, *consequent, indentation+1);
            pretty_print_at(tree, *alternative, indentation+1);
            indent(indentation);
            println!(") ; end if")
        }
        Expr::Let{name, value, body} => {
            println!("(let [{}", name);
            pretty_print_at(tree, *value, indentation+1);
            indent(indentation);
            println!("] ; in:");
            pretty_print_at(tree, *body, indentation+1);
            indent(indentation);
            println!(") ; end let")
        }
        Expr::Sample(s) => {
            println!("(sample");
            pretty_print_at(tree, *s, indentation+1);
            indent(indentation);
            println!(") ; end sample")
        }
        Expr::Observe{observable, observed} => {
            println!("(observe");
            pretty_print_at(tree, *observable, indentation+1);
            pretty_print_at(tree, *observed, indentation+1);
            indent(indentation);
            println!(") ; end observe")
        }
        Expr::Decision(d) => {
            println!("(decision");
            pretty_print_at(tree, *d, indentation+1);
            indent(indentation);
            println!(") ; end decision")
        }
        Expr::Constrain{prob, relation, left, right} => {
            println!("(constrain with-prob={} {}", prob, relation.pretty_print());
            pretty_print_at(tree, *left, indentation+1);
            pretty_print_at(tree, *right, indentation+1);
            indent(indentation);
            println!(") ; end constrain")
        }
        Expr::Builtin{builtin, args} => {
            println!("({:?}", builtin);
            for arg in args {
                pretty_print_at(tree, *arg, indentation+1);
            }
            indent(indentation);
            println!(") ; end builtin")
        }
        Expr::Distribution{distribution, args} => {
            println!("({:?}", distribution);
            for arg in args {
                pretty_print_at(tree, *arg, indentation+1);
            }
            indent(indentation);
            println!(") ; end distribution")
        }
        Expr::Placeholder => {
            println!("({{placeholder}})")
        }
    }
}