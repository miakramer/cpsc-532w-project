// use ndarray::prelude::*;
// use common::{*, primitives::*};
// use smallvec::*;


// #[derive(Clone, Debug)]
// pub struct VariableInfo<T : Clone + std::fmt::Debug> {
//     pub name: Identifier,
//     pub domain: T,
//     pub cardinality: usize,
// }


// #[derive(Clone, Debug)]
// pub enum Variable<T : Clone + std::fmt::Debug> {
//     Single(VariableInfo<T>),
//     Multi {
//         group_name: Identifier,
//         members: Vec<VariableInfo<T>>,
//         cardinality: usize,
//     }
// }

// impl<T : Clone + std::fmt::Debug> Variable<T> {
//     pub fn group_name<'a>(&'a self) -> &'a Identifier {
//         match self {
//             Variable::Single(v) => &v.name,
//             Variable::Multi{group_name, members: _, cardinality: _} => &group_name,
//         }
//     }

//     pub fn cardinality(&self) -> usize {
//         match self {
//             Variable::Single(v) => v.cardinality,
//             Variable::Multi{group_name: _, members: _, cardinality} => *cardinality,
//         }
//     }
// }

// pub type DecisionVariable = Variable<Domain>;
// pub type StochasticVariable = Variable<Distribution>;


// #[derive(Copy, Clone, Debug)]
// pub struct StageIndex {
//     pub decision: u32,
//     pub stochastic: u32,
// }

// impl StageIndex {
//     pub fn decision(&self) -> usize {
//         self.decision as usize
//     }

//     pub fn stochastic(&self) -> usize {
//         self.stochastic as usize
//     }
// }

// pub struct QVariableGraph {
//     pub decision: Vec<DecisionVariable>,
//     pub stochastic: Vec<StochasticVariable>,

//     // starting index for each stage
//     pub stages: Vec<StageIndex>,
// }


// pub fn prepare_graph(src: &ScpGraph) -> QVariableGraph {
//     // sort variables into stages here
//     let decision: Vec<DecisionVariable> = todo!();
//     let stochastic: Vec<StochasticVariable> = todo!();

//     // count possible assignments
//     let n_decision = decision.iter().sum()
// }
