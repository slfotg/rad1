use clap::{App, AppSettings, Arg};
use rad1::command;
use rad1::command::Command;

fn main() {
    let analyze_command = command::analyze();
    let play_command = command::play();
    let matches = App::new("Rad1 Chess Engine")
        .setting(AppSettings::SubcommandRequired)
        .version("0.1.0")
        .author("Sam Foster <slfotg@gmail.com>")
        .about("A Simple Chess Engine in Rust")
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
        .subcommand(play_command.options())
        .subcommand(analyze_command.options())
        .get_matches();

    let depth: usize = matches.value_of("depth").unwrap().parse().unwrap();

    if let Some(subcommand) = matches.subcommand_name() {
        if subcommand == analyze_command.command_name() {
            analyze_command.exec_with_depth(
                depth,
                matches.subcommand_matches(analyze_command.command_name()).unwrap(),
            );
        } else {
            play_command.exec_with_depth(
                depth,
                matches.subcommand_matches(play_command.command_name()).unwrap(),
            );
        }
    }
}
