use super::Command;
use crate::agent;
use crate::agent::ChessAgent;
use ansi_term::Colour;
use ansi_term::Style;
use chess::{Board, BoardStatus, Color, Piece, Rank, Square};
use clap::{App, Arg, ArgMatches, SubCommand};
use itertools::Either;
use std::str::FromStr;

const COMMAND_NAME: &str = "play";

pub struct PlayCommand {}

impl Default for PlayCommand {
    fn default() -> Self {
        PlayCommand {}
    }
}

impl<'a, 'b> Command<'a, 'b> for PlayCommand {
    fn command_name(&self) -> &'static str {
        COMMAND_NAME
    }

    fn options(&self) -> App<'a, 'b> {
        SubCommand::with_name(COMMAND_NAME)
            .about("Play against the chess engine from terminal")
            .arg(
                Arg::with_name("start-position")
                    .long("from")
                    .short("f")
                    .required(false)
                    .takes_value(true)
                    .default_value("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .hide_default_value(true)
                    .help("The Forsyth-Edwards Notation (FEN) of the position to be from"),
            )
            .arg(
                Arg::with_name("color")
                    .long("color")
                    .short("c")
                    .required(false)
                    .default_value("White")
                    .possible_values(&["White", "Black"])
                    .help("The color you want to play as"),
            )
    }

    fn exec_with_depth(&self, depth: usize, matches: &ArgMatches) {
        let start_position = matches.value_of("start-position").unwrap();
        let mut board = Board::from_str(start_position).expect("Failed to parse FEN");
        let color = matches.value_of("color").unwrap();

        if color == "White" {
            let mut white_player = agent::command_line_agent();
            let mut black_player = agent::alpha_beta_agent(depth);
            play_game(&mut board, &mut white_player, &mut black_player, false);
        } else {
            let mut white_player = agent::alpha_beta_agent(depth);
            let mut black_player = agent::command_line_agent();
            play_game(&mut board, &mut white_player, &mut black_player, true);
        }
    }
}

fn play_game(
    board: &mut Board,
    white_player: &mut dyn ChessAgent,
    black_player: &mut dyn ChessAgent,
    reverse_board: bool,
) {
    let mut current_player = board.side_to_move();
    print_board(board, reverse_board);
    while board.status() == BoardStatus::Ongoing {
        let best_move = match current_player {
            Color::White => white_player.best_move(&board),
            Color::Black => black_player.best_move(&board),
        };
        *board = board.make_move_new(best_move);
        current_player = !current_player;
        print_board(board, reverse_board);
    }
    println!("{:?}", board.status());
}

fn print_board(board: &Board, reverse_board: bool) {
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support();

    let italic: Style = Style::new().italic();
    let fg_black: Colour = Colour::Fixed(16);
    let bg_black: Style = fg_black.on(Colour::Fixed(34));
    let bg_white: Style = fg_black.on(Colour::Fixed(220));
    let range = if reverse_board {
        Either::Left(chess::ALL_RANKS.iter())
    } else {
        Either::Right(chess::ALL_RANKS.iter().rev())
    };
    for rank in range {
        print_rank(rank, italic, bg_black, bg_white, board);
    }
    println!("{}", italic.paint("    A  B  C  D  E  F  G  H"));
}

fn print_rank(rank: &Rank, italic: Style, bg_black: Style, bg_white: Style, board: &Board) {
    let mut line: String = String::new();
    let mut background = if rank.to_index() % 2 == 1 { bg_white } else { bg_black };
    line.push_str(&italic.paint(format!(" {} ", rank.to_index() + 1)).to_string());
    for file in chess::ALL_FILES.iter() {
        let square = Square::make_square(*rank, *file);
        let piece_char = get_piece_char(board.color_on(square), board.piece_on(square));
        line.push_str(&background.paint(format!(" {} ", piece_char)).to_string());
        background = if background == bg_white {
            bg_black
        } else {
            bg_white
        };
    }
    println!("{}", line);
}

fn get_piece_char(color: Option<Color>, piece: Option<Piece>) -> &'static str {
    match (color, piece) {
        (Some(Color::White), Some(Piece::Pawn)) => "♙",
        (Some(Color::White), Some(Piece::Knight)) => "♘",
        (Some(Color::White), Some(Piece::Bishop)) => "♗",
        (Some(Color::White), Some(Piece::Rook)) => "♖",
        (Some(Color::White), Some(Piece::Queen)) => "♕",
        (Some(Color::White), Some(Piece::King)) => "♔",
        (Some(Color::Black), Some(Piece::Pawn)) => "♟︎",
        (Some(Color::Black), Some(Piece::Knight)) => "♞",
        (Some(Color::Black), Some(Piece::Bishop)) => "♝",
        (Some(Color::Black), Some(Piece::Rook)) => "♜",
        (Some(Color::Black), Some(Piece::Queen)) => "♛",
        (Some(Color::Black), Some(Piece::King)) => "♚",
        (_, _) => " ",
    }
}
