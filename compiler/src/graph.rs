pub use crate::parser::{ProclaimThreshold, Relation, C};
pub use crate::partial_eval::*;

use smallvec::SmallVec;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariableKind {
    Stochastic,
    Decision,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VarRef {
    pub kind: VariableKind,
    pub id: u32
}


#[derive(Clone, Debug)]
pub struct Variable {
    pub kind: VariableKind,
    pub name: Identifier,
    pub definition: EvaluatedTree,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Predicate {
    pub pred: EvaluatedTree,
    pub negated: bool,
}

#[derive(Clone, Debug)]
pub struct Constraint {
    pub probability: f64,
    pub relation: Relation,
    pub left: EvaluatedTree,
    pub right: EvaluatedTree,
    pub predicate: SmallVec<[Predicate; 8]>,
}

#[derive(Clone, Debug)]
pub struct Dependency {
    pub this: VarRef,
    pub depends_on: SmallVec<[VarRef; 8]>,
}

#[derive(Clone, Debug)]
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


pub fn compile_graph(body: EvaluatedTree) -> ScpGraph {
    let mut variables = Variables::new();
    gather_variables(&body, body.root(), &mut variables);
    let mut dependencies = Vec::new();
    gather_dependencies(&variables, &mut dependencies);
    let mut constraints = Vec::new();
    gather_constraints(&body, body.root(), &variables, &mut constraints, &im::Vector::new());

    ScpGraph{variables, dependencies, constraints, body}
}

fn gather_variables(evald: &EvaluatedTree, at: ExpressionRef, variables: &mut Variables) {
    match evald.deref(at) {
        EE::C(_) => (),
        EE::Begin(v) => {
            for expr in v {
                gather_variables(evald, *expr, variables);
            }
        }
        EE::Decision{id, body} => {
            if !variables.has_name(id) {
                let mut new_body = EvaluatedTree::new();
                clone(evald, &mut new_body, *body);
                variables.push(VariableKind::Decision, id.clone(), new_body);
            }
        }
        EE::Stochastic{id, body} => {
            if !variables.has_name(id) {
                let mut new_body = EvaluatedTree::new();
                clone(evald, &mut new_body, *body);
                variables.push(VariableKind::Stochastic, id.clone(), new_body);
            }
        }
        EE::If{predicate, consequent, alternative} => {
            gather_variables(evald, *predicate, variables);
            gather_variables(evald, *consequent, variables);
            gather_variables(evald, *alternative, variables);
        }
        EE::Constrain{prob: _, relation: _, left, right} => {
            gather_variables(evald, *left, variables);
            gather_variables(evald, *right, variables);
        }
        EE::Builtin{builtin: _, args} => {
            for expr in args {
                gather_variables(evald, *expr, variables);
            }
        }
        EE::Distribution{distribution: _, args} => {
            for expr in args {
                gather_variables(evald, *expr, variables);
            }
        }
        _ => unreachable!()
    }
}

fn clone(from: &EvaluatedTree, to: &mut EvaluatedTree, src_at: ExpressionRef) -> ExpressionRef {
    let placeholder = to.placeholder();
    let copy = match from.deref(src_at) {
        EE::C(c) => EE::C(c.clone()),
        EE::Begin(v) => {
            let mut new = v.clone();
            for expr in new.iter_mut() {
                *expr = clone(from, to, *expr);
            }
            EE::Begin(new)
        },
        EE::Decision{id, body} => {
            EE::Decision{id: id.clone(), body: clone(from, to, *body)}
        },
        EE::Stochastic{id, body} => {
            let (id, body) = (id.clone(), *body);
            EE::Stochastic{id, body: clone(from, to, body)}
        },
        EE::If{predicate, consequent, alternative} => {
            let (predicate, consequent, alternative) = (*predicate, *consequent, *alternative);
            let predicate = clone(from, to, predicate);
            let consequent = clone(from, to, consequent);
            let alternative = clone(from, to, alternative);
            EE::If{predicate, consequent, alternative}
        }
        EE::Constrain{prob, relation, left, right} => {
            let (relation, left, right) = (*relation, *left, *right);
            let left = clone(from, to, left);
            let right = clone(from, to, right);
            EE::Constrain{prob: *prob, relation, left, right}
        }
        EE::Builtin{builtin, args} => {
            let builtin = *builtin;
            let mut args = args.clone();
            for expr in args.iter_mut() {
                *expr = clone(from, to, *expr);
            }
            EE::Builtin{builtin, args}
        }
        EE::Distribution{distribution, args} => {
            let distribution = *distribution;
            let mut args = args.clone();
            for expr in args.iter_mut() {
                *expr = clone(from, to, *expr);
            }
            EE::Distribution{distribution, args}
        }
        _ => todo!()
    };
    to.replace(placeholder, copy)
}

fn gather_dependencies(variables: &Variables, dependencies: &mut Vec<Dependency>) {
    for (i, var) in variables.variables.iter().enumerate() {
        let mut refs: SmallVec<[VarRef; 8]> = SmallVec::new();
        gather_dependencies_at(&var.definition, var.definition.root(), variables, &var.name, &mut refs);
        dependencies.push(Dependency{
            this: VarRef{
                kind: var.kind,
                id: i as u32
            },
            depends_on: refs
        })
    }
}

fn hasref(refs: &[VarRef], variables: &Variables, name: &Identifier) -> bool {
    for r in refs {
        if &variables.deref(r).name == name {
            return true;
        }
    }
    false
}

fn gather_dependencies_at(tree: &EvaluatedTree, at: ExpressionRef, variables: &Variables, this: &Identifier, refs: &mut SmallVec<[VarRef; 8]>) {
    match tree.deref(at) {
        EE::C(_) => (),
        EE::Begin(v) => {
            for expr in v {
                gather_dependencies_at(tree, *expr, variables, this, refs);
            }
        }
        EE::Decision{id, body} => {
            if id != this {
                if !hasref(&refs, variables, id) {
                    refs.push(variables.get_by_name(id).unwrap())
                }
            } else {
                gather_dependencies_at(tree, *body, variables, this, refs);
            }
        }
        EE::Stochastic{id, body} => {
            if id != this {
                if !hasref(&refs, variables, id) {
                    refs.push(variables.get_by_name(id).unwrap())
                }
            } else {
                gather_dependencies_at(tree, *body, variables, this, refs);
            }
        }
        EE::If{predicate, consequent, alternative} => {
            gather_dependencies_at(tree, *predicate, variables, this, refs);
            gather_dependencies_at(tree, *consequent, variables, this, refs);
            gather_dependencies_at(tree, *alternative, variables, this, refs);
        }
        EE::Constrain{prob: _, relation: _, left, right} => {
            gather_dependencies_at(tree, *left, variables, this, refs);
            gather_dependencies_at(tree, *right, variables, this, refs);
        }
        EE::Builtin{builtin: _, args} => {
            for expr in args {
                gather_dependencies_at(tree, *expr, variables, this, refs);
            }
        }
        EE::Distribution{distribution: _, args} => {
            for expr in args {
                gather_dependencies_at(tree, *expr, variables, this, refs);
            }
        }
        _ => unreachable!()
    }
}

pub fn push(predicate: EvaluatedTree, negated: bool, predicates: &im::Vector<Predicate>) -> im::Vector<Predicate> {
    let mut preds = predicates.clone();
    preds.push_back(Predicate{
        pred: predicate,
        negated
    });
    preds
}

fn gather_constraints(tree: &EvaluatedTree, at: ExpressionRef, variables: &Variables, constraints: &mut Vec<Constraint>, predicates: &im::Vector<Predicate>) {
    match tree.deref(at) {
        EE::C(_) => (),
        EE::Begin(v) => {
            for expr in v {
                gather_constraints(tree, *expr, variables, constraints, predicates);
            }
        }
        EE::Decision{id: _, body: _} => (),
        EE::Stochastic{id: _, body: _} => (),
        EE::If{predicate, consequent, alternative} => {
            gather_constraints(tree, *predicate, variables, constraints, predicates);
            let mut pred = EvaluatedTree::new();
            clone(tree, &mut pred, *predicate);
            gather_constraints(tree, *consequent, variables, constraints, &push(pred.clone(), false, predicates));
            gather_constraints(tree, *alternative, variables, constraints, &push(pred.clone(), true, predicates));
        }
        EE::Constrain{prob, relation, left, right} => {
            let mut new_left = EvaluatedTree::new();
            clone(tree, &mut new_left, *left);
            let mut new_right = EvaluatedTree::new();
            clone(tree, &mut new_right, *right);
            constraints.push(Constraint {
                probability: *prob,
                relation: *relation,
                left: new_left,
                right: new_right,
                predicate: predicates.iter().cloned().collect()
            })
        }
        EE::Builtin{builtin: _, args} => {
            for expr in args {
                gather_constraints(tree, *expr, variables, constraints, predicates);
            }
        }
        EE::Distribution{distribution: _, args} => {
            for expr in args {
                gather_constraints(tree, *expr, variables, constraints, predicates);
            }
        }
        _ => unreachable!()
    }
}


pub fn pretty_print(graph: &ScpGraph) {
    println!("Variables:");
    for var in graph.variables.variables.iter() {
        println!("• {}, {:?}", var.name, var.kind);
        print!("  → depends on:");
        for r in &graph.dependencies_of(graph.get_ref(var)).depends_on {
            let d = graph.variables.deref(r);
            print!(" {}", d.name);
        }
        println!("\n  → definition:");
        pretty_print_at(&var.definition, var.definition.root(), 2);
    }
    println!("\nConstraints:");
    for constraint in graph.constraints.iter() {
        println!("\n• (P={}) {}", constraint.probability, constraint.relation.pretty_print());
        println!("  → left:");
        pretty_print_at(&constraint.left, constraint.left.root(), 2);
        println!("  → right:");
        pretty_print_at(&constraint.right, constraint.right.root(), 2);
        println!("  → when:");
        if constraint.predicate.len() == 0 {
            println!("    true");
        } else {
            for pred in constraint.predicate.iter() {
                println!("    (all");
                if pred.negated {
                    println!("      (not");
                    pretty_print_at(&pred.pred, pred.pred.root(), 4);
                    println!("      )");
                } else {
                    pretty_print_at(&pred.pred, pred.pred.root(), 3);
                }
                println!("    )")
            }
        }
    }
}