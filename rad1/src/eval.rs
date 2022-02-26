use chess::Board;

pub mod naive;

pub trait Evaluator {
    type Result;
    fn min_value(&self) -> Self::Result;
    fn max_value(&self) -> Self::Result;
    fn evaluate(&self, board: &Board) -> Self::Result;
}

pub fn naive_evaluator() -> naive::NaiveEvaluator {
    naive::NaiveEvaluator
}
