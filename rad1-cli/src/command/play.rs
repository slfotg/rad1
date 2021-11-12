use ansi_term::{Colour, Style};
use chess::{Action, Board, Color, Game, Piece, Rank, Square};
use clap::{App, Arg, ArgMatches};
use itertools::Either;
use rad1::agent;
use rad1::agent::ChessAgent;
use rad1::eval;
use rad1::tt::TranspositionTable;
use std::str::FromStr;
use std::sync::Arc;

pub fn play_app(command_name: &str) -> App<'static, 'static> {
    App::new(command_name)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Play against the chess engine from terminal")
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

pub fn exec(matches: &ArgMatches) {
    let start_position = matches.value_of("start-position").unwrap();
    let mut game = Game::from_str(start_position).expect("Failed to parse FEN");
    let color = matches.value_of("color").unwrap();
    let depth: u8 = matches.value_of("depth").unwrap().parse().unwrap();

    if color == "White" {
        let white_player = agent::command_line_agent();
        let black_player = agent::alpha_beta_agent(
            depth,
            TranspositionTable::default(),
            Arc::new(eval::naive_evaluator()),
        );
        play_game(&mut game, &white_player, &black_player, false);
    } else {
        let white_player = agent::alpha_beta_agent(
            depth,
            TranspositionTable::default(),
            Arc::new(eval::naive_evaluator()),
        );
        let black_player = agent::command_line_agent();
        play_game(&mut game, &white_player, &black_player, true);
    }
}

fn play_game(
    game: &mut Game,
    white_player: &dyn ChessAgent,
    black_player: &dyn ChessAgent,
    reverse_board: bool,
) {
    print_board(&game.current_position(), reverse_board);
    while game.result() == None {
        let action = match game.current_position().side_to_move() {
            Color::White => white_player.get_action(game),
            Color::Black => black_player.get_action(game),
        };
        match action {
            Action::MakeMove(chess_move) => game.make_move(chess_move),
            Action::OfferDraw(color) => game.offer_draw(color),
            Action::AcceptDraw => game.accept_draw(),
            Action::DeclareDraw => game.declare_draw(),
            Action::Resign(color) => game.resign(color),
        };
        print_board(&game.current_position(), reverse_board);
    }
    println!("{:?}", game.result().unwrap());
}

fn print_board(board: &Board, reverse_board: bool) {
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support().expect("ANSI colors not supported");

    let italic: Style = Style::new().italic();
    let fg_black: Colour = Colour::Fixed(16);
    let bg_black: Style = fg_black.on(Colour::Fixed(34));
    let bg_white: Style = fg_black.on(Colour::Fixed(220));
    let ranks = if reverse_board {
        Either::Left(chess::ALL_RANKS.iter())
    } else {
        Either::Right(chess::ALL_RANKS.iter().rev())
    };
    for rank in ranks {
        print_rank(rank, italic, bg_black, bg_white, board, reverse_board);
    }
    if reverse_board {
        println!("{}", italic.paint("    H  G  F  E  D  C  B  A"));
    } else {
        println!("{}", italic.paint("    A  B  C  D  E  F  G  H"));
    }
}

fn print_rank(
    rank: &Rank,
    italic: Style,
    bg_black: Style,
    bg_white: Style,
    board: &Board,
    reverse_board: bool,
) {
    let mut line: String = String::new();
    let mut background = if rank.to_index() % 2 == 1 {
        bg_white
    } else {
        bg_black
    };
    line.push_str(
        &italic
            .paint(format!(" {} ", rank.to_index() + 1))
            .to_string(),
    );
    let files = if reverse_board {
        Either::Left(chess::ALL_FILES.iter().rev())
    } else {
        Either::Right(chess::ALL_FILES.iter())
    };
    for file in files {
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
