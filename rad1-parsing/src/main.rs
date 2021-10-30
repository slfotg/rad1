use core::num::NonZeroU32;
use pgn_reader::{BufferedReader, Color, Outcome, SanPlus, Skip, Visitor};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use shakmaty::fen;
use shakmaty::{Chess, Position, Setup};
use std::fs::File;
use std::io::{prelude::*, BufReader, LineWriter};

#[derive(Default)]
struct PgnGame {
    game: Chess,
    fens: Vec<String>,
    outcome: Option<Outcome>,
    error: bool,
}

fn parse_outcome(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Decisive {
            winner: Color::White,
        } => "W",
        Outcome::Decisive {
            winner: Color::Black,
        } => "B",
        _ => "D",
    }
}

impl Visitor for PgnGame {
    type Result = ();

    fn begin_game(&mut self) {
        self.game = Chess::default();
        self.fens = vec![];
        self.outcome = None;
        self.error = false;
    }

    fn san(&mut self, san_plus: SanPlus) {
        if !self.error {
            match san_plus.san.to_move(&self.game) {
                Ok(chess_move) => {
                    self.game.play_unchecked(&chess_move);
                    // skip opening moves and checkmates for evaluation
                    if self.game.fullmoves() >= NonZeroU32::new(5).unwrap()
                        && !self.game.is_checkmate()
                    {
                        self.fens.push(fen::fen(&self.game));
                    }
                }
                _ => {
                    self.error = true;
                }
            }
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        if !self.error {
            if let Some(outcome) = self.outcome {
                for fen in self.fens.iter() {
                    println!("{}\t{}", parse_outcome(outcome), fen);
                }
            }
        }
    }
    fn outcome(&mut self, outcome: Option<Outcome>) {
        if !self.error {
            self.outcome = outcome;
        }
    }
}

fn main() {
    println!("Init");
    let mut indexes: Vec<usize> = (0..204270106).collect();
    println!("Shuffle");
    indexes.shuffle(&mut thread_rng());
    let mut ns = indexes.as_mut_slice();

    // sort the first thousand 10,000 piece chunks
    // to use as dataset
    for _ in 0..1000 {
        let (left, right) = ns.split_at_mut(10000);
        left.sort();
        ns = right;
    }
    (0..1000).into_par_iter().for_each(|n| {
        let file = File::open("/home/sam/repos/rad1/rad1-parsing/all_fens.txt").unwrap();
        let out_file = File::create(format!("/home/sam/repos/rad1/dataset/{:0>4}.fen", n)).unwrap();
        let reader = BufReader::new(file);
        let mut writer = LineWriter::new(out_file);
        let mut i = 0;
        let mut line_number = 0;
        for line in reader.lines() {
            if indexes[n * 10000 + i] == line_number {
                writer
                    .write_all(format!("{}\n", &line.unwrap()).as_bytes())
                    .expect("fail write");
                i += 1;
                if i == 10000 {
                    break;
                }
            }
            line_number += 1;
        }
        println!("{}", indexes[n * 10000]);
    });
    // for file in fs::read_dir("/home/sam/repos/rad1/pgns/unzip").unwrap() {
    //     convert(format!("{}", file.unwrap().path().display()));
    // }
}

fn _convert(path: String) {
    let file = File::open(path).unwrap();
    let mut reader = BufferedReader::new(file);
    let mut game = PgnGame::default();
    loop {
        match reader.read_game(&mut game) {
            Ok(None) => {
                break;
            }
            _ => {
                // do nothing
            }
        }
    }
}
