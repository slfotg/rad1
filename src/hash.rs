use shakmaty::*;
use std::num::Wrapping;

#[derive(Debug, Clone, Copy)]
struct RandomNumberGenerator {
    a: Wrapping<i64>,
    b: Wrapping<i64>,
    c: Wrapping<i64>,
    d: Wrapping<i64>,
}

fn rot(x: i64, k: u8) -> i64 {
    (x << k) | (x >> (64 - k))
}

impl RandomNumberGenerator {
    fn new(seed: i64) -> RandomNumberGenerator {
        let mut rng = RandomNumberGenerator {
            a: Wrapping(0xf1ea5eed),
            b: Wrapping(seed),
            c: Wrapping(seed),
            d: Wrapping(seed),
        };
        for _ in 0..20 {
            rng.next_value();
        }

        rng
    }

    fn next_value(&mut self) -> i64 {
        let e = self.a - Wrapping(rot(self.b.0, 7));
        self.a = self.b ^ Wrapping(rot(self.c.0, 13));
        self.b = self.c + Wrapping(rot(self.d.0, 37));
        self.c = self.d + e;
        self.d = e + self.a;
        self.d.0
    }
}

pub struct ChessHasher {
    random_numbers: [i64; 781],
    corners: [Bitboard; 4],
}

impl ChessHasher {
    pub fn new() -> ChessHasher {
        let mut rng = RandomNumberGenerator::new(42);
        let mut random_numbers = [0; 781];
        for i in 0..781 {
            random_numbers[i] = rng.next_value();
        }
        let corners: [Bitboard; 4] = [
            Bitboard::from_square(Square::A1),
            Bitboard::from_square(Square::A8),
            Bitboard::from_square(Square::H1),
            Bitboard::from_square(Square::H8),
        ];
        ChessHasher {
            random_numbers,
            corners,
        }
    }

    fn get_piece_hash(&self, square: Square, piece: Piece) -> i64 {
        // 64 squares
        let s = usize::from(square);
        // 6 roles
        let r = usize::from(piece.role) - 1;
        // 2 colors
        let c = if piece.color == Color::Black { 0 } else { 1 };

        let index = c * 384 + r * 64 + s;
        self.random_numbers[index]
    }

    // Color to move [768]
    fn get_color_hash(&self, color: Color) -> i64 {
        if color == Color::Black {
            self.random_numbers[768]
        } else {
            0
        }
    }

    // Castling Rights [769, 770, 771, 772]
    fn get_castling_hash(&self, rights: Bitboard) -> i64 {
        let mut hash = 0;
        for (i, corner) in self.corners.iter().enumerate() {
            if rights & *corner == *corner {
                hash ^= self.random_numbers[i + 769];
            }
        }
        hash
    }

    // En passant file [773, 774, 775, 776, 777, 778, 779, 780]
    fn get_en_passant_hash(&self, ep_square: Option<Square>) -> i64 {
        match ep_square {
            Option::None => 0,
            Option::Some(square) => self.random_numbers[usize::from(square.file()) + 773],
        }
    }

    pub fn hash(&self, game: &Chess) -> i64 {
        let mut hash = 0;
        for (square, piece) in game.board().pieces() {
            hash ^= self.get_piece_hash(square, piece);
        }

        hash ^= self.get_color_hash(game.turn());
        hash ^= self.get_castling_hash(game.castling_rights());
        hash ^ self.get_en_passant_hash(game.ep_square())
    }

    pub fn update_hash(
        &self,
        original_hash: i64,
        original_game: &Chess,
        new_game: &Chess,
        chess_move: &Move,
    ) -> i64 {
        let mut new_hash = original_hash;
        let color = original_game.turn();
        let opp_color = new_game.turn();
        // update color
        new_hash ^= self.random_numbers[768];
        // update castling rights
        new_hash ^= self.get_castling_hash(original_game.castling_rights());
        new_hash ^= self.get_castling_hash(new_game.castling_rights());
        // update en passant
        new_hash ^= self.get_en_passant_hash(original_game.ep_square());
        new_hash ^= self.get_en_passant_hash(new_game.ep_square());
        // update piece positions
        match *chess_move {
            Move::Normal {
                role,
                from,
                capture,
                to,
                promotion,
            } => {
                new_hash ^= self.get_piece_hash(from, Piece { color, role });
                if let Some(role) = promotion {
                    new_hash ^= self.get_piece_hash(to, Piece { color, role });
                } else {
                    new_hash ^= self.get_piece_hash(to, Piece { color, role });
                }
                if let Some(capture) = capture {
                    new_hash ^= self.get_piece_hash(
                        to,
                        Piece {
                            color: opp_color,
                            role: capture,
                        },
                    );
                }
            }
            Move::EnPassant { from, to } => {
                new_hash ^= self.get_piece_hash(
                    from,
                    Piece {
                        color,
                        role: Role::Pawn,
                    },
                );
                new_hash ^= self.get_piece_hash(
                    to,
                    Piece {
                        color,
                        role: Role::Pawn,
                    },
                );
                new_hash ^= self.get_piece_hash(
                    Square::from_coords(to.file(), from.rank()),
                    Piece {
                        color: opp_color,
                        role: Role::Pawn,
                    },
                );
            }
            Move::Castle { king, rook } => {
                new_hash ^= self.get_piece_hash(
                    king,
                    Piece {
                        color,
                        role: Role::King,
                    },
                );
                new_hash ^= self.get_piece_hash(
                    rook,
                    Piece {
                        color,
                        role: Role::Rook,
                    },
                );
                let rank = king.rank();
                match rook.file() {
                    File::A => {
                        new_hash ^= self.get_piece_hash(
                            Square::from_coords(File::C, rank),
                            Piece {
                                color,
                                role: Role::King,
                            },
                        );
                        new_hash ^= self.get_piece_hash(
                            Square::from_coords(File::D, rank),
                            Piece {
                                color,
                                role: Role::Rook,
                            },
                        );
                    }
                    File::H => {
                        new_hash ^= self.get_piece_hash(
                            Square::from_coords(File::G, rank),
                            Piece {
                                color,
                                role: Role::King,
                            },
                        );
                        new_hash ^= self.get_piece_hash(
                            Square::from_coords(File::F, rank),
                            Piece {
                                color,
                                role: Role::Rook,
                            },
                        );
                    }
                    _ => {
                        panic!("Illegal castle")
                    }
                }
            }
            Move::Put { role, to } => {
                new_hash ^= self.get_piece_hash(to, Piece { color, role });
            }
        }

        new_hash
    }
}

#[cfg(test)]
mod tests {
    use super::ChessHasher;
    use shakmaty::fen::Fen;
    use shakmaty::uci::Uci;
    use shakmaty::{CastlingMode, Chess, Position};
    use test_case::test_case;
    #[test]
    fn test_update_hash_from_start() {
        let position = Chess::default();
        let hasher = ChessHasher::new();
        let hash = hasher.hash(&position);
        for chess_move in position.legal_moves() {
            let mut next_position = position.clone();
            next_position.play_unchecked(&chess_move);
            let expected_hash = hasher.hash(&next_position);
            let actual_hash = hasher.update_hash(hash, &position, &next_position, &chess_move);
            println!("{}", expected_hash);
            assert_eq!(expected_hash, actual_hash);
        }
    }

    #[test_case("r1bqkb1r/ppp1pppp/2nP1n2/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1", "d6c7"; "capture")]
    #[test_case("r1bqkbnr/ppp1pppp/2n5/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1", "e5d6"; "en passant")]
    #[test_case("r1bqkb1r/ppP2ppp/2n2n2/4p3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1", "c7d8q"; "capture and promotion")]
    #[test_case("r3k2r/ppq2pbp/2n2np1/1B2pb2/3P4/1PN2NP1/PBP1QP1P/R3K2R w KQkq - 0 1", "e1g1"; "white king side castle")]
    #[test_case("r3k2r/ppq2pbp/2n2np1/1B2pb2/3P4/1PN2NP1/PBP1QP1P/R3K2R w KQkq - 0 1", "e1c1"; "white queen side castle")]
    #[test_case("r3k2r/ppq2pbp/2n2np1/1B2pb2/3P4/1PN2NP1/PBP1QP1P/R3K2R b KQkq - 0 1", "e8g8"; "black king side castle")]
    #[test_case("r3k2r/ppq2pbp/2n2np1/1B2pb2/3P4/1PN2NP1/PBP1QP1P/R3K2R b KQkq - 0 1", "e8c8"; "black queen side castle")]
    fn test_update_hash(fen: &str, uci: &str) {
        let setup: Fen = fen.parse().unwrap();

        let position: Chess = setup.position(CastlingMode::Standard).unwrap();

        let chess_move = Uci::from_ascii(uci.as_bytes())
            .unwrap()
            .to_move(&position)
            .unwrap();
        let mut next_position = position.clone();
        next_position.play_unchecked(&chess_move);

        let hasher = ChessHasher::new();
        let hash = hasher.hash(&position);
        let expected_hash = hasher.hash(&next_position);
        let actual_hash = hasher.update_hash(hash, &position, &next_position, &chess_move);
        println!("{}", expected_hash);
        assert_eq!(expected_hash, actual_hash);
    }
}
