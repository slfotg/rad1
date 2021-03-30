use rand;
use shakmaty::{Chess, Color, Position};

pub trait ChessAgent {
    fn take_turn(&mut self, position: Chess) -> Chess;
}

pub fn random_chess_agent(color: Color) -> impl ChessAgent {
    random::RandomChessAgent {
        color,
        rng: rand::thread_rng(),
    }
}

pub fn command_line_agent(color: Color) -> impl ChessAgent {
    uci::UciAgent {
        color,
    }
}

fn check_side_to_move(color: &Color, position: &impl Position) {
    if position.turn() != *color {
        panic!("Wrong color's turn to move");
    }
}

mod random {
    use rand::rngs::ThreadRng;
    use rand::Rng;
    use shakmaty::{Chess, Color, Position};
    use super::ChessAgent;

    pub struct RandomChessAgent {
        pub color: Color,
        pub rng: ThreadRng,
    }

    impl ChessAgent for RandomChessAgent {
        fn take_turn(&mut self, mut position: Chess) -> Chess {
            super::check_side_to_move(&self.color, &position);
            let moves = position.legal_moves();
            position.play_unchecked(&moves[self.rng.gen_range(0..moves.len())].clone());
            position
        }
    }
}

mod uci {
    use std::io;
    use shakmaty::{Chess, Color, Move, Position};
    use shakmaty::uci::Uci;
    use super::ChessAgent;

    pub struct UciAgent {
        pub color: Color,
    }

    impl ChessAgent for UciAgent {
        fn take_turn(&mut self, mut position: Chess) -> Chess {
            let chess_move: Move;
            loop {
                println!("Please enter move (UCI notation)");
                let mut uci_move = String::new();
                io::stdin().read_line(&mut uci_move).expect("Failed to read line");
                match Uci::from_ascii(uci_move.trim().as_bytes()) {
                    Ok(uci) => match uci.to_move(&position) {
                        Ok(m) => {
                            chess_move = m;
                            break;
                        },
                        Err(_illegal_move) => println!("Illegal Move for current position"),
                    },
                    Err(_uci_error) => println!("Failed to parse UCI string"),
                }
            }
            position.play_unchecked(&chess_move);
            position
        }
    }
}