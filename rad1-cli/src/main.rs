use clap::{App, AppSettings};

mod command;

use command::analyze;
use command::play;

const ANALYZE_COMMAND: &str = "analyze";
const PLAY_COMMAND: &str = "play";

fn main() {
    let analyze_app = analyze::analyze_app(ANALYZE_COMMAND);
    let play_app = play::play_app(PLAY_COMMAND);
    let matches = App::new("Rad1 Chess Engine CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(analyze_app)
        .subcommand(play_app)
        .get_matches();

    if let Some(subcommand) = matches.subcommand_name() {
        if subcommand == ANALYZE_COMMAND {
            analyze::exec(matches.subcommand_matches(ANALYZE_COMMAND).unwrap());
        } else {
            play::exec(matches.subcommand_matches(PLAY_COMMAND).unwrap());
        }
    }
}
