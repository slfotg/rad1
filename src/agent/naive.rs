use super::ChessAgent;
use shakmaty::{Color, Move, Position};
use std::cell::RefCell;
use std::cmp;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::game::Game;

mod eval;

use eval::Evaluation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DepthAndEval {
    Exact(usize, Evaluation),
    LowerBound(usize, Evaluation),
    UpperBound(usize, Evaluation),
}

impl DepthAndEval {
    fn depth(&self) -> usize {
        match *self {
            Self::Exact(depth, _) => depth,
            Self::LowerBound(depth, _) => depth,
            Self::UpperBound(depth, _) => depth,
        }
    }
    fn value(&self) -> Evaluation {
        match *self {
            Self::Exact(_, value) => value,
            Self::LowerBound(_, value) => value,
            Self::UpperBound(_, value) => value,
        }
    }
}

struct Evaluator {
    cache: RefCell<HashMap<i64, DepthAndEval>>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            cache: RefCell::new(HashMap::default()),
        }
    }
}

impl Evaluator {
    fn get_evaluation(&self, game: &Game) -> Option<DepthAndEval> {
        match self.cache.borrow().get(&game.hash) {
            None => None,
            Some(val) => Some(*val),
        }
    }

    fn update_evaluation(&self, game: &Game, depth_and_eval: DepthAndEval) {
        let mut cache = self.cache.borrow_mut();
        cache.insert(game.hash, depth_and_eval);
    }
}

struct Node {
    game: Game,
    children: Vec<(Move, Option<Rc<RefCell<Node>>>)>,
}

impl Drop for Node {
    fn drop(&mut self) {
        //println!("drop");
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
            game,
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

    fn expand(&mut self) {
        if !self.is_expanded() {
            let moves = self.game.position.legal_moves();
            for m in moves.iter() {
                self.children.push((m.clone(), None));
            }
        }
    }

    fn sort_children(&mut self, evaluator: &Evaluator) {
        self.children.sort_by(|a, b| match (&a.1, &b.1) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(node_a), Some(node_b)) => {
                match (
                    evaluator.get_evaluation(&node_a.borrow().game),
                    evaluator.get_evaluation(&node_b.borrow().game),
                ) {
                    (None, None) => Ordering::Equal,
                    (None, _) => Ordering::Greater,
                    (Some(_), None) => Ordering::Less,
                    (Some(a_val), Some(b_val)) => a_val.value().cmp(&b_val.value()),
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

    fn update_head(&mut self, game: &Game) {
        let mut updated = false;
        let head_node = Rc::clone(&self.head);
        for (_, rc) in head_node.borrow().children.iter() {
            if let Some(node) = rc {
                if node.borrow().game.hash == game.hash {
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
        node: &mut Node,
        depth: usize,
        mut alpha: Evaluation,
        mut beta: Evaluation,
    ) -> Evaluation {
        let alpha_orig = alpha;
        let mut value = Evaluation::MIN;
        let game = node.game.clone();
        if depth > 0 {
            node.expand();
        }
        let evaluation = match self.evaluator.get_evaluation(&game) {
            None => None,
            Some(depth_and_eval) => {
                if depth_and_eval.depth() >= depth {
                    value = depth_and_eval.value();
                    match depth_and_eval {
                        DepthAndEval::Exact(_, evaluation) => Some(evaluation),
                        DepthAndEval::LowerBound(_, evaluation) => {
                            alpha = cmp::max(alpha, evaluation);
                            None
                        }
                        DepthAndEval::UpperBound(_, evaluation) => {
                            beta = cmp::max(beta, evaluation);
                            None
                        }
                    }
                } else {
                    None
                }
            }
        };
        if let Some(evaluation) = evaluation {
            evaluation
        } else if depth == 0 || game.position.is_game_over() {
            let value = Evaluation::evaluate(&game);
            self.evaluator
                .update_evaluation(&game, DepthAndEval::Exact(depth, value));
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
            node.sort_children(&self.evaluator);
            if value <= alpha_orig {
                self.evaluator
                    .update_evaluation(&game, DepthAndEval::UpperBound(depth, value));
            } else if value >= beta {
                self.evaluator
                    .update_evaluation(&game, DepthAndEval::LowerBound(depth, value));
            } else {
                self.evaluator
                    .update_evaluation(&game, DepthAndEval::Exact(depth, value));
            }
            value
        }
    }
}

impl ChessAgent for NaiveChessAgent {
    fn take_turn(&mut self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
        self.update_head(&game);
        for i in 1..=self.depth {
            self.alpha_beta(
                &mut self.head.borrow_mut(),
                i,
                Evaluation::MIN,
                Evaluation::MAX,
            );
            let game = self.head.borrow().first_child().borrow().game.clone();
            println!(
                "{} - {} = {:?}",
                i,
                self.head.borrow().best_move(),
                self.evaluator.get_evaluation(&game).unwrap().value()
            );
        }
        let rc = self.head.borrow().first_child();
        self.head = rc;
        self.head.borrow().game.clone()
    }
}
