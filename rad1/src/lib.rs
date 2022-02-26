use crate::eval::Evaluator;
use chess::BitBoard;
use chess::Board;
use chess::Game;
use chess::MoveGen;
use std::cmp::Ordering;
use std::str::FromStr;

pub mod agent;
pub mod eval;
pub mod tt;

mod move_hash;
mod node;

const EVALUATOR: eval::naive::NaiveEvaluator = eval::naive::NaiveEvaluator {};

// type aliases for now to decouple
// the engine code from chess library being used
pub type Action = chess::Action;
pub type PositionStatus = chess::BoardStatus;
pub type ChessMove = chess::ChessMove;
pub type Color = chess::Color;
pub type Piece = chess::Piece;
pub type Square = chess::Square;
pub type Rank = chess::Rank;
pub type File = chess::File;
pub type ParseError = chess::Error;
pub type GameResult = chess::GameResult;

pub const ALL_SQUARES: [Square; 64] = chess::ALL_SQUARES;
pub const ALL_RANKS: [Rank; 8] = chess::ALL_RANKS;
pub const ALL_FILES: [File; 8] = chess::ALL_FILES;
pub const PROMOTION_PIECES: [Piece; 4] = chess::PROMOTION_PIECES;

pub struct ChessGame {
    game: Game,
}

#[derive(Default)]
pub struct Position {
    board: Board,
}

impl Default for ChessGame {
    fn default() -> Self {
        Self { game: Game::new() }
    }
}

impl FromStr for ChessGame {
    type Err = ParseError;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            game: Game::new_with_board(Board::from_str(fen)?),
        })
    }
}

impl ChessGame {
    pub fn current_position(&self) -> Position {
        Position {
            board: self.game.current_position(),
        }
    }

    pub fn side_to_move(&self) -> Color {
        self.game.side_to_move()
    }

    pub fn min_evaluation() -> i16 {
        EVALUATOR.min_value()
    }

    pub fn max_evaluation() -> i16 {
        EVALUATOR.max_value()
    }

    pub fn result(&self) -> Option<GameResult> {
        self.game.result()
    }

    pub fn take_action(&mut self, action: Action) {
        match action {
            Action::MakeMove(chess_move) => self.game.make_move(chess_move),
            Action::OfferDraw(color) => self.game.offer_draw(color),
            Action::AcceptDraw => self.game.accept_draw(),
            Action::DeclareDraw => self.game.declare_draw(),
            Action::Resign(color) => self.game.resign(color),
        };
    }
}

impl Position {
    pub fn evaluate(&self) -> i16 {
        EVALUATOR.evaluate(&self.board)
    }

    pub fn get_hash(&self) -> u64 {
        self.board.get_hash()
    }

    pub fn color_on(&self, square: Square) -> Option<Color> {
        self.board.color_on(square)
    }

    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        self.board.piece_on(square)
    }

    pub fn legal(&self, chess_move: ChessMove) -> bool {
        self.board.legal(chess_move)
    }

    pub fn in_check(&self) -> bool {
        self.board.checkers().popcnt() > 0
    }

    pub fn make_move_new(&self, chess_move: ChessMove) -> Self {
        Self {
            board: self.board.make_move_new(chess_move),
        }
    }

    pub fn null_move(&self) -> Option<Self> {
        self.board.null_move().map(|b| Self { board: b })
    }

    pub fn status(&self) -> PositionStatus {
        self.board.status()
    }

    #[inline]
    pub fn legal_moves(&self) -> Vec<ChessMove> {
        MoveGen::new_legal(&self.board).collect::<Vec<ChessMove>>()
    }

    #[inline]
    pub fn sorted_moves(&self, best_move: Option<ChessMove>) -> Vec<ChessMove> {
        let mut sorted_moves = Vec::new();
        let mut move_gen = MoveGen::new_legal(&self.board);
        if let Some(best_move) = best_move {
            if self.board.legal(best_move) {
                move_gen.remove_move(best_move);
                sorted_moves.push(best_move);
            }
        }
        let mut moves = MoveGen::new_legal(&self.board).collect::<Vec<ChessMove>>();
        moves.sort_by(|a, b| compare_moves(&self.board, a, b));
        //moves
        sorted_moves.append(&mut moves);
        sorted_moves
    }

    #[inline]
    pub fn sorted_captures(&self) -> Vec<ChessMove> {
        let mut captures = captures(&self.board).collect::<Vec<ChessMove>>();
        captures.sort_by(|a, b| compare_moves(&self.board, a, b));
        captures
    }
}

#[inline]
fn is_capture(board: &Board, chess_move: &ChessMove) -> bool {
    let square = BitBoard::from_square(chess_move.get_dest());
    (square & board.color_combined(!board.side_to_move())) != chess::EMPTY
}

fn is_promotion(chess_move: &ChessMove) -> bool {
    matches!(chess_move.get_promotion(), Some(_))
}

#[inline]
fn captures(board: &Board) -> MoveGen {
    let mut moves = MoveGen::new_legal(board);
    moves.set_iterator_mask(*board.color_combined(!board.side_to_move()));
    moves
}

#[inline]
fn capture_score(board: &Board, a: &ChessMove) -> i8 {
    let values = [1, 3, 3, 5, 9, 0];
    if is_capture(board, a) {
        values[board.piece_on(a.get_source()).unwrap() as usize]
            - values[board.piece_on(a.get_dest()).unwrap() as usize]
    } else if is_promotion(a) {
        10
    } else {
        i8::MAX
    }
}

#[inline]
fn compare_moves(board: &Board, a: &ChessMove, b: &ChessMove) -> Ordering {
    capture_score(board, a).cmp(&capture_score(board, b))
}
