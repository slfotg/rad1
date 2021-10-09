use clap::{App, AppSettings};

mod command;

use command::analyze;
use command::play;

fn main() {
    let analyze_app = analyze::analyze_app();
    let play_app = play::play_app();
    let matches = App::new("Rad1 Chess Engine CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(analyze_app)
        .subcommand(play_app)
        .get_matches();

    if let Some(subcommand) = matches.subcommand_name() {
        if subcommand == "analyze" {
            analyze::exec(matches.subcommand_matches("analyze").unwrap());
        } else {
            play::exec(matches.subcommand_matches("play").unwrap());
        }
    }
}
