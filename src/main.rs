use clap::{App, AppSettings};
use rad1::command;
use rad1::command::Command;

fn main() {
    let analyze_command = command::analyze();
    let play_command = command::play();
    let matches = App::new("Rad1 Chess Engine")
        .setting(AppSettings::SubcommandRequired)
        .version("0.2.0")
        .author("Sam Foster <slfotg@gmail.com>")
        .about("A Simple Chess Engine in Rust")
        .subcommand(play_command.options())
        .subcommand(analyze_command.options())
        .get_matches();

    if let Some(subcommand) = matches.subcommand_name() {
        if subcommand == analyze_command.command_name() {
            analyze_command.exec(
                matches
                    .subcommand_matches(analyze_command.command_name())
                    .unwrap(),
            );
        } else {
            play_command.exec(
                matches
                    .subcommand_matches(play_command.command_name())
                    .unwrap(),
            );
        }
    }
}
