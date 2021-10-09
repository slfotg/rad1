use chess::{ChessMove, Square, ALL_SQUARES, PROMOTION_PIECES};
use lazy_static::lazy_static;

lazy_static! {
    static ref MOVE_HASH: [ChessMove; HASH_SIZE] = move_hash();
}

// Size of hash
// 64 possible source squares
// 64 possible destination squares
// 5 possible promotion pieces [None, Queen, Knight, Rook, Bishop]
const HASH_SIZE: usize = 64 * 64 * 5; // 20480

fn move_hash() -> [ChessMove; HASH_SIZE] {
    let default_move = ChessMove::new(Square::A1, Square::A1, None);
    let mut all_moves: [ChessMove; HASH_SIZE] = [default_move; HASH_SIZE];

    let mut index = 0;
    for source in ALL_SQUARES {
        for dest in ALL_SQUARES {
            all_moves[index] = ChessMove::new(source, dest, None);
            index += 1;
            for promotion in PROMOTION_PIECES {
                all_moves[index] = ChessMove::new(source, dest, Some(promotion));
                index += 1;
            }
        }
    }

    // just double check the math here
    assert_eq!(index, HASH_SIZE);

    all_moves
}

pub fn get_hash(chess_move: ChessMove) -> u16 {
    let source_index = chess_move.get_source().to_index();
    let dest_index = chess_move.get_dest().to_index();
    let promotion_piece_index = if let Some(promotion_piece) = chess_move.get_promotion() {
        if promotion_piece == PROMOTION_PIECES[0] {
            1
        } else if promotion_piece == PROMOTION_PIECES[1] {
            2
        } else if promotion_piece == PROMOTION_PIECES[2] {
            3
        } else if promotion_piece == PROMOTION_PIECES[3] {
            4
        } else {
            panic!("Invalid promotion piece calculating hash")
        }
    } else {
        0
    };
    (source_index * 64 * 5 + dest_index * 5 + promotion_piece_index) as u16
}

pub fn get_move(hash: u16) -> ChessMove {
    MOVE_HASH[hash as usize]
}

#[cfg(test)]
mod tests {
    #[test]
    fn hash_test() {
        for i in 0..(super::HASH_SIZE) {
            let chess_move = super::get_move(i as u16);
            assert_eq!(super::get_hash(chess_move), i as u16);
        }
    }
}
