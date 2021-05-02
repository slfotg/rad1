use rad1::agent;
use rad1::agent::ChessAgent;
use rad1::game::Game;
use shakmaty::*;

fn main() {
    let mut chess_game = Game::default();
    let mut player1 = agent::command_line_agent();
    let mut player2 = agent::naive_chess_agent(8);
    let mut current_player = Color::White;
    print_game(&chess_game.position);
    while !chess_game.position.is_game_over() {
        let best_move = match current_player {
            Color::White => {
                current_player = Color::Black;
                player1.best_move(&chess_game)
            }
            Color::Black => {
                current_player = Color::White;
                player2.best_move(&chess_game)
            }
        };
        chess_game.play_mut(&best_move);
        print_game(&chess_game.position);
    }
    println!("{:?}", chess_game.position.outcome());
}

fn print_game(game: &Chess) {
    const ITALIC: &str = "\u{001b}[3m";
    const FG_BLACK: &str = "\u{001b}[38;5;16m";
    const BG_BLACK: &str = "\u{001b}[48;5;34m";
    const BG_WHITE: &str = "\u{001b}[48;5;220m";
    const RESET: &str = "\u{001b}[0m";
    let board = game.board();
    for rank in (0..8).rev() {
        let mut background = if rank % 2 == 1 { BG_WHITE } else { BG_BLACK };
        print!("{} {} {}{}", ITALIC, rank + 1, RESET, FG_BLACK);
        for file in 0..8 {
            let square = Square::new(rank * 8 + file);
            let piece_char = board
                .piece_at(square)
                .map_or(" ", |piece| get_piece_char(piece));
            print!("{} {} ", background, piece_char);
            background = if background == BG_WHITE {
                BG_BLACK
            } else {
                BG_WHITE
            };
        }
        println!("{}", RESET);
    }
    println!("{}    A  B  C  D  E  F  G  H{}", ITALIC, RESET);
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
