use chess::{BitBoard, Color, File, Piece, Rank, Square};
use lazy_static::lazy_static;
use std::num::Wrapping;

lazy_static! {
    pub static ref CHESS_HASHER: ChessHasher = ChessHasher::default();
}

#[derive(Debug, Clone, Copy)]
struct RandomNumberGenerator {
    a: Wrapping<u64>,
    b: Wrapping<u64>,
    c: Wrapping<u64>,
    d: Wrapping<u64>,
}

fn rot(x: u64, k: u8) -> u64 {
    (x << k) | (x >> (64 - k))
}

impl RandomNumberGenerator {
    fn new(seed: u64) -> RandomNumberGenerator {
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

    fn next_value(&mut self) -> u64 {
        let e = self.a - Wrapping(rot(self.b.0, 7));
        self.a = self.b ^ Wrapping(rot(self.c.0, 13));
        self.b = self.c + Wrapping(rot(self.d.0, 37));
        self.c = self.d + e;
        self.d = e + self.a;
        self.d.0
    }
}

pub struct ChessHasher {
    random_numbers: [u64; 781],
    corners: [BitBoard; 4],
}

impl ChessHasher {
    pub fn default() -> Self {
        let mut rng = RandomNumberGenerator::new(101);
        let mut random_numbers = [0; 781];
        for num in &mut random_numbers {
            *num = rng.next_value();
        }
        let corners: [BitBoard; 4] = [
            BitBoard::from_square(Square::make_square(Rank::First, File::A)),
            BitBoard::from_square(Square::make_square(Rank::Eighth, File::A)),
            BitBoard::from_square(Square::make_square(Rank::First, File::H)),
            BitBoard::from_square(Square::make_square(Rank::Eighth, File::H)),
        ];
        Self {
            random_numbers,
            corners,
        }
    }

    fn get_piece_hash(&self, square: Square, piece: Piece, color: Color) -> u64 {
        // 64 squares
        let s = square.to_index();
        // 6 pieces
        let r = piece.to_index();
        // 2 colors
        let c = color.to_index();

        let index = c * 384 + r * 64 + s;
        self.random_numbers[index]
    }

    // Color to move [768]
    fn get_color_hash(&self, color: Color) -> u64 {
        if color == Color::Black {
            self.random_numbers[768]
        } else {
            0
        }
    }

    // Castling Rights [769, 770, 771, 772]
    fn get_castling_hash(&self, rights: BitBoard) -> u64 {
        let mut hash = 0;
        for (i, corner) in self.corners.iter().enumerate() {
            if rights & *corner == *corner {
                hash ^= self.random_numbers[i + 769];
            }
        }
        hash
    }

    // En passant file [773, 774, 775, 776, 777, 778, 779, 780]
    fn get_en_passant_hash(&self, ep_square: Option<Square>) -> u64 {
        match ep_square {
            Option::None => 0,
            Option::Some(square) => self.random_numbers[usize::from(square.file()) + 773],
        }
    }

    pub fn hash(&self, game: &Chess) -> u64 {
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
        original_hash: u64,
        original_game: &Chess,
        new_game: &Chess,
        chess_move: &Move,
    ) -> u64 {
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
    use super::CHESS_HASHER;
    use shakmaty::fen::Fen;
    use shakmaty::uci::Uci;
    use shakmaty::{CastlingMode, Chess, Position};
    use test_case::test_case;
    #[test]
    fn test_update_hash_from_start() {
        let position = Chess::default();
        let hash = CHESS_HASHER.hash(&position);
        for chess_move in position.legal_moves() {
            let mut next_position = position.clone();
            next_position.play_unchecked(&chess_move);
            let expected_hash = CHESS_HASHER.hash(&next_position);
            let actual_hash =
                CHESS_HASHER.update_hash(hash, &position, &next_position, &chess_move);
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

        let hash = CHESS_HASHER.hash(&position);
        let expected_hash = CHESS_HASHER.hash(&next_position);
        let actual_hash = CHESS_HASHER.update_hash(hash, &position, &next_position, &chess_move);
        println!("{}", expected_hash);
        assert_eq!(expected_hash, actual_hash);
    }
}
