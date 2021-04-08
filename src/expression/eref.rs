use std::fmt::Display;
use num_traits::ToPrimitive;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpressionRef {
    idx: u32,
}

impl ExpressionRef {
    pub fn root() -> Self { Self::from(0u32) }
}


impl<T : ToPrimitive + Display> From<T> for ExpressionRef {
    fn from(t: T) -> Self {
        Self {
            idx: match t.to_u32() {
                Some(u) => u,
                None => panic!("{} cannot be represented as u32.", t),
            }
        }
    }
}

pub(crate) fn idx(eref: ExpressionRef) -> usize {
    eref.idx as usize
}
