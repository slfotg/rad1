use crate::Position;

pub mod naive;

pub trait Evaluator {
    type Result;
    fn min_value(&self) -> Self::Result;
    fn max_value(&self) -> Self::Result;
    fn evaluate(&self, position: &Position) -> Self::Result;
}

pub fn naive_evaluator() -> naive::NaiveEvaluator {
    naive::NaiveEvaluator
}
