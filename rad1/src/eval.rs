use chess::Board;

mod naive;

pub trait Evaluator<T>
where
    T: PartialEq + PartialOrd,
{
    fn min_value(&self) -> T;
    fn max_value(&self) -> T;
    fn evaluate(&self, board: &Board) -> T;
}

pub fn naive_evaluator() -> impl Evaluator<i16> {
    naive::NaiveEvaluator::default()
}
