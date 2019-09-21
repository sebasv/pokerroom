use crate::engine::score::Score;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub type Money = u32;

pub enum GameType {
    NoLimit,
    FixedLimit,
    PotLimit,
}

#[derive(
    Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, FromPrimitive, Serialize, Deserialize,
)]
pub enum Suit {
    /*(♥)*/ Hearts,
    /*(♠)*/ Spades,
    /*(♣)*/ Clubs,
    /*(♦)*/ Diamonds,
}

/// Cards struct represents card. It would be slightly better to replace suits with an enum.
/// Suit has range 2-14(aces high) but when evaluating straights includes 1(aces low).
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ack,
    Action(PlayerAction),
}
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Hole {
        player: usize,
        cards: (Card, Card),
    },
    Flop(Card, Card, Card),
    River(Card),
    Turn(Card),
    Showdown {
        score: Score,
        pot: Money,
        players: Vec<usize>,
        stacks: Vec<Money>,
    },
    RequestAction {
        player_id: usize,
        bets: Vec<Option<Money>>,
        pot: Money,
    },
    Error(ErrorMessage),
    GameOver,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorMessage {
    InvalidResponse,
    BetNotAllowed,
}

pub trait Callback {
    fn callback(&mut self, message: Message) -> Response;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerAction {
    Fold,
    Call,
    Raise(Money),
}
