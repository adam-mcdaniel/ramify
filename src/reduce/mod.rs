pub(crate) mod ast;
pub(crate) mod constructor;
pub(crate) mod data;
pub(crate) mod expression;
pub(crate) mod function;
use crate::lambda::Lambda;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ReductionError {}

pub trait Reduce<T> {
    fn reduce(&self, t: &T) -> Result<Lambda, ReductionError>;
}
