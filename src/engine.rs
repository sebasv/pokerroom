use num_traits::FromPrimitive;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod score;
use crate::communication::{
    Callback, Card, ErrorMessage, GameType, Message, Money, PlayerAction, Response, Suit,
};
use score::Score;

/*  TODO
* test other rules
* make call and raise relative such that call==raise(0)
**/

/// chips are discrete, so money should be as well
const ZERO_MONEY: Money = 0;

pub struct Table<T>
where
    T: Callback,
{
    game_type: GameType,
    blind: Money,
    dealer: usize,
    players: Vec<Player>,
    callback: T,
}

/// the game-manager. players register by creating a new table, which then
/// automatically plays rounds until only one player is left.
impl<T> Table<T>
where
    T: Callback,
{
    pub fn new(game_type: GameType, blind: Money, players: Vec<Money>, callback: T) -> Table<T> {
        let players = players.iter().map(|stack| Player::new(*stack)).collect();
        Table {
            game_type,
            blind,
            dealer: 0,
            players,
            callback,
        }
    }

    /// Play rounds until only one player has a stack of chips left.
    pub fn play_until_end(&mut self) {
        while self.players.iter().filter(|p| p.active()).count() > 1 {
            if let Err(e) = self.play_round() {
                self.callback.callback(Message::Error(e)).ok();
                break;
            }
        }
        self.callback.callback(Message::GameOver).ok();
    }

    /// Play n rounds.
    pub fn play_n_rounds(&mut self, n: usize) {
        for _ in 0..n {
            if let Err(e) = self.play_round() {
                self.callback.callback(Message::Error(e)).ok();
                break;
            }
        }
        self.callback.callback(Message::GameOver).ok();
    }

    /// Play a single round.
    fn play_round(&mut self) -> Result<(), ErrorMessage> {
        let mut deck = Deck::new();
        let mut pot = ZERO_MONEY;
        let mut table_cards = Vec::new();
        let n = self.players.len();

        for (i, player) in &mut self.players.iter_mut().enumerate() {
            match (i + n - self.dealer) % n {
                1 if player.active() => player.call(self.blind),
                2 if player.active() => player.call(self.blind * 2),
                _ => {}
            }
            player.hole_cards = if player.active() {
                let cards = (deck.draw(), deck.draw());
                self.callback.callback(Message::Hole { player: i, cards })?;
                Some(cards)
            } else {
                None
            }
        }
        // pre-flop
        pot += self.betting_round((self.dealer + 3) % n, pot)?;
        for _ in 0..3 {
            table_cards.push(deck.draw());
        }
        self.callback.callback(Message::Flop(
            table_cards[0],
            table_cards[1],
            table_cards[2],
        ))?;

        //  river
        pot += self.betting_round(self.dealer + 1, pot)?;
        table_cards.push(deck.draw());
        self.callback.callback(Message::River(table_cards[3]))?;

        //  turn
        pot += self.betting_round(self.dealer + 1, pot)?;
        table_cards.push(deck.draw());
        self.callback.callback(Message::Turn(table_cards[4]))?;

        // showdown
        pot += self.betting_round(self.dealer + 1, pot)?;
        let (splitters, score) = self.showdown(&table_cards);

        //  divide pot over winners, bank takes change via integer division
        let share = pot / splitters.len() as Money;
        for splitter in &splitters {
            self.players[*splitter].stack += share;
        }
        self.callback.callback(Message::Showdown {
            score,
            pot,
            players: splitters,
            stacks: self.players.iter().map(|p| p.stack).collect(),
        })?;
        self.dealer = (self.dealer + 1) % self.players.len();
        Ok(())
    }

    /// A single round of poker consists of a series of betting rounds.
    /// These rules depend on the game type.
    fn betting_round(&mut self, first_player: usize, pot: Money) -> Result<Money, ErrorMessage> {
        let n = self.players.len();
        let mut max_bet = self.players.iter().map(|p| p.bet).max().unwrap();
        let mut min_betsize = self.blind * 2;
        for i in (0..self.players.len()).map(|i| (i + first_player) % n) {
            if self.players[i].can_bet(max_bet) {
                self.bet(i, max_bet, min_betsize, pot)?;
                min_betsize = min_betsize.max(self.players[i].bet - max_bet);
                max_bet = max_bet.max(self.players[i].bet);
            }
        }

        // continue until everyone equal, all-in or folded
        let mut old_max_bet = ZERO_MONEY;

        while old_max_bet < max_bet {
            old_max_bet = max_bet;
            for i in (0..self.players.len()).map(|i| (i + first_player) % n) {
                if self.players[i].can_bet(max_bet) {
                    self.bet(i, max_bet, min_betsize, pot)?;
                    min_betsize = min_betsize.max(self.players[i].bet - max_bet);
                    max_bet = max_bet.max(self.players[i].bet);
                }
            }
        }

        Ok(self
            .players
            .iter_mut()
            .map(|p| p.yield_bet())
            .sum::<Money>())
    }

    /// Request a player's action, verify this action is allowed within the
    /// rule set of the current game type, and update pot & table bets.
    fn bet(
        &mut self,
        player: usize,
        max_bet: Money,
        min_betsize: Money,
        pot: Money,
    ) -> Result<(), ErrorMessage> {
        match self.callback.callback(Message::RequestAction {
            player,
            bets: self
                .players
                .iter()
                .map(|p| if p.folded() { None } else { Some(p.bet) })
                .collect(),
            pot,
        }) {
            Ok(Response::Action(PlayerAction::Fold)) => {
                self.players[player].fold();
                Ok(())
            }
            Ok(Response::Action(PlayerAction::Call)) => {
                self.players[player].call(max_bet);
                Ok(())
            }
            Ok(Response::Action(PlayerAction::Raise(new_bet))) => {
                if new_bet - max_bet < min_betsize || self.players[player].raise(new_bet).is_err() {
                    Err(ErrorMessage::BetNotAllowed)
                } else {
                    Ok(())
                }
            }
            Ok(_) => Err(ErrorMessage::InvalidResponse),
            Err(e) => Err(e),
        }
    }

    /// Calculate the score of each player, determine the winning hand and the
    /// winners.
    fn showdown(&self, table_cards: &[Card]) -> (Vec<usize>, Score) {
        let scores = self
            .players
            .iter()
            .map(|p| match p.hole_cards {
                None => Score::folded(),
                Some((c1, c2)) => Score::calculate(vec![
                    c1,
                    c2,
                    table_cards[0],
                    table_cards[1],
                    table_cards[2],
                    table_cards[3],
                    table_cards[4],
                ]),
            })
            .collect::<Vec<Score>>();
        let max_score = scores.iter().max().unwrap();
        let splitters = scores
            .iter()
            .enumerate()
            .filter_map(|(i, s)| if s == max_score { Some(i) } else { None })
            .collect();

        (splitters, *max_score)
    }
}

/// Struct to manage the state of a player
struct Player {
    hole_cards: Option<(Card, Card)>,
    stack: Money,
    bet: Money,
}

impl Player {
    fn new(stack: Money) -> Player {
        Player {
            stack,
            bet: ZERO_MONEY,
            hole_cards: None,
        }
    }

    fn folded(&self) -> bool {
        self.hole_cards.is_none()
    }

    /// A player can bet more if their stack is sufficiently big and they
    /// haven't folded
    fn can_bet(&self, bet: Money) -> bool {
        self.stack + self.bet >= bet && !self.folded()
    }

    /// A player is active if they have chips left to play with
    fn active(&self) -> bool {
        self.stack + self.bet > ZERO_MONEY
    }

    /// Attempt to raise. A player can raise if their stack is sufficiently big.
    fn raise(&mut self, bet: Money) -> Result<(), ()> {
        if bet <= self.stack {
            self.stack -= bet;
            self.bet += bet;
            Ok(())
        } else {
            Err(())
        }
    }

    /// if calling on more than you have, you are all in
    fn call(&mut self, bet: Money) {
        if self.stack + self.bet > bet {
            self.stack -= bet - self.bet;
            self.bet = bet;
        } else {
            self.stack = ZERO_MONEY;
            self.bet = bet;
        }
    }

    fn fold(&mut self) {
        self.hole_cards = None;
    }

    /// A player yields their bet to the pot at the end of the betting round.
    fn yield_bet(&mut self) -> Money {
        let bet = self.bet;
        self.bet = ZERO_MONEY;
        bet
    }
}

struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new() -> Deck {
        let mut cards = (0..52)
            .map(|i| Card {
                suit: Suit::from_u8(i / 13).unwrap(),
                rank: 2 + i % 13,
            })
            .collect::<Vec<Card>>();
        cards.as_mut_slice().shuffle(&mut thread_rng());
        Deck { cards }
    }

    /// Draw a card. Will panic if the deck is empty, which should be
    /// impossible by the game rules.
    fn draw(&mut self) -> Card {
        self.cards.pop().expect("drew a card from an empty deck")
    }
}

/*
table min=20 big blind, table max= 100x big blind

* create new table
* join existing table
* room tracks tables
* one table consists of a number of deals (until all but 1 are broke)

no-limit:
minimum raise is last bet

fixed-limit:
2 bet increments
big blind=small bet
pre-flop&flop only small increment
turn&river only big increment
4 increments max

pot-limit:
can only double pot

table properties
*type; no-limit/pot-limit/fixed-limit
*blind;
*number of participants

game sequence:
* deal hole cards
* pre-flop
* deal flop
* flop bet
* deal river
* river bet
* deal turn
* turn bet
* showdown
*/
