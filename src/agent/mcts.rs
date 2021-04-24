use super::ChessAgent;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::seq::SliceRandom;
use shakmaty::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::{Add, AddAssign};

use crate::agent;
use crate::game::Game;

// number of simulations to run per move
const MAX_SIMULATIONS: usize = 100_000;

// exploration factor
// should be in the range (0, ~1.5)
// lower number = less exploration / more asymetrical tree
// higher number = more exploration / more symetrical tree
const EXPLORATION_FACTOR: f64 = 0.85;

// number of simulations need to be run to expand a node
const EXPANSION_MIN: f64 = 4.0;

#[derive(Debug, Copy, Clone)]
struct Score {
    white_wins: f64,
    black_wins: f64,
    games: f64,
}

// + for Score
impl Add for Score {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            white_wins: self.white_wins + other.white_wins,
            black_wins: self.black_wins + other.black_wins,
            games: self.games + other.games,
        }
    }
}

// += for Score
impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Default for Score {
    fn default() -> Self {
        Self {
            white_wins: 0.0,
            black_wins: 0.0,
            games: 0.0,
        }
    }
}

impl Score {
    fn new(outcome: &Outcome) -> Self {
        let mut white_wins = 0.0;
        let mut black_wins = 0.0;
        let games = 1.0;
        match *outcome {
            Outcome::Decisive { winner } => match winner {
                Color::White => white_wins = 1.0,
                Color::Black => black_wins = 1.0,
            },
            _ => (),
        };
        Self {
            white_wins,
            black_wins,
            games,
        }
    }

    fn exploitation_part(&self, for_color: Color) -> f64 {
        match for_color {
            Color::White => self.white_wins / self.games,
            Color::Black => self.black_wins / self.games,
        }
    }

    fn exploration_part(&self, parent_games: f64) -> f64 {
        EXPLORATION_FACTOR * (parent_games.ln() / self.games).sqrt()
    }

    fn uct(&self, for_color: Color, parent_games: f64) -> f64 {
        if self.games == 0.0 {
            f64::MAX
        } else {
            self.exploitation_part(for_color)
                + self.exploration_part(parent_games)
        }
    }

    fn order_by_uct(
        lhs: &Self,
        rhs: &Self,
        for_color: Color,
        parent_games: f64,
    ) -> Ordering {
        rhs.uct(for_color, parent_games)
            .partial_cmp(&lhs.uct(for_color, parent_games))
            .unwrap()
    }

    fn order_by_games(lhs: &Self, rhs: &Self) -> Ordering {
        rhs.games.partial_cmp(&lhs.games).unwrap()
    }
}

struct Evaluation {}

impl Evaluation {
    const PIECE_VALUES: [f64; 7] = [0.0, 1.0, 3.0, 3.0, 5.0, 9.0, 0.0];

    #[inline]
    fn piece_value(piece: &Piece) -> f64 {
        Self::PIECE_VALUES[usize::from(piece.role)]
    }

    #[inline]
    fn evaluate(game: &Game, color: Color) -> f64 {
        if game.position.is_game_over() {
            match game.position.outcome().unwrap().winner() {
                Option::None => 0.5,
                Option::Some(c) => {
                    if color == c {
                        1.0
                    } else {
                        0.0
                    }
                }
            }
        } else {
            let mut white_value = 0.0;
            let mut black_value = 0.0;
            for (_, piece) in game.position.board().pieces() {
                match piece.color {
                    Color::White => white_value += Self::piece_value(&piece),
                    _ => black_value += Self::piece_value(&piece),
                }
            }
            let sum_value = white_value + black_value;
            match color {
                Color::White => white_value / sum_value,
                _ => black_value / sum_value,
            }
        }
    }
}

struct Node {
    game: Game,
    score: Score,
    children: Vec<RefCell<Node>>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            game: Game::default(),
            score: Score::default(),
            children: vec![],
        }
    }
}

impl Node {
    fn new(game: Game) -> Self {
        Self {
            game,
            score: Score::default(),
            children: vec![],
        }
    }

    fn _len(&self) -> usize {
        if self.is_leaf() {
            1
        } else {
            self.children.iter().map(|c| c.borrow()._len()).sum()
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    fn expand(&mut self) {
        let mut rng = rand::thread_rng();
        self.children = self
            .game
            .position
            .legal_moves()
            .into_iter()
            .map(|m| RefCell::new(Node::new(self.game.play(&m))))
            .collect();
        self.children.shuffle(&mut rng);
    }

    fn _weighted_simulation(&self) -> Score {
        let mut result = Outcome::Draw;
        let mut game = self.game.clone();
        for _ in 0..200 {
            if game.position.is_game_over() {
                result = game.position.outcome().unwrap();
                break;
            }
            let resulting_games: Vec<Game> = game
                .position
                .legal_moves()
                .iter()
                .map(|m| game.play(m))
                .collect();
            let weights: Vec<f64> = resulting_games
                .iter()
                .map(|g| Evaluation::evaluate(&g, game.position.turn()))
                .collect();
            let max_weight = weights.iter().cloned().fold(0.0, f64::max);
            if max_weight == 0.0 {
                result = Outcome::Decisive {
                    winner: !game.position.turn(),
                };
                break;
            } else if max_weight == 1.0 {
                result = Outcome::Decisive {
                    winner: game.position.turn(),
                };
                break;
            } else {
                let dist = WeightedIndex::new(&weights).unwrap();
                let mut rng = rand::thread_rng();
                game = resulting_games[dist.sample(&mut rng)].clone();
            }
        }
        Score::new(&result)
    }

    fn random_simulation(&self) -> Score {
        let mut result = Outcome::Draw;
        let mut game = self.game.clone();
        let mut agent = agent::random_chess_agent();
        for _ in 0..200 {
            if game.position.is_game_over() {
                result = game.position.outcome().unwrap();
                break;
            }
            game = agent.take_turn(game);
        }
        Score::new(&result)
    }
}

pub struct MctsAgent {
    color: Color,
    head: RefCell<Node>,
}

impl MctsAgent {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            head: RefCell::new(Node::default()),
        }
    }

    fn update_head(&mut self, game: Game) {
        let mut next_head = None;
        {
            let head = self.head.borrow();
            for i in 0..head.children.len() {
                if head.children[i].borrow().game.hash == game.hash {
                    next_head = Some(RefCell::new(head.children[i].take()));
                    break;
                }
            }
        }
        match next_head {
            None => self.head = RefCell::new(Node::new(game)),
            Some(next_head) => self.head = next_head,
        };
    }

    fn update_simulations(node: &RefCell<Node>) -> Score {
        let score = if node.borrow().game.position.is_game_over() {
            Score::new(&node.borrow().game.position.outcome().unwrap())
        } else if node.borrow().is_leaf() {
            if node.borrow().score.games <= EXPANSION_MIN {
                node.borrow().random_simulation()
            } else {
                node.borrow_mut().expand();
                Self::update_simulations(&node.borrow().children[0])
            }
        } else {
            let color = node.borrow().game.position.turn();
            let simulations = node.borrow().score.games;
            node.borrow_mut().children.sort_by(|lhs, rhs| {
                Score::order_by_uct(
                    &lhs.borrow().score,
                    &rhs.borrow().score,
                    color,
                    simulations
                )
            });
            Self::update_simulations(&node.borrow().children[0])
        };
        node.borrow_mut().score += score;
        score
    }
}

impl ChessAgent for MctsAgent {
    fn take_turn(&mut self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
        self.update_head(game.clone());
        for _i in 0..MAX_SIMULATIONS {
            //println!("{}", _i);
            MctsAgent::update_simulations(&self.head);
        }
        self.head
            .borrow_mut()
            .children
            .sort_by(|lhs, rhs| Score::order_by_games(&lhs.borrow().score, &rhs.borrow().score));
        for child in self.head.borrow().children.iter() {
            println!("Score: {:?}", child.borrow().score);
        }
        println!("Size: {}", self.head.borrow()._len());
        let first_child = self.head.borrow().children[0].take();
        self.head = RefCell::new(first_child);
        self.head.borrow().game.clone()
    }
}
