use super::Command;
use clap::{App, Arg, ArgMatches, SubCommand};
use shakmaty::fen::Fen;
use shakmaty::*;

use crate::agent;
use crate::agent::ChessAgent;
use crate::game::Game;

const COMMAND_NAME: &str = "analyze";

pub struct AnalyzeCommand {}

impl Default for AnalyzeCommand {
    fn default() -> Self {
        AnalyzeCommand {}
    }
}

impl<'a, 'b> Command<'a, 'b> for AnalyzeCommand {
    fn command_name(&self) -> &'static str {
        COMMAND_NAME
    }

    fn options(&self) -> App<'a, 'b> {
        SubCommand::with_name(COMMAND_NAME)
            .about("Analyze a single position")
            .arg(
                Arg::with_name("fen")
                    .long("fen")
                    .short("f")
                    .required(true)
                    .takes_value(true)
                    .help("The Forsyth-Edwards Notation (FEN) of the position to be analyzed"),
            )
    }

    fn exec_with_depth(&self, depth: usize, matches: &ArgMatches) {
        let fen = matches.value_of("fen").unwrap();
        let setup: Fen = fen.parse().expect("Failed to parse FEN");
        let position: Chess = setup
            .position(CastlingMode::Standard)
            .expect("Failed to setup position from FEN");
        let chess_game = Game::from_position(position);
        analyze_position(&chess_game, depth);
    }
}

fn analyze_position(chess_game: &Game, depth: usize) {
    let mut agent = agent::alpha_beta_agent(depth);
    agent.best_move(chess_game);
}
