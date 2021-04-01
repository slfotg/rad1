use super::ChessAgent;
use shakmaty::uci::Uci;
use shakmaty::{Chess, Color, Move, Position};
use std::io;

pub struct UciAgent {
    pub color: Color,
}

impl ChessAgent for UciAgent {
    fn take_turn(&mut self, mut position: Chess) -> Chess {
        let chess_move: Move;
        loop {
            println!("Please enter move (UCI notation)");
            let mut uci_move = String::new();
            io::stdin()
                .read_line(&mut uci_move)
                .expect("Failed to read line");
            match Uci::from_ascii(uci_move.trim().as_bytes()) {
                Ok(uci) => match uci.to_move(&position) {
                    Ok(m) => {
                        chess_move = m;
                        break;
                    }
                    Err(_illegal_move) => println!("Illegal Move for current position"),
                },
                Err(_uci_error) => println!("Failed to parse UCI string"),
            }
        }
        position.play_unchecked(&chess_move);
        position
    }
}