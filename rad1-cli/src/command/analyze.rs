use super::Command;
use chess::Game;
use clap::{App, Arg, ArgMatches, SubCommand};
use rad1::agent;
use rad1::agent::ChessAgent;
use std::str::FromStr;

const COMMAND_NAME: &str = "analyze";

#[derive(Default)]
pub struct AnalyzeCommand;

impl<'a, 'b> Command<'a, 'b> for AnalyzeCommand {
    fn command_name(&self) -> &'static str {
        COMMAND_NAME
    }

    fn options(&self) -> App<'a, 'b> {
        SubCommand::with_name(COMMAND_NAME)
            .about("Analyze a single position")
            .arg(
                Arg::with_name("depth")
                    .long("depth")
                    .short("d")
                    .required(false)
                    .takes_value(true)
                    .default_value("8")
                    .possible_values(&["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
                    .hide_possible_values(true)
                    .help(
                        "The depth of the search tree. Higher values means better move selections.",
                    ),
            )
            .arg(
                Arg::with_name("fen")
                    .long("fen")
                    .short("f")
                    .required(true)
                    .takes_value(true)
                    .help("The Forsyth-Edwards Notation (FEN) of the position to be analyzed"),
            )
    }

    fn exec(&self, matches: &ArgMatches) {
        let fen = matches.value_of("fen").unwrap();
        let game = Game::from_str(fen).expect("Failed to parse FEN");
        let depth: u8 = matches.value_of("depth").unwrap().parse().unwrap();
        analyze_position(&game, depth);
    }
}

fn analyze_position(game: &Game, depth: u8) {
    let agent = agent::alpha_beta_agent(depth);
    agent.get_action(game);
}
