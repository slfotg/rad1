use rad1::agent;
use rad1::agent::ChessAgent;
use shakmaty::*;

fn main() {
    let mut chess_game = Chess::default();
    let mut player1 = agent::command_line_agent(Color::White);
    let mut player2 = agent::random_chess_agent(Color::Black);
    let mut current_player = Color::White;
    print_game(&chess_game);
    while !chess_game.is_game_over() {
        match current_player {
            Color::White => {
                chess_game = player1.take_turn(chess_game);
                current_player = Color::Black;
            },
            Color::Black => {
                chess_game = player2.take_turn(chess_game);
                current_player = Color::White;
            }
        };
        print_game(&chess_game);
    }
    println!("{:?}", chess_game.outcome());
}

fn print_game(game: &Chess) {
    const ITALIC: &str = "\u{001b}[3m";
    const FG_BLACK: &str = "\u{001b}[38;5;16m";
    const BG_BLACK: &str = "\u{001b}[48;5;34m";
    const BG_WHITE: &str = "\u{001b}[48;5;220m";
    const RESET: &str = "\u{001b}[0m";
    let mut rank_num = 8;
    let board = game.board();
    for rank in 0..8 {
        let mut background = if rank_num % 2 == 0 {
            BG_WHITE
        } else {
            BG_BLACK
        };
        print!("{} {} {}{}", ITALIC, rank_num, RESET, FG_BLACK);
        for file in 0..8 {
            let square = Square::new((7 - rank) * 8 + file);
            let piece_char = board.piece_at(square).map_or(" ", |piece| get_piece_char(piece));
            print!("{} {} ", background, piece_char);
            background = if background == BG_WHITE {
                BG_BLACK
            } else {
                BG_WHITE
            };
        }
        print!("{}\n", RESET);
        rank_num -= 1;
    }
    println!("{}    A  B  C  D  E  F  G  H{}", ITALIC, RESET);
}

fn get_piece_char(piece: Piece) -> &'static str {
    match piece.color {
        Color::White => match piece.role {
            Role::Pawn => "♙",
            Role::Knight => "♘",
            Role::Bishop => "♗",
            Role::Rook => "♖",
            Role::Queen => "♕",
            Role::King => "♔",
        },
        Color::Black => match piece.role {
            Role::Pawn => "♟︎",
            Role::Knight => "♞",
            Role::Bishop => "♝",
            Role::Rook => "♜",
            Role::Queen => "♛",
            Role::King => "♚",
        },
    }
}