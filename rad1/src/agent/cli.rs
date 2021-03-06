use super::ChessAgent;
use crate::Action;
use crate::ChessGame;
use crate::ChessMove;
use core::str::FromStr;
use std::io;

#[derive(Default)]
pub struct CommandLineAgent;

impl ChessAgent for CommandLineAgent {
    fn get_action(&self, game: &ChessGame) -> Action {
        let action: Action;
        let board = game.current_position();
        loop {
            println!("Please enter move (Long algebraic notation) or enter 'resign' to resign");
            println!("Examples:  e2e4, e7e5, e1g1 (white short castling), e7e8q (for promotion)");
            let mut uci_move = String::new();
            io::stdin()
                .read_line(&mut uci_move)
                .expect("Failed to read line");
            let uci_move = uci_move.trim();
            match uci_move {
                "resign" => {
                    action = Action::Resign(game.side_to_move());
                    break;
                }
                uci_move => match ChessMove::from_str(uci_move) {
                    Ok(uci) => {
                        if board.legal(uci) {
                            action = Action::MakeMove(uci);
                            break;
                        } else {
                            println!("Illegal Move for current position");
                        }
                    }
                    Err(_) => println!("Failed to parse move format"),
                },
            }
        }
        action
    }
}
