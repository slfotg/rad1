use chess;
use chess::{BitBoard, Board, BoardStatus, Color, Piece, Rank, Square};
use rad1::eval::Evaluator;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Clone, Deserialize, Serialize)]
pub struct EvaluationConfig {
    pub opening_weight: i16,
    pub endgame_weight: i16,
    pub opening_values: EvaluationConstants,
    pub endgame_values: EvaluationConstants,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            opening_weight: 0,
            endgame_weight: 0,
            opening_values: EvaluationConstants::default(),
            endgame_values: EvaluationConstants::default(),
        }
    }
}

impl Index<usize> for EvaluationConfig {
    type Output = i16;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.opening_weight,
            1 => &self.endgame_weight,
            i if i < self.opening_values.size() + 2 => &self.opening_values[i - 2],
            i if i < self.endgame_values.size() + 2 + self.opening_values.size() => {
                &self.endgame_values[i - 2 - self.opening_values.size()]
            }
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for EvaluationConfig {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.opening_weight,
            1 => &mut self.endgame_weight,
            i if i < self.opening_values.size() + 2 => &mut self.opening_values[i - 2],
            i if i < self.endgame_values.size() + 2 + self.opening_values.size() => {
                &mut self.endgame_values[i - 2 - self.opening_values.size()]
            }
            _ => panic!("Index out of bounds"),
        }
    }
}

impl EvaluationConfig {
    pub fn size(&self) -> usize {
        2 + self.opening_values.size() + self.endgame_values.size()
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct PieceConstants<T> {
    pawn: T,
    knight: T,
    bishop: T,
    rook: T,
    queen: T,
    king: T,
}

impl<T> Index<Piece> for PieceConstants<T> {
    type Output = T;
    fn index(&self, index: Piece) -> &Self::Output {
        match index {
            Piece::Pawn => &self.pawn,
            Piece::Bishop => &self.knight,
            Piece::Knight => &self.bishop,
            Piece::Rook => &self.rook,
            Piece::Queen => &self.queen,
            Piece::King => &self.king,
        }
    }
}

impl<T> IndexMut<Piece> for PieceConstants<T> {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        match index {
            Piece::Pawn => &mut self.pawn,
            Piece::Bishop => &mut self.knight,
            Piece::Knight => &mut self.bishop,
            Piece::Rook => &mut self.rook,
            Piece::Queen => &mut self.queen,
            Piece::King => &mut self.king,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BoardValues([[i16; 8]; 8]);

impl Default for BoardValues {
    fn default() -> Self {
        Self([[0; 8]; 8])
    }
}

impl Index<Rank> for BoardValues {
    type Output = [i16; 8];
    fn index(&self, index: Rank) -> &Self::Output {
        &self.0[7 - index.to_index()]
    }
}

impl IndexMut<Rank> for BoardValues {
    fn index_mut(&mut self, index: Rank) -> &mut Self::Output {
        &mut self.0[7 - index.to_index()]
    }
}

impl Index<Square> for BoardValues {
    type Output = i16;
    fn index(&self, index: Square) -> &Self::Output {
        &self[index.get_rank()][index.get_file().to_index()]
    }
}

impl IndexMut<Square> for BoardValues {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index.get_rank()][index.get_file().to_index()]
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct EvaluationConstants {
    piece_values: PieceConstants<i16>,
    position_values: PieceConstants<BoardValues>,
}

impl Index<usize> for EvaluationConstants {
    type Output = i16;
    fn index(&self, index: usize) -> &Self::Output {
        let mut i = index;
        if index >= self.size() {
            panic!("Index out of bounds");
        }
        if i < 6 {
            &self.piece_values[chess::ALL_PIECES[i]]
        } else {
            i -= 6;
            let j = i / 64;
            let k = i % 64;
            &self.position_values[chess::ALL_PIECES[j]][chess::ALL_SQUARES[k]]
        }
    }
}

impl IndexMut<usize> for EvaluationConstants {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let mut i = index;
        if index >= self.size() {
            panic!("Index out of bounds");
        }
        if i < 6 {
            &mut self.piece_values[chess::ALL_PIECES[i]]
        } else {
            i -= 6;
            let j = i / 64;
            let k = i % 64;
            &mut self.position_values[chess::ALL_PIECES[j]][chess::ALL_SQUARES[k]]
        }
    }
}

impl EvaluationConstants {
    fn size(&self) -> usize {
        return 6 + 64 * 6;
    }

    #[inline]
    pub fn piece_value(&self, piece: Piece) -> i16 {
        self.piece_values[piece]
    }

    #[inline]
    pub fn position_value(&self, piece: Piece, square: Square) -> i16 {
        self.position_values[piece][square]
    }

    #[inline]
    pub fn piece_score(&self, white_count: usize, black_count: usize, piece: Piece) -> i16 {
        (white_count - black_count) as i16 * self.piece_value(piece)
    }

    #[inline]
    pub fn pawn_score(&self, white_pawns: usize, black_pawns: usize) -> i16 {
        self.piece_score(white_pawns, black_pawns, Piece::Pawn)
    }

    #[inline]
    pub fn knight_score(&self, white_knights: usize, black_knights: usize) -> i16 {
        self.piece_score(white_knights, black_knights, Piece::Knight)
    }

    #[inline]
    pub fn bishop_score(&self, white_bishops: usize, black_bishops: usize) -> i16 {
        self.piece_score(white_bishops, black_bishops, Piece::Bishop)
    }

    #[inline]
    pub fn rook_score(&self, white_rooks: usize, black_rooks: usize) -> i16 {
        self.piece_score(white_rooks, black_rooks, Piece::Rook)
    }

    #[inline]
    pub fn queen_score(&self, white_queens: usize, black_queens: usize) -> i16 {
        self.piece_score(white_queens, black_queens, Piece::Queen)
    }

    #[inline]
    pub fn king_score(&self, white_kings: usize, black_kings: usize) -> i16 {
        self.piece_score(white_kings, black_kings, Piece::King)
    }
}

impl Evaluator<i16> for EvaluationConstants {
    #[inline]
    fn min_value(&self) -> i16 {
        -30000
    }

    #[inline]
    fn max_value(&self) -> i16 {
        30000
    }

    #[inline]
    fn evaluate(&self, board: &Board) -> i16 {
        match board.status() {
            BoardStatus::Stalemate => 0,
            BoardStatus::Checkmate => self.min_value(),
            BoardStatus::Ongoing => {
                let mut evaluation = 0;
                let white_pieces = board.color_combined(Color::White);
                let white_pawns = white_pieces & board.pieces(Piece::Pawn);
                let white_knights = white_pieces & board.pieces(Piece::Knight);
                let white_bishops = white_pieces & board.pieces(Piece::Bishop);
                let white_rooks = white_pieces & board.pieces(Piece::Rook);
                let white_queens = white_pieces & board.pieces(Piece::Queen);
                let white_kings = white_pieces & board.pieces(Piece::King);

                let black_pieces = board.color_combined(Color::Black);
                let black_pawns = (black_pieces & board.pieces(Piece::Pawn)).reverse_colors();
                let black_knights = (black_pieces & board.pieces(Piece::Knight)).reverse_colors();
                let black_bishops = (black_pieces & board.pieces(Piece::Bishop)).reverse_colors();
                let black_rooks = (black_pieces & board.pieces(Piece::Rook)).reverse_colors();
                let black_queens = (black_pieces & board.pieces(Piece::Queen)).reverse_colors();
                let black_kings = (black_pieces & board.pieces(Piece::King)).reverse_colors();

                // Piece Values:
                evaluation += self.pawn_score(white_pawns.count(), black_pawns.count());
                evaluation += self.knight_score(white_knights.count(), black_knights.count());
                evaluation += self.bishop_score(white_bishops.count(), black_bishops.count());
                evaluation += self.rook_score(white_rooks.count(), black_rooks.count());
                evaluation += self.queen_score(white_queens.count(), black_queens.count());
                evaluation += self.king_score(white_kings.count(), black_kings.count());

                // Position Values:
                for square in white_pawns {
                    evaluation += self.position_value(Piece::Pawn, square);
                }
                for square in black_pawns {
                    evaluation -= self.position_value(Piece::Pawn, square);
                }
                for square in white_knights {
                    evaluation += self.position_value(Piece::Knight, square);
                }
                for square in black_knights {
                    evaluation -= self.position_value(Piece::Knight, square);
                }
                for square in white_bishops {
                    evaluation += self.position_value(Piece::Bishop, square);
                }
                for square in black_bishops {
                    evaluation -= self.position_value(Piece::Bishop, square);
                }
                for square in white_rooks {
                    evaluation += self.position_value(Piece::Rook, square);
                }
                for square in black_rooks {
                    evaluation -= self.position_value(Piece::Rook, square);
                }
                for square in white_queens {
                    evaluation += self.position_value(Piece::Queen, square);
                }
                for square in black_queens {
                    evaluation -= self.position_value(Piece::Queen, square);
                }
                for square in white_kings {
                    evaluation += self.position_value(Piece::King, square);
                }
                for square in black_kings {
                    evaluation -= self.position_value(Piece::King, square);
                }

                if board.side_to_move() == Color::White {
                    evaluation
                } else {
                    -evaluation
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EvaluationConstants;
    use chess;
    use chess::{BitBoard, Color, Square};

    #[test]
    fn test_index_mut() {
        let mut constants = EvaluationConstants::default();
        for i in 0..constants.size() {
            constants[i] = i as i16;
        }
        for i in 0..constants.size() {
            assert_eq!(constants[i], i as i16);
        }
    }

    #[test]
    fn test_get_rook_moves() {
        let blockers = BitBoard::from_square(Square::A2);
        let moves = chess::get_rook_moves(Square::A1, blockers);
        for square in moves {
            println!("{}", square);
        }
        assert_eq!(moves.count(), 8);
    }

    #[test]
    fn test_get_rook_rays() {
        let rays = chess::get_bishop_rays(Square::C3);
        for square in rays {
            println!("{}", square);
        }
        assert_eq!(rays.count(), 11);
    }

    #[test]
    fn test_get_pawn_moves() {
        let square = Square::E2;
        let blockers = BitBoard::from_square(Square::D3) | BitBoard::from_square(Square::F3);
        let moves = chess::get_pawn_attacks(square, Color::White, blockers);

        for square in moves {
            println!("{}", square);
        }
        assert_eq!(moves.count(), 2);
    }
}
