
use clap::{App, ArgMatches};

mod analyze;
mod play;

pub trait Command<'a, 'b> {

    fn command_name(&self) -> &'static str;

    fn options(&self) -> App<'a, 'b>;

    fn exec_with_depth(&self, depth: usize, matches: &ArgMatches);
}

pub fn analyze() -> analyze::AnalyzeCommand {
    analyze::AnalyzeCommand::default()
}

pub fn play() -> play::PlayCommand {
    play::PlayCommand::default()
}