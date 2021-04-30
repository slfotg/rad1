use super::ChessAgent;
use shakmaty::{Color, Move, Position};
use std::cell::RefCell;
use std::cmp;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::game::Game;

mod eval;

use eval::Evaluation;

const CACHE_SIZE: usize = 8388608;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CachedEvaluation {
    Empty,
    Exact(i64, usize, Evaluation),
    LowerBound(i64, usize, Evaluation),
    UpperBound(i64, usize, Evaluation),
}

impl CachedEvaluation {
    fn hash(&self) -> i64 {
        match *self {
            Self::Exact(hash, _, _) => hash,
            Self::LowerBound(hash, _, _) => hash,
            Self::UpperBound(hash, _, _) => hash,
            _ => 0,
        }
    }

    fn depth(&self) -> usize {
        match *self {
            Self::Exact(_, depth, _) => depth,
            Self::LowerBound(_, depth, _) => depth,
            Self::UpperBound(_, depth, _) => depth,
            _ => 0,
        }
    }

    fn value(&self) -> Evaluation {
        match *self {
            Self::Exact(_, _, value) => value,
            Self::LowerBound(_, _, value) => value,
            Self::UpperBound(_, _, value) => value,
            _ => Evaluation::ZERO,
        }
    }
}

struct Evaluator {
    cache: RefCell<Vec<CachedEvaluation>>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            cache: RefCell::new(vec![CachedEvaluation::Empty; CACHE_SIZE]),
        }
    }
}

impl Evaluator {
    fn get_evaluation(&self, game: &Game) -> CachedEvaluation {
        match self.cache.borrow()[(game.hash() as usize) & (CACHE_SIZE - 1)] {
            CachedEvaluation::Empty => CachedEvaluation::Empty,
            val => {
                if val.hash() == game.hash() {
                    val
                } else {
                    CachedEvaluation::Empty
                }
            }
        }
    }

    fn update_evaluation(&self, game: &Game, cached_eval: CachedEvaluation) {
        let mut cache = self.cache.borrow_mut();
        cache[(game.hash() as usize) & (CACHE_SIZE - 1)] = cached_eval;
    }
}

struct Node {
    hash: i64,
    evaluation: Option<Evaluation>,
    children: Vec<LazyNode>,
}

struct LazyNode {
    chess_move: Move,
    node: Option<Rc<RefCell<Node>>>,
}

impl LazyNode {
    fn new(chess_move: Move) -> Self {
        Self {
            chess_move,
            node: None,
        }
    }

    fn node(&mut self, game: &Game) -> Rc<RefCell<Node>> {
        let chess_move = self.chess_move.clone();
        self.node
            .get_or_insert_with(|| Rc::new(RefCell::new(Node::new(game.play(&chess_move)))))
            .clone()
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

    fn first_child(&mut self, game: &Game) -> Rc<RefCell<Self>> {
        self.children[0].node(game)
    }

    fn is_expanded(&self) -> bool {
        self.children.len() > 0
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
    color: Color,
    depth: usize,
    evaluator: Evaluator,
    head: Rc<RefCell<Node>>,
}

impl NaiveChessAgent {
    pub fn new(color: Color, depth: usize) -> Self {
        Self {
            color,
            depth,
            evaluator: Evaluator::default(),
            head: Rc::new(RefCell::new(Node::default())),
        }
    }

    fn size(&self) -> usize {
        self.head.borrow().size()
    }

    fn update_head(&mut self, game: &Game) {
        let mut updated = false;
        let head_node = Rc::clone(&self.head);
        for i in 0..head_node.borrow().children.len() {
            let child = &head_node.borrow().children[i];
            if let Some(node) = &child.node {
                if node.borrow().hash == game.hash() {
                    self.head = Rc::clone(&node);
                    updated = true;
                    break;
                }
            }
        }
        if !updated {
            self.head = Rc::new(RefCell::new(Node::new(game.clone())));
            self.evaluator = Evaluator::default();
        }
    }

    fn cached_evaluation(
        &self,
        game: &Game,
        depth: usize,
        value: &mut Evaluation,
        alpha: &mut Evaluation,
        beta: &mut Evaluation,
    ) -> Option<Evaluation> {
        match self.evaluator.get_evaluation(game) {
            CachedEvaluation::Empty => None,
            cached_eval => {
                if cached_eval.depth() >= depth {
                    *value = cached_eval.value();
                    match cached_eval {
                        CachedEvaluation::Exact(_, _, evaluation) => Some(evaluation),
                        CachedEvaluation::LowerBound(_, _, evaluation) => {
                            *alpha = cmp::max(*alpha, evaluation);
                            None
                        }
                        CachedEvaluation::UpperBound(_, _, evaluation) => {
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

    fn q_search(&self, game: &Game, mut alpha: Evaluation, beta: Evaluation) -> Evaluation {
        let evaluation = Evaluation::evaluate(game);
        if evaluation >= beta {
            beta
        } else {
            if alpha < evaluation {
                alpha = evaluation;
            }
            for m in game.sorted_captures().into_iter() {
                let score = -self.q_search(&game.play(&m), -beta, -alpha).increment();
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

    fn alpha_beta(
        &self,
        game: &Game,
        node: &mut Node,
        mut depth: usize,
        mut alpha: Evaluation,
        mut beta: Evaluation,
    ) -> Evaluation {
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
                .update_evaluation(game, CachedEvaluation::Exact(game.hash(), depth, value));
            value
        } else {
            for child_node in node.children.iter_mut() {
                let child_move = child_node.chess_move.clone();

                let child_value = -self
                    .alpha_beta(
                        &game.play(&child_move),
                        &mut child_node.node(game).borrow_mut(),
                        depth - 1,
                        -beta.decrement(),
                        -alpha.decrement(),
                    )
                    .increment();
                value = cmp::max(child_value, value);
                alpha = cmp::max(alpha, value);
                if alpha >= beta {
                    break;
                }
            }
            node.sort_children_by_evaluation();
            let cached_eval = if value <= alpha_orig {
                CachedEvaluation::UpperBound(game.hash(), depth, value)
            } else if value >= beta {
                CachedEvaluation::LowerBound(game.hash(), depth, value)
            } else {
                CachedEvaluation::Exact(game.hash(), depth, value)
            };
            self.evaluator.update_evaluation(&game, cached_eval);
            value
        };
        node.evaluation = Some(value);
        value
    }
}

impl ChessAgent for NaiveChessAgent {
    fn take_turn(&mut self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
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
        let m = self.head.borrow().best_move();
        println!("Best move: {}", m);
        println!("Size: {}", self.size());

        // update head of tree
        let rc = self.head.borrow_mut().first_child(&game);
        self.head = rc;
        game.play(&m)
    }
}
