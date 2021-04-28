pub use crate::partial_eval::*;
use common::*;

use primitives::Primitive;
use smallvec::SmallVec;



pub fn compile_graph(body: &EvaluatedTree) -> ScpGraph {
    let mut variables = Variables::new();
    gather_variables(&body, body.root(), &mut variables);
    // make_groups(&mut variables);

    let mut dependencies = Vec::new();
    gather_dependencies(&variables, &mut dependencies);

    let mut constraints = Vec::new();
    gather_constraints(&body, body.root(), &variables, &mut constraints, &im::Vector::new());

    // replace variable definitions with simpler versions
    // for var in variables.variables.iter_mut() {
    //     match var {
    //         VariableOrGroup::Group(g) => {
    //             let mut new_def = EvaluatedTree::new();
    //             clone_refs(&g.definition, &mut new_def, g.definition.root());
    //             g.definition = new_def;
    //         }
    //         VariableOrGroup::Variable(v) => {
    //             let mut new_def = EvaluatedTree::new();
    //             clone_refs(&v.definition, &mut new_def, v.definition.root());
    //             v.definition = new_def;
    //         }
    //     }
    // }

    let mut new_body = EvaluatedTree::new();
    clone_refs(body, &mut new_body, body.root());

    ScpGraph{variables, dependencies, constraints, body: new_body}
}

fn dependency<'a>(variables: &Variables, dependencies: &'a mut Vec<Dependency>, name: &Identifier) -> Option<&'a mut Dependency> {
    for d in dependencies {
        if variables.name(d.this) == name {
            return Some(d)
        }
    }
    None
}

// fn make_groups(variables: &mut Variables) {
//     let refs: Vec<VarRef> = variables.iter().collect();
//     for var in refs {
//         make_group(variables, var);
//     }
// }

// fn make_group(variables: &mut Variables, var: VarRef) -> bool {
//     let definition = variables.deref_group(var).definition();
//     let name = variables.name(var);
//     let kind = variables.deref_group(var).kind();

//     let root = definition.deref(definition.root());

//     if let EE::C(_) = root {
//         return false;
//     }
//     else if let EE::Distribution{distribution: _, args: _} = root {
//         return false;
//     }
//     else if let EE::Builtin{builtin, args: _} = root {
//         match builtin {
//             Builtin::IntRange | Builtin::OneOf => {
//                 return false;
//             },
//             _ => {
//                 panic!("Can't yet invert control for {:?}", builtin);
//             }
//         }
//     }
//     else {
//         let mut new = EvaluatedTree::new();
//         let mut name_state = 0;
//         let mut new_names: SmallVec<[Identifier; 4]> = SmallVec::new();
//         let mut new_name = || -> Identifier {
//             let ret: Identifier = format!("{}:{}", name, name_state).into();
//             name_state += 1;
//             new_names.push(ret.clone());
//             ret
//         };
//         invert_control(definition, &mut new, definition.root(), &mut new_name);

//         drop(definition);
//         let name = name.clone();

//         variables.variables[var.id as usize] = VariableOrGroup::Group(
//             VariableGroup {
//                 kind,
//                 group_name: name,
//                 names: new_names,
//                 definition: new,
//             }
//         );

//         true
//     }
// }

// fn invert_control<F>(tree: &EvaluatedTree, to: &mut EvaluatedTree, at: ExpressionRef, new_name: &mut F)  -> ExpressionRef
//     where F : FnMut() -> Identifier  {
//     let placeholder = to.placeholder();

//     match tree.deref(at) {
//         EE::C(c) => match c {
//             Primitive::Distribution(_) => {
//                 let body = to.push(EE::C(c.clone()));
//                 to.replace(placeholder, EE::Stochastic{id: new_name(), body})
//             },
//             Primitive::Domain(_) => {
//                 let body = to.push(EE::C(c.clone()));
//                 to.replace(placeholder, EE::Decision{id: new_name(), body})
//             },
//             _ => to.replace(placeholder, EE::C(c.clone()))
//         },
//         EE::If{predicate, consequent, alternative} => {
//             let predicate = invert_control(tree, to, *predicate, new_name);
//             let consequent = invert_control(tree, to, *consequent, new_name);
//             let alternative = invert_control(tree, to, *alternative, new_name);
//             to.replace(placeholder, EE::If{predicate, consequent, alternative})
//         }
//         EE::Decision{id, body} => {
//             let id = id.clone();
//             let body = clone(tree, to, *body);
//             to.replace(placeholder, EE::Decision{id, body})
//         }
//         EE::Stochastic{id, body} => {
//             let id = id.clone();
//             let body = clone(tree, to, *body);
//             to.replace(placeholder, EE::Decision{id, body})
//         }
//         EE::Constrain{prob, relation, left, right} => {
//             let left = clone(tree, to, *left);
//             let right = clone(tree, to, *right);
//             to.replace(placeholder, EE::Constrain{prob: *prob, relation: *relation, left, right})
//         }
//         EE::Builtin{builtin, args} => {
//             let mut new_args = Vec::with_capacity(args.len());
//             for arg in args {
//                 new_args.push(clone(tree, to, *arg));
//             }
//             match builtin {
//                 Builtin::IntRange | Builtin::OneOf => {
//                     let body = to.push(EE::Builtin{builtin: *builtin, args: new_args});
//                     to.replace(placeholder, EE::Decision{id: new_name(), body})
//                 },
//                 _ => {
//                     to.replace(placeholder, EE::Builtin{builtin: *builtin, args: new_args})
//                 }
//             }
//         }
//         EE::Distribution{distribution, args} => {
//             let mut new_args = Vec::with_capacity(args.len());
//             for arg in args {
//                 new_args.push(clone(tree, to, *arg));
//             }
//             let body = to.push(EE::Distribution{distribution: *distribution, args: new_args});
//             to.replace(placeholder, EE::Stochastic{id: new_name(), body})
//         }
//         _ => todo!()
//     }
// }

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
                clone_refs(evald, &mut new_body, *body);
                variables.push(VariableKind::Decision, id.clone(), new_body);
            }
        }
        EE::Stochastic{id, body} => {
            if !variables.has_name(id) {
                let mut new_body = EvaluatedTree::new();
                clone_refs(evald, &mut new_body, *body);
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
        EE::VarRef(vr) => EE::VarRef(vr.clone()),
        _ => todo!()
    };
    to.replace(placeholder, copy)
}

fn clone_refs(from: &EvaluatedTree, to: &mut EvaluatedTree, src_at: ExpressionRef) -> ExpressionRef {
    let placeholder = to.placeholder();
    let copy = match from.deref(src_at) {
        EE::C(c) => EE::C(c.clone()),
        EE::Begin(v) => {
            // let mut new = v.clone();
            // for expr in new.iter_mut() {
            //     *expr = clone_refs(from, to, *expr);
            // }
            let mut new = Vec::with_capacity(1);
            new.push(clone_refs(from, to, *v.last().unwrap()));
            EE::Begin(new)
        },
        EE::Decision{id, body: _} => {
            EE::VarRef(id.clone())//{id: id.clone(), body: clone(from, to, *body)}
        },
        EE::Stochastic{id, body: _} => {
            // let (id, body) = (id.clone(), *body);
            // EE::Stochastic{id, body: clone(from, to, body)}
            EE::VarRef(id.clone())
        },
        EE::If{predicate, consequent, alternative} => {
            let (predicate, consequent, alternative) = (*predicate, *consequent, *alternative);
            let predicate = clone_refs(from, to, predicate);
            let consequent = clone_refs(from, to, consequent);
            let alternative = clone_refs(from, to, alternative);
            EE::If{predicate, consequent, alternative}
        }
        EE::Constrain{prob: _, relation: _, left: _, right: _} => {
            // let (relation, left, right) = (*relation, *left, *right);
            // let left = clone_refs(from, to, left);
            // let right = clone_refs(from, to, right);
            // EE::Constrain{prob: *prob, relation, left, right}
            EE::Builtin{builtin: Builtin::Nil, args: Vec::new()}
        }
        EE::Builtin{builtin, args} => {
            let builtin = *builtin;
            let mut args = args.clone();
            for expr in args.iter_mut() {
                *expr = clone_refs(from, to, *expr);
            }
            EE::Builtin{builtin, args}
        }
        EE::Distribution{distribution, args} => {
            let distribution = *distribution;
            let mut args = args.clone();
            for expr in args.iter_mut() {
                *expr = clone_refs(from, to, *expr);
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
        if refs.len() > 0 {
            dependencies.push(Dependency{
                this: VarRef{
                    kind: var.kind,
                    id: i as u32,
                },
                depends_on: refs
            })
        }
    }
}

fn hasref(refs: &[VarRef], variables: &Variables, name: &Identifier) -> bool {
    for r in refs {
        if &variables.deref(*r).name == name {
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
        EE::VarRef(id) => {
            if id != this {
                if !hasref(&refs, variables, id) {
                    refs.push(variables.get_by_name(id).unwrap())
                }
            }
        },
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
            clone_refs(tree, &mut new_left, *left);
            let mut new_right = EvaluatedTree::new();
            clone_refs(tree, &mut new_right, *right);
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
    for var in graph.variables.iter() {
        let v = graph.variables.deref(var);
        println!("• {}, {:?}", &v.name, var.kind);
        
        if let Some(d) = graph.dependencies_of(var) {
            print!("  → depends on:");
            for r in &d.depends_on {
                let d = graph.variables.deref(*r);
                print!(" {}", &d.name);
            }
            println!();
        }
        
        println!("  → definition:");
        pretty_print_at(&v.definition, v.definition.root(), 2);
            // VariableOrGroup::Group(g) => {
            //     print!("• Group {} ( ", g.group_name);
            //     for (i, _v) in graph.variables.iter_group(var).unwrap().enumerate() {
            //         print!("{} ", g.names[i]);
            //     }
            //     println!("), {:?}", g.kind);

            //     if let Some(d) = graph.dependencies_of(var) {
            //         print!("  → depends on:");
            //         for r in &d.depends_on {
            //             let d = graph.variables.deref_group(*r);
            //             print!(" {}", &d.name());
            //         }
            //         println!();
            //     }

            //     println!("  → definition:");
            //     pretty_print_at(&g.definition, g.definition.root(), 2);
            // }
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

    println!("\nBody:");
    pretty_print_at(&graph.body, graph.body.root(), 1);
}