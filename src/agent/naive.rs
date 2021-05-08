use super::ChessAgent;
use shakmaty::{Move, Position};
use std::cell::RefCell;
use std::cmp;
use std::cmp::Ordering;

use crate::eval::Evaluation;
use crate::game::Game;
use crate::tt::*;

struct Node {
    hash: u64,
    evaluation: Option<i16>,
    children: Vec<LazyNode>,
}

struct LazyNode {
    chess_move: Move,
    node: Option<RefCell<Node>>,
}

impl LazyNode {
    fn new(chess_move: Move) -> Self {
        Self {
            chess_move,
            node: None,
        }
    }

    fn node(&mut self, game: &Game) -> &RefCell<Node> {
        let chess_move = self.chess_move.clone();
        self.node
            .get_or_insert_with(|| RefCell::new(Node::new(game.play(&chess_move))))
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new(Game::default())
    }
}

impl Node {
    fn new(game: Game) -> Self {
        Self {
            hash: game.hash(),
            evaluation: None,
            children: vec![],
        }
    }

    fn best_move(&self) -> Move {
        self.children[0].chess_move.clone()
    }

    fn first_child(&mut self) -> Option<RefCell<Self>> {
        self.children[0].node.take()
    }

    fn find_child(&mut self, game: &Game) -> Option<RefCell<Self>> {
        for i in 0..self.children.len() {
            if let Some(node) = &self.children[i].node {
                if node.borrow().hash == game.hash() {
                    return self.children[i].node.take();
                }
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
                self.children.push(LazyNode::new(m));
            }
        }
    }

    fn size(&self) -> usize {
        let mut size = 1;
        for i in 0..self.children.len() {
            size += match &self.children[i].node {
                None => 0,
                Some(node) => node.borrow().size(),
            };
        }
        size
    }

    fn sort_children_by_evaluation(&mut self) {
        self.children
            .sort_by(|a, b| match (a.node.as_ref(), b.node.as_ref()) {
                (None, None) => Ordering::Equal,
                (None, _) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(node_a), Some(node_b)) => {
                    match (node_a.borrow().evaluation, node_b.borrow().evaluation) {
                        (None, None) => Ordering::Equal,
                        (None, _) => Ordering::Greater,
                        (Some(_), None) => Ordering::Less,
                        (Some(a_val), Some(b_val)) => a_val.cmp(&b_val),
                    }
                }
            });
    }
}

pub struct NaiveChessAgent {
    depth: usize,
    evaluator: TranspositionTable,
    head: RefCell<Node>,
}

impl NaiveChessAgent {
    pub fn new(depth: usize) -> Self {
        Self {
            depth,
            evaluator: TranspositionTable::default(),
            head: RefCell::new(Node::default()),
        }
    }

    fn size(&self) -> usize {
        self.head.borrow().size()
    }

    fn update_head(&mut self, game: &Game) {
        let mut updated = false;
        let child = self.head.borrow_mut().find_child(game);
        if let Some(node) = child {
            self.head = node;
            updated = true;
        }
        if !updated {
            self.head = RefCell::new(Node::new(game.clone()));
            self.evaluator = TranspositionTable::default();
        }
    }

    fn cached_evaluation(
        &self,
        game: &Game,
        depth: usize,
        value: &mut i16,
        alpha: &mut i16,
        beta: &mut i16,
    ) -> Option<i16> {
        match self.evaluator.get_evaluation(game) {
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

    fn q_search(&self, game: &Game, mut alpha: i16, beta: i16) -> i16 {
        let evaluation = Evaluation::evaluate(game);
        if evaluation >= beta {
            beta
        } else {
            if alpha < evaluation {
                alpha = evaluation;
            }
            for m in game.sorted_captures().into_iter() {
                let score = -self.q_search(&game.play(&m), -beta, -alpha);
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
                let val = -Self::null_alpha_beta(&game.play(child_move), depth - 1, -beta, -alpha);
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

    fn alpha_beta(
        &self,
        game: &Game,
        node: &mut Node,
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
            node.expand(game);
        }
        let cached_evaluation =
            self.cached_evaluation(game, depth, &mut value, &mut alpha, &mut beta);
        let value = if let Some(evaluation) = cached_evaluation {
            evaluation
        } else if game.position.is_game_over() {
            Evaluation::evaluate(game)
        } else if depth == 0 {
            let value = self.q_search(game, alpha, beta);
            self.evaluator
                .update_evaluation(game, CachedValue::Exact(game.hash(), depth, value));
            value
        } else {
            if depth >= 3 {
                if let Ok(null_move) = game.swap_turn() {
                    let score = -Self::null_alpha_beta(&null_move, depth - 3, -beta, -beta + 1);
                    if score >= beta {
                        node.evaluation = Some(beta);
                        return beta;
                    }
                }
            }
            for child_node in node.children.iter_mut() {
                let child_move = child_node.chess_move.clone();

                let child_value = -self.alpha_beta(
                    &game.play(&child_move),
                    &mut child_node.node(game).borrow_mut(),
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
            node.sort_children_by_evaluation();
            let cached_eval = if value <= alpha_orig {
                CachedValue::Beta(game.hash(), depth, value)
            } else if value >= beta {
                CachedValue::Alpha(game.hash(), depth, value)
            } else {
                CachedValue::Exact(game.hash(), depth, value)
            };
            self.evaluator.update_evaluation(&game, cached_eval);
            value
        };
        node.evaluation = Some(value);
        value
    }
}

impl ChessAgent for NaiveChessAgent {
    fn best_move(&mut self, game: &Game) -> Move {
        self.update_head(&game);
        for i in 1..=self.depth {
            self.alpha_beta(
                &game,
                &mut self.head.borrow_mut(),
                i,
                Evaluation::MIN,
                Evaluation::MAX,
            );
            println!(
                "{} - {} = {:?}",
                i,
                self.head.borrow().best_move(),
                self.head.borrow().evaluation,
            );
        }

        // get best move
        let best_move = self.head.borrow().best_move();
        println!("Best move: {}", best_move);
        println!("Size: {}", self.size());

        // update head of tree
        let first_child = self.head.borrow_mut().first_child();
        self.head = first_child.unwrap();
        best_move
    }
}
