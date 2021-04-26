// pub use crate::parser::{ProclaimThreshold, Relation, C};
// pub use crate::desugar::*;
// // use common::DistributionType;

// mod expressions;
// pub use expressions::*;


// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub enum VariableKind {
//     Stochastic,
//     Decision,
// }


// #[derive(Clone, Copy, Debug)]
// pub struct VarRef {
//     kind: VariableKind,
//     id: u32
// }


// #[derive(Clone, Debug)]
// pub struct Variable {
//     kind: VariableKind,
//     pub name: Identifier,
//     pub offset: u32,
//     pub definition: ExpressionTree<Identifier>,
// }


// impl Variable {
//     pub fn format(&self) -> String {
//         format!("{}${}", self.name, self.offset)
//     }
// }


// #[derive(Clone, Debug)]
// pub struct Variables {
//     variables: Vec<Variable>,
//     // definitions: ,
// }

// impl Variables {
//     pub fn deref<'a>(&'a self, at: &VarRef) -> &'a Variable {
//         &self.variables[at.id as usize]
//     }

//     pub fn new() -> Self {
//         Self { variables: Vec::new() }
//     }

//     pub fn push(&mut self, name: Identifier, kind: VariableKind, definition: ) -> VarRef {
//         let id = self.variables.len() as u32;
//         let offset = self.offset_for_name(&name);
//         self.variables.push(Variable{
//             name,
//             kind,
//             offset,
//         });
//         VarRef {
//             kind,
//             id
//         }
//     }

//     fn offset_for_name(&self, name: &Identifier) -> u32 {
//         for var in self.variables.iter().rev() {
//             if &var.name == name {
//                 return var.offset + 1;
//             }
//         }
//         0
//     }
// }

// #[derive(Clone, Debug)]
// pub struct Constraint {
//     pub relation: Relation,
//     pub left: ExpressionTree<VarRef>,
//     pub right: ExpressionTree<VarRef>,
//     pub when: Predicate,
// }

// #[derive(Clone, Debug)]
// pub struct Dependency {
//     pub this: VarRef,
//     pub depends_on: VarRef,
// }

// #[derive(Clone, Debug)]
// pub struct ScpGraph {
//     pub variables: Variables,
//     pub dependencies: Vec<Dependency>,
//     pub links: Vec<ExpressionTree<VarRef>>,
//     pub constraints: Vec<Constraint>,
// }

// fn fresh(state: &mut u32) -> Identifier {
//     use std::io::Write;

//     let mut buf = [0u8; 22];
//     {
//         let mut br = &mut buf[..];
//         write!(br, "@{}", state).unwrap();
//     }
//     let mut i = 0;
//     while buf[i] != 0 {
//         i += 1;
//     }
//     let s = std::str::from_utf8(&buf[..i]).unwrap();

//     *state += 1;

//     Identifier::from(s)
// }

// pub fn gather_variables(tree: &ExpressionTree<Identifier>) -> Variables {
//     let mut variables = Variables::new();
//     let mut name_state = 0;
//     gather_variables_at(&mut variables, tree, tree.root(), None, &mut name_state);
//     variables
// }

// fn gather_variables_at(variables: &mut Variables, tree: &ExpressionTree<Identifier>, at: ExpressionRef, in_let: Option<&Identifier>, name_state: &mut u32) {
//     match tree.deref(at) {
//         Expr::Begin(v) => {
//             for expr in v {
//                 gather_variables_at(variables, tree, *expr, in_let, name_state);
//             }
//         }
//         Expr::If{predicate, consequent, alternative} => {
//             gather_variables_at(variables, tree, *predicate, None, name_state);
//             gather_variables_at(variables, tree, *consequent, None, name_state);
//             gather_variables_at(variables, tree, *alternative, None, name_state);
//         },
//         Expr::Let{name, value, body} => {
//             gather_variables_at(variables, tree, *value, None, name_state);
//             gather_variables_at(variables, tree, *body, Some(name), name_state);
//         },
//         Expr::Sample(e) => {
//             if let Some(n) = in_let {
//                 variables.push()
//             }
//         }
//         _ => (),
//     }
// }