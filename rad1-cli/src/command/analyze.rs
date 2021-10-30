use chess::Game;
use clap::{App, Arg, ArgMatches};
use rad1::agent;
use rad1::agent::ChessAgent;
use rad1::tt::TranspositionTable;
use rad1::eval;
use std::str::FromStr;

pub fn analyze_app(command_name: &str) -> App<'static, 'static> {
    App::new(command_name)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Analyze a single position with Rad1 chess engine")
        .arg(
            Arg::with_name("depth")
                .long("depth")
                .short("d")
                .required(false)
                .takes_value(true)
                .default_value("8")
                .possible_values(&["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
                .hide_possible_values(true)
                .help("The depth of the search tree. Higher values means better move selections."),
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

pub fn exec(matches: &ArgMatches) {
    let fen = matches.value_of("fen").unwrap();
    let game = Game::from_str(fen).expect("Failed to parse FEN");
    let depth: u8 = matches.value_of("depth").unwrap().parse().unwrap();
    analyze_position(&game, depth);
}

fn analyze_position(game: &Game, depth: u8) {
    let agent = agent::alpha_beta_agent(depth, TranspositionTable::default(), Box::new(eval::naive_evaluator()));
    agent.get_action(game);
}
