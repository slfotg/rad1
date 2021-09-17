use super::ChessAgent;
use shakmaty::san::San;
use shakmaty::{Move, Position};
use std::cmp;
use std::cmp::Ordering;

use crate::eval::Evaluation;
use crate::game::Game;
use crate::tt::*;

// quiescence search
fn q_search(game: &Game, mut alpha: i16, beta: i16) -> i16 {
    let evaluation = Evaluation::evaluate(game);
    if evaluation >= beta {
        beta
    } else {
        if alpha < evaluation {
            alpha = evaluation;
        }
        for m in game.sorted_captures().into_iter() {
            let score = -q_search(&game.play(&m), -beta, -alpha);
            if score >= beta {
                alpha = beta;
                break;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
}

// this is really just a pure alpha beta search
// with no caching or storing evaluations in nodes
// used for the null move heursitic
fn null_alpha_beta(game: &Game, depth: usize, mut alpha: i16, beta: i16) -> i16 {
    if depth == 0 {
        Evaluation::evaluate(game)
    } else {
        for child_move in game.sorted_moves().iter() {
            let val = -null_alpha_beta(&game.play(child_move), depth - 1, -beta, -alpha);
            if val >= beta {
                return beta;
            }
            if val > alpha {
                alpha = val;
            }
        }
        alpha
    }
}

fn cached_evaluation(
    trans_table: &TranspositionTable,
    game: &Game,
    depth: usize,
    value: &mut i16,
    alpha: &mut i16,
    beta: &mut i16,
) -> Option<i16> {
    match trans_table.get_evaluation(game) {
        CachedValue::Empty => None,
        cached_eval => {
            if cached_eval.depth() >= depth {
                *value = cached_eval.value();
                match cached_eval {
                    CachedValue::Exact(_, _, evaluation) => Some(evaluation),
                    CachedValue::Alpha(_, _, evaluation) => {
                        *alpha = cmp::max(*alpha, evaluation);
                        None
                    }
                    CachedValue::Beta(_, _, evaluation) => {
                        *beta = cmp::max(*beta, evaluation);
                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

struct Node {
    hash: u64,
    evaluation: Option<i16>,
    chess_move: Option<Move>,
    children: Vec<Node>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            hash: Game::default().hash(),
            evaluation: None,
            chess_move: None,
            children: vec![],
        }
    }
}

impl Node {
    fn new(game: Game, chess_move: Option<Move>) -> Self {
        Self {
            hash: game.hash(),
            evaluation: None,
            chess_move,
            children: vec![],
        }
    }

    fn best_move(&self) -> Option<Move> {
        self.children[0].chess_move.clone()
    }

    fn first_child(&mut self) -> Self {
        self.children.remove(0)
    }

    fn find_child(&mut self, game: &Game) -> Option<Self> {
        for i in 0..self.children.len() {
            if self.children[i].hash == game.hash() {
                return Some(self.children.remove(i));
            }
        }
        None
    }

    fn is_expanded(&self) -> bool {
        !self.children.is_empty()
    }

    fn expand(&mut self, game: &Game) {
        if !self.is_expanded() {
            let moves = game.sorted_moves();
            for m in moves.into_iter() {
                self.children.push(Node::new(game.play(&m), Some(m)));
            }
        }
    }

    fn size(&self) -> usize {
        let mut size = 1;
        for i in 0..self.children.len() {
            size += self.children[i].size();
        }
        size
    }

    fn sort_children_by_evaluation(&mut self) {
        self.children
            .sort_by(|a, b| match (a.evaluation, b.evaluation) {
                (None, None) => Ordering::Equal,
                (None, _) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a_val), Some(b_val)) => a_val.cmp(&b_val),
            });
    }

    fn alpha_beta(
        &mut self,
        trans_table: &TranspositionTable,
        game: &Game,
        mut depth: usize,
        mut alpha: i16,
        mut beta: i16,
    ) -> i16 {
        if game.position.is_check() && depth > 0 {
            depth += 1;
        }
        let alpha_orig = alpha;
        let mut value = Evaluation::MIN;
        if depth > 0 {
            self.expand(game);
        }
        let cached_evaluation =
            cached_evaluation(trans_table, game, depth, &mut value, &mut alpha, &mut beta);
        let value = if let Some(evaluation) = cached_evaluation {
            evaluation
        } else if game.position.is_game_over() {
            Evaluation::evaluate(game)
        } else if depth == 0 {
            let value = q_search(game, alpha, beta);
            trans_table.update_evaluation(game, CachedValue::Exact(game.hash(), depth, value));
            value
        } else {
            if depth >= 3 {
                if let Ok(null_move) = game.swap_turn() {
                    let score = -null_alpha_beta(&null_move, depth - 3, -beta, -beta + 1);
                    if score >= beta {
                        self.evaluation = Some(beta);
                        return beta;
                    }
                }
            }
            for child_node in self.children.iter_mut() {
                let child_move = child_node.chess_move.clone().unwrap();

                let child_value = -child_node.alpha_beta(
                    trans_table,
                    &game.play(&child_move),
                    depth - 1,
                    -beta,
                    -alpha,
                );
                value = cmp::max(child_value, value);
                alpha = cmp::max(alpha, value);
                if alpha >= beta {
                    break;
                }
            }
            self.sort_children_by_evaluation();
            let cached_eval = if value <= alpha_orig {
                CachedValue::Beta(game.hash(), depth, value)
            } else if value >= beta {
                CachedValue::Alpha(game.hash(), depth, value)
            } else {
                CachedValue::Exact(game.hash(), depth, value)
            };
            trans_table.update_evaluation(&game, cached_eval);
            value
        };
        self.evaluation = Some(value);
        value
    }
}

pub struct AlphaBetaChessAgent {
    depth: usize,
    evaluator: TranspositionTable,
    head: Option<Node>,
}

impl AlphaBetaChessAgent {
    pub fn new(depth: usize) -> Self {
        AlphaBetaChessAgent {
            depth,
            evaluator: TranspositionTable::default(),
            head: Some(Node::default()),
        }
    }

    fn size(&self) -> usize {
        self.head.as_ref().unwrap().size()
    }

    fn update_head(&mut self, game: &Game) {
        let child = self.head.as_mut().unwrap().find_child(game);
        if child.is_some() {
            self.head = child;
        } else {
            self.head = Some(Node::new(game.clone(), None));
            self.evaluator = TranspositionTable::default();
        }
    }
}

impl ChessAgent for AlphaBetaChessAgent {
    fn best_move(&mut self, game: &Game) -> Move {
        self.update_head(&game);
        let alpha = Evaluation::MIN;
        let beta = Evaluation::MAX;
        let mut head = self.head.take().unwrap();
        for i in 1..=self.depth {
            head.alpha_beta(&self.evaluator, &game, i, alpha, beta);
            println!(
                "{} - {} = {:?}",
                i,
                San::from_move(&game.position, &head.best_move().unwrap()).to_string(),
                head.evaluation.unwrap(),
            );
        }

        // get best move
        let best_move = head.best_move().unwrap();

        // update head of tree
        let first_child = head.first_child();
        self.head = Some(first_child);

        println!(
            "Best move: {}",
            San::from_move(&game.position, &best_move).to_string()
        );
        println!("Size: {}", self.size());
        best_move
    }
}
