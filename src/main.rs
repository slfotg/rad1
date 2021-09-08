use ansi_term::Colour;
use ansi_term::Style;
use clap::{App, AppSettings, Arg, SubCommand};
use rad1::agent;
use rad1::agent::ChessAgent;
use rad1::game::Game;
use shakmaty::fen::Fen;
use shakmaty::*;

fn main() {
    let matches = App::new("Rad1 Chess Engine")
        .setting(AppSettings::SubcommandRequired)
        .version("0.1.0")
        .author("Sam Foster <slfotg@gmail.com>")
        .about("A Simple Chess Engine in Rust")
        .subcommand(
            SubCommand::with_name("play").about("Play against the chess engine from terminal"),
        )
        .subcommand(
            SubCommand::with_name("analyze")
                .about("Analyze a single position")
                .arg(
                    Arg::with_name("fen")
                        .long("fen")
                        .short("f")
                        .required(true)
                        .takes_value(true)
                        .help("The Forsyth-Edwards Notation (FEN) of the position to be analyzed"),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("analyze") {
        let fen = matches.value_of("fen").unwrap();
        let setup: Fen = fen.parse().expect("Failed to parse FEN");
        let position: Chess = setup
            .position(CastlingMode::Standard)
            .expect("Failed to setup position from FEN");
        let chess_game = Game::from_position(position);
        analyze_position(&chess_game);
    } else {
        play_game();
    }
}

fn analyze_position(chess_game: &Game) {
    let mut agent = agent::alpha_beta_agent(8);
    agent.best_move(chess_game);
}

fn play_game() {
    let mut chess_game = Game::default();
    let mut player1 = agent::command_line_agent();
    let mut player2 = agent::alpha_beta_agent(3);
    let mut current_player = Color::White;
    print_game(&chess_game.position);
    while !chess_game.position.is_game_over() {
        let best_move = match current_player {
            Color::White => player1.best_move(&chess_game),
            Color::Black => player2.best_move(&chess_game),
        };
        chess_game.play_mut(&best_move);
        current_player = !current_player;
        print_game(&chess_game.position);
    }
    println!("{:?}", chess_game.position.outcome());
}

fn print_game(game: &Chess) {
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support();

    let italic: Style = Style::new().italic();
    let fg_black: Colour = Colour::Fixed(16);
    let bg_black: Style = fg_black.on(Colour::Fixed(34));
    let bg_white: Style = fg_black.on(Colour::Fixed(220));
    let board = game.board();
    for rank in (0..8).rev() {
        let mut line: String = String::from("");
        let mut background = if rank % 2 == 1 { bg_white } else { bg_black };
        line.push_str(&italic.paint(format!(" {} ", rank + 1)).to_string());
        for file in 0..8 {
            let square = Square::new(rank * 8 + file);
            let piece_char = board
                .piece_at(square)
                .map_or(" ", |piece| get_piece_char(piece));
            line.push_str(&background.paint(format!(" {} ", piece_char)).to_string());
            background = if background == bg_white {
                bg_black
            } else {
                bg_white
            };
        }
        println!("{}", line);
    }
    println!("{}", italic.paint("    A  B  C  D  E  F  G  H"));
}

fn get_piece_char(piece: Piece) -> &'static str {
    match (piece.color, piece.role) {
        (Color::White, Role::Pawn) => "♙",
        (Color::White, Role::Knight) => "♘",
        (Color::White, Role::Bishop) => "♗",
        (Color::White, Role::Rook) => "♖",
        (Color::White, Role::Queen) => "♕",
        (Color::White, Role::King) => "♔",
        (Color::Black, Role::Pawn) => "♟︎",
        (Color::Black, Role::Knight) => "♞",
        (Color::Black, Role::Bishop) => "♝",
        (Color::Black, Role::Rook) => "♜",
        (Color::Black, Role::Queen) => "♛",
        (Color::Black, Role::King) => "♚",
    }
}
