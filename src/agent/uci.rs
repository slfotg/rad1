use super::ChessAgent;
use shakmaty::uci::Uci;
use shakmaty::{Color, Move};
use std::io;

use crate::game::Game;

pub struct UciAgent {
    color: Color,
}

impl UciAgent {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl ChessAgent for UciAgent {
    fn take_turn(&self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
        let chess_move: Move;
        loop {
            println!("Please enter move (UCI notation)");
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
                Err(_uci_error) => println!("Failed to parse UCI string"),
            }
        }
        game.play(&chess_move)
    }
}
