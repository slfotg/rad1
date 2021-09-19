use super::ChessAgent;
use crate::game::Game;
use chess::ChessMove;
use core::str::FromStr;
use std::io;

pub struct CommandLineAgent {}

impl Default for CommandLineAgent {
    fn default() -> Self {
        Self {}
    }
}

impl ChessAgent for CommandLineAgent {
    fn best_move(&mut self, game: &Game) -> ChessMove {
        let chess_move: ChessMove;
        loop {
            println!("Please enter move (Long algebraic notation)");
            println!("Examples:  e2e4, e7e5, e1g1 (white short castling), e7e8q (for promotion)");
            let mut uci_move = String::new();
            io::stdin()
                .read_line(&mut uci_move)
                .expect("Failed to read line");
            let uci_move = uci_move.trim();
            match ChessMove::from_str(uci_move) {
                Ok(uci) => {
                    if game.is_legal(uci) {
                        chess_move = uci;
                        break;
                    } else {
                        println!("Illegal Move for current position");
                    }
                },
                Err(_) => println!("Failed to parse move format"),
            }
        }
        chess_move
    }
}
