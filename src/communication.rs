use crate::engine::score::Score;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub type Money = u32;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum RequestTable {
    RequestTable,
    Table(TableRequest),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct TableRequest {
    pub n_players: usize,
    pub small_blind: Money,
    pub big_blind: Money,
    pub stack: Money,
    pub game_type: GameType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameType {
    NoLimit,
    FixedLimit,
    PotLimit,
}

/// the callback that is used to communicate the game state from the engine to
/// the api.
pub trait Callback {
    fn callback(&mut self, message: Message) -> Result<Response, Error>;
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

/// Cards struct represents playing card.
/// rank has range 2-14(aces high) but when evaluating straights includes 1(aces low).
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
}

/// Response from the callback.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    // generic meaningless response
    Ack,
    // describes the player's move
    Action(PlayerAction),
}

/// Message sent to the callback
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    // game updates that require no response
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
    GameOver,
    // inform player of current game state and request a PlayerAction response
    RequestAction {
        player: usize,
        bets: Vec<Option<Money>>,
        pot: Money,
    },
    /// The offending player's id is passed as well so punishment can be served.
    Error(Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub player: usize,
    pub error: ErrorMessage,
}

/// Everything that can go wrong and should be messaged to the players.
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorMessage {
    InvalidResponse,
    BetNotAllowed,
    WebSocketError,
}

/// All the actions at the disposal of the player.
#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerAction {
    Fold,
    Call,
    Raise(Money),
}
