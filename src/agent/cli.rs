use super::ChessAgent;
use shakmaty::uci::Uci;
use shakmaty::Move;
use std::io;

use crate::game::Game;

pub struct CommandLineAgent {}

impl Default for CommandLineAgent {
    fn default() -> Self {
        Self {}
    }
}

impl ChessAgent for CommandLineAgent {
    fn best_move(&mut self, game: &Game) -> Move {
        let chess_move: Move;
        loop {
            println!("Please enter move (Long algebraic notation)");
            println!("Examples:  e2e4, e7e5, e1g1 (white short castling), e7e8q (for promotion)");
            let mut uci_move = String::new();
            io::stdin()
                .read_line(&mut uci_move)
                .expect("Failed to read line");
            match Uci::from_ascii(uci_move.trim().as_bytes()) {
                Ok(uci) => match uci.to_move(&game.position) {
                    Ok(m) => {
                        chess_move = m;
                        break;
                    }
                    Err(_illegal_move) => println!("Illegal Move for current position"),
                },
                Err(_uci_error) => println!("Failed to parse move format"),
            }
        }
        chess_move
    }
}
