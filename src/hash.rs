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
}
