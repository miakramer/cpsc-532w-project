use ndarray::prelude::*;
use common::{*, primitives::*};
use smallvec::*;

mod variables;
use variables::*;


pub struct Q {
    // store active and previous
    q0: Array2<f32>,
    q1: Array2<f32>,
}

impl Q {
    pub fn new(_graph: &ScpGraph) -> Q {
        todo!()
        // get all stochastic variables, take the product of their cardinality
        // get all decision variables, take the product of their cardinality
        // allocate new Qs, whichhot = false
    }

    pub fn end_round(&mut self) {
        todo!()
        // do some stuff
        // swap the buffers
    }

    // other useful functions go here
}

pub fn q_learn(graph: &ScpGraph) -> Q {
    let mut q = Q::new(graph);



    q
}