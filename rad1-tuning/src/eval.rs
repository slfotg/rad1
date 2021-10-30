use chess::{BitBoard, Board, BoardStatus, Piece};
use rad1::eval::Evaluator;

pub mod config;

pub struct PositionEvaluator {
    config: config::EvaluationConfig,
}

impl PositionEvaluator {
    const MIN: i16 = -30000;
    const MAX: i16 = 30000;

    pub fn new(config: config::EvaluationConfig) -> Self {
        Self { config }
    }

    fn total_piece_score(
        &self,
        pawns: &BitBoard,
        knights: &BitBoard,
        bishops: &BitBoard,
        rooks: &BitBoard,
        queens: &BitBoard,
        kings: &BitBoard,
    ) -> i16 {
        let mut score = 0;
        score += self.config.opening_values.piece_value(Piece::Pawn) * pawns.count() as i16;
        score += self.config.opening_values.piece_value(Piece::Knight) * knights.count() as i16;
        score += self.config.opening_values.piece_value(Piece::Bishop) * bishops.count() as i16;
        score += self.config.opening_values.piece_value(Piece::Rook) * rooks.count() as i16;
        score += self.config.opening_values.piece_value(Piece::Queen) * queens.count() as i16;
        score += self.config.opening_values.piece_value(Piece::King) * kings.count() as i16;
        return score;
    }

    fn evaluate_opening(&self, board: &Board) -> i16 {
        self.config.opening_values.evaluate(board)
    }

    fn evaluate_middle_game(&self, board: &Board, total_piece_score: i16) -> i16 {
        let opening_score = self.evaluate_opening(board) as f32;
        let endgame_score = self.evaluate_endgame(board) as f32;
        let low = self.config.endgame_weight as f32;
        let high = self.config.opening_weight as f32;
        let t = total_piece_score as f32;
        let diff = high - low;
        let low_factor = (high - t) / diff;
        let high_factor = (t - low) / diff;
        let score = opening_score * high_factor + endgame_score * low_factor;
        score.round() as i16
    }

    fn evaluate_endgame(&self, board: &Board) -> i16 {
        self.config.endgame_values.evaluate(board)
    }
}

impl Evaluator<i16> for PositionEvaluator {
    #[inline]
    fn min_value(&self) -> i16 {
        Self::MIN
    }

    #[inline]
    fn max_value(&self) -> i16 {
        Self::MAX
    }

    #[inline]
    fn evaluate(&self, board: &Board) -> i16 {
        match board.status() {
            BoardStatus::Stalemate => 0,
            BoardStatus::Checkmate => self.min_value(),
            BoardStatus::Ongoing => {
                let pawns = board.pieces(Piece::Pawn);
                let knights = board.pieces(Piece::Knight);
                let bishops = board.pieces(Piece::Bishop);
                let rooks = board.pieces(Piece::Rook);
                let queens = board.pieces(Piece::Queen);
                let kings = board.pieces(Piece::King);

                let total_piece_score =
                    self.total_piece_score(pawns, knights, bishops, rooks, queens, kings);
                if total_piece_score >= self.config.opening_weight {
                    return self.evaluate_opening(board);
                } else if total_piece_score <= self.config.endgame_weight {
                    return self.evaluate_endgame(board);
                } else {
                    return self.evaluate_middle_game(board, total_piece_score);
                }
            }
        }
    }
}
