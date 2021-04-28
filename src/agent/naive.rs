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
enum DepthAndEval {
    Exact(i64, usize, Evaluation),
    LowerBound(i64, usize, Evaluation),
    UpperBound(i64, usize, Evaluation),
}

impl DepthAndEval {
    fn hash(&self) -> i64 {
        match *self {
            Self::Exact(hash, _, _) => hash,
            Self::LowerBound(hash, _, _) => hash,
            Self::UpperBound(hash, _, _) => hash,
        }
    }

    fn depth(&self) -> usize {
        match *self {
            Self::Exact(_, depth, _) => depth,
            Self::LowerBound(_, depth, _) => depth,
            Self::UpperBound(_, depth, _) => depth,
        }
    }

    fn value(&self) -> Evaluation {
        match *self {
            Self::Exact(_, _, value) => value,
            Self::LowerBound(_, _, value) => value,
            Self::UpperBound(_, _, value) => value,
        }
    }
}

struct Evaluator {
    cache: RefCell<Vec<Option<DepthAndEval>>>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            cache: RefCell::new(vec![None; CACHE_SIZE]),
        }
    }
}

impl Evaluator {
    fn get_evaluation(&self, game: &Game) -> Option<DepthAndEval> {
        match self.cache.borrow()[(game.hash as usize) & (CACHE_SIZE - 1)] {
            None => None,
            val => {
                if val.unwrap().hash() == game.hash {
                    val
                } else {
                    None
                }
            }
        }
    }

    fn update_evaluation(&self, game: &Game, depth_and_eval: DepthAndEval) {
        let mut cache = self.cache.borrow_mut();
        cache[(game.hash as usize) & (CACHE_SIZE - 1)] = Some(depth_and_eval);
    }
}

struct Node {
    hash: i64,
    evaluation: Option<Evaluation>,
    children: Vec<(Move, Option<Rc<RefCell<Node>>>)>,
}

impl Default for Node {
    fn default() -> Self {
        Self::new(Game::default())
    }
}

impl Node {
    fn new(game: Game) -> Self {
        Self {
            hash: game.hash,
            evaluation: None,
            children: vec![],
        }
    }

    fn best_move(&self) -> Move {
        self.children[0].0.clone()
    }

    fn first_child(&self) -> Rc<RefCell<Self>> {
        Rc::clone(self.children[0].1.as_ref().unwrap())
    }

    fn is_expanded(&self) -> bool {
        self.children.len() > 0
    }

    fn expand(&mut self, game: &Game) {
        if !self.is_expanded() {
            let moves = game.position.legal_moves();
            for m in moves.iter() {
                self.children.push((m.clone(), None));
            }
        }
    }

    fn size(&self) -> usize {
        let mut size = 1;
        for i in 0..self.children.len() {
            size += match &self.children[i].1 {
                None => 0,
                Some(node) => node.borrow().size(),
            };
        }
        size
    }

    fn sort_children(&mut self) {
        self.children.sort_by(|a, b| match (&a.1, &b.1) {
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
        for (_, rc) in head_node.borrow().children.iter() {
            if let Some(node) = rc {
                if node.borrow().hash == game.hash {
                    self.head = Rc::clone(node);
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

    fn alpha_beta(
        &self,
        game: &Game,
        node: &mut Node,
        mut depth: usize,
        mut alpha: Evaluation,
        mut beta: Evaluation,
    ) -> Evaluation {
        if game.position.is_check() && depth > 0 {
            depth -= 1;
        }
        let alpha_orig = alpha;
        let mut value = Evaluation::MIN;
        if depth > 0 {
            node.expand(&game);
        }
        let evaluation = match self.evaluator.get_evaluation(&game) {
            None => None,
            Some(depth_and_eval) => {
                if depth_and_eval.depth() >= depth {
                    value = depth_and_eval.value();
                    match depth_and_eval {
                        DepthAndEval::Exact(_, _, evaluation) => Some(evaluation),
                        DepthAndEval::LowerBound(_, _, evaluation) => {
                            alpha = cmp::max(alpha, evaluation);
                            None
                        }
                        DepthAndEval::UpperBound(_, _, evaluation) => {
                            beta = cmp::max(beta, evaluation);
                            None
                        }
                    }
                } else {
                    None
                }
            }
        };
        let value = if let Some(evaluation) = evaluation {
            evaluation
        } else if depth == 0 || game.position.is_game_over() {
            let value = Evaluation::evaluate(&game);
            self.evaluator
                .update_evaluation(&game, DepthAndEval::Exact(game.hash, depth, value));
            value
        } else {
            for (child_move, child_node) in node.children.iter_mut() {
                let mut child_node = child_node
                    .get_or_insert_with(|| {
                        Rc::new(RefCell::new(Node::new(game.clone().play(&child_move))))
                    })
                    .borrow_mut();

                let child_value = -self
                    .alpha_beta(
                        &game.play(&child_move),
                        &mut child_node,
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
            node.sort_children();
            let depth_and_eval = if value <= alpha_orig {
                DepthAndEval::UpperBound(game.hash, depth, value)
            } else if value >= beta {
                DepthAndEval::LowerBound(game.hash, depth, value)
            } else {
                DepthAndEval::Exact(game.hash, depth, value)
            };
            self.evaluator.update_evaluation(&game, depth_and_eval);
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
        let rc = self.head.borrow().first_child();
        self.head = rc;
        game.play(&m)
    }
}
