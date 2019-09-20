use rand::seq::SliceRandom;
use rand::thread_rng;

use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;


use std::collections::HashMap;

/*  TODO
* test other rules
* bla
**/

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, FromPrimitive)]
pub enum Suit {
    /*(♥)*/Hearts,
    /*(♠)*/Spades,
    /*(♣)*/Clubs,
    /*(♦)*/Diamonds,
}

/// Cards struct represents card. It would be slightly better to replace suits with an enum.
/// Suit has range 2-14(aces high) but when evaluating straights includes 1(aces low).
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub struct Card {
    rank: u8,
    suit: Suit,
}
type Money = u32;
const ZERO_MONEY: u32 = 0u32;

pub enum GameType {
    NoLimit,
    FixedLimit,
    PotLimit,
}

pub struct Table<T>
where T: ActionCallback {
    game_type: GameType,
    blind: Money,
    dealer: usize,
    players: Vec<Player>,
    callback: T
}

/// the game-manager. players register by creating a new table, which then
/// automatically plays rounds until only one player is left. 
impl<T> Table<T>
where T: ActionCallback {
    pub fn new(game_type: GameType, blind: Money, players: Vec<Money>, callback: T) -> Table<T> {
        let players = players.iter().map(|stack| {
            Player::new(*stack)
        }).collect();
        Table {
            game_type,
            blind,
            dealer: 0,
            players,
            callback,
        }
    }

    pub fn play_until_end(&mut self) {
        while self.players.iter().filter(|p|p.active()).count() > 1 {
            self.play_round();
        }
        self.callback.callback(Message::GameOver);
    }

    pub fn play_n_rounds(&mut self, n: usize) {
        for _ in 0..n {
            self.play_round();
        }
        self.callback.callback(Message::GameOver);
    }

    pub fn play_round(&mut self) {
        let mut deck = Deck::new();
        let mut pot = ZERO_MONEY;
        let mut table_cards = Vec::new();
        let n = self.players.len();

        for (i, player) in &mut self.players.iter_mut().enumerate() {
            match (i + n - self.dealer) % n {
                1 if player.active() => player.call(self.blind),
                2 if player.active() => player.call(self.blind * 2),
                _ => {},
            } 
            player.hole_cards = if player.active() {
                let cards = (deck.draw(), deck.draw());
                self.callback.callback(Message::Hole(i, cards.0, cards.1));
                 Some(cards)
            } else {
                None
            }
        }
        // pre-flop 
        pot += self.betting_round((self.dealer+3)%n);
        for _ in 0..3 {
            table_cards.push(deck.draw());
        }
        self.callback.callback(Message::Flop(table_cards[0], table_cards[1], table_cards[2]));

        //  river
        pot += self.betting_round(self.dealer+1);
        table_cards.push(deck.draw());
        self.callback.callback(Message::River(table_cards[3]));

        //  turn 
        pot += self.betting_round(self.dealer+1);
        table_cards.push(deck.draw());
        self.callback.callback(Message::Turn(table_cards[4]));

        // showdown
        pot += self.betting_round(self.dealer+1);
        let (splitters, score) = self.showdown(&table_cards);

        //  divide pot over winners
        let share = pot / splitters.len() as Money;
        for splitter in &splitters {
            self.players[*splitter].stack += share;
        }
        self.callback.callback(Message::Showdown{score, pot, players: splitters});
        self.dealer = (self.dealer + 1) % self.players.len();
        println!("{:?}", self.players.iter().map(|p| p.stack).collect::<Vec<Money>>());
    }

    fn betting_round(&mut self, first_player: usize) -> Money {
        let n = self.players.len();
        let mut max_bet = self.players.iter().map(|p| p.bet).max().unwrap();
        let mut min_betsize = self.blind*2;
        for i in (0..self.players.len()).map(|i| (i+first_player) % n) {
            if self.players[i].can_bet(max_bet) {
                self.bet(i, max_bet, min_betsize);
                min_betsize = min_betsize.max(self.players[i].bet - max_bet);
                max_bet = max_bet.max(self.players[i].bet);
            }
        }

        // continue until everyone equal, all-in or folded
        let mut old_max_bet = ZERO_MONEY;

        while old_max_bet < max_bet {
            old_max_bet = max_bet;
            for i in (0..self.players.len()).map(|i| (i+first_player) % n) {
                if self.players[i].can_bet(max_bet) {
                    self.bet(i, max_bet, min_betsize);
                    min_betsize = min_betsize.max(self.players[i].bet - max_bet);
                    max_bet = max_bet.max(self.players[i].bet);
                }
            }
        }

        self.players.iter_mut().map(|p| p.yield_bet()).sum::<Money>()
    }

    fn bet(&mut self, player_index: usize, max_bet: Money, min_betsize: Money) {
        match self.callback.callback(Message::RequestAction(player_index)) {
            Message::Player{action: PlayerAction::Fold, ..} => {
                self.players[player_index].fold();
            },
            Message::Player{action: PlayerAction::Call, ..} => {
                self.players[player_index].call(max_bet);
            },
            Message::Player{action: PlayerAction::Raise(new_bet), ..} => {
                if new_bet - max_bet < min_betsize || self.players[player_index].raise(new_bet).is_err() {
                    self.callback.callback(Message::Error(ErrorMessage::BetNotAllowed));
                    self.bet(player_index, max_bet, min_betsize);
                }
            },
            _ => {
                self.callback.callback(Message::Error(ErrorMessage::InvalidResponse));
                self.bet(player_index, max_bet, min_betsize);
            },
        }

    }

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
        let splitters =
            scores
                .iter()
                .enumerate()
                .filter_map(|(i, s)| if s == max_score { Some(i) } else { None }).collect();

        (splitters, *max_score)
    }
}

#[derive(Debug)]
pub enum Message {
    Hole(usize, Card, Card),
    Flop(Card, Card, Card),
    River(Card),
    Turn(Card),
    Showdown{score: Score, pot: Money, players: Vec<usize>},
    Player{id: usize, action: PlayerAction},
    RequestAction(usize),
    Error(ErrorMessage),
    Ack,
    GameOver,
}

#[derive(Debug)]
pub enum ErrorMessage {
    InvalidResponse,
    BetNotAllowed,
}

pub trait ActionCallback{
    fn callback(&mut self, message: Message) -> Message;
}

#[derive(Debug)]
pub enum PlayerAction {
    Fold,
    Call,
    Raise(Money)
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Default, Clone, Copy, Debug)]
pub struct Score {
    royal_flush: bool,
    // aces-high straight flush.
    straight_flush: u8,
    // Any straight with all five cards of the same suit.
    four_of_a_kind: u8,
    // Any four cards of the same rank. If two players share the same Four of a Kind (on the board), the bigger fifth card (the "kicker") decides who wins the pot.
    full_house: (u8, u8),
    // Any three cards of the same rank together with any two cards of the same rank. Our example shows "Aces full of Kings" and it is a bigger full house than "Kings full of Aces."
    flush: u8,
    // Any five cards of the same suit (not consecutive). The highest card of the five determines the rank of the flush. Our example shows an Ace-high flush, which is the highest possible.
    straight: u8,
    // Any five consecutive cards of different suits. Aces can count as either a high or a low card. Our example shows a five-high straight, which is the lowest possible straight.
    three_of_a_kind: u8,
    // Any three cards of the same rank. Our example shows three-of-a-kind Aces, with a King and a Queen as side cards - the best possible three of a kind.
    two_pair: (u8, u8),
    // Any two cards of the same rank together with another two cards of the same rank. Our example shows the best possible two-pair, Aces and Kings. The highest pair of the two determines the rank of the two-pair.
    one_pair: u8,
    // Any two cards of the same rank. Our example shows the best possible one-pair hand.
    high_card: [u8; 5],
    // Any hand not in the above-mentioned hands. Our example shows the best possible high-card hand.
}

impl Score {
    fn folded() -> Score {
        Score::default()
    }

    fn calculate(mut cards: Vec<Card>) -> Score {
        // TODO count ace both as high and low in straights
        let mut score = Score::default();
        cards.sort();
        cards.reverse();

        // add ace as low ace as well, only for straights
        let straight_cards = {
            let mut vec = cards.clone();
            vec.append(&mut cards.iter().filter_map(|c| if c.rank==14 {Some(Card{suit: c.suit, rank: 1})} else {None}).collect());
            vec.sort();
            vec.reverse();
            vec
        };
        
        for i in 0..straight_cards.len()-5 {
            // if 5 consecutive and colors match
            if straight_cards.iter().skip(i).take(5).enumerate().all(|(j, c)| j as u8 + c.rank == straight_cards[i].rank)
            && straight_cards[i + 4].suit == straight_cards[i].suit {
                match straight_cards[i].rank {
                    14 => score.royal_flush = true,
                    i => score.straight_flush = i,
                };
                return score;
            }
        }

        let ranks = {
            let mut vec = cards.iter().map(|card| card.rank).collect::<Vec<u8>>();
            vec.sort();
            vec.reverse();
            vec
        };
        for i in 0..3 {
            // if 4 consecutive cards have the same rank (ordered by rank)
            if ranks[i] == ranks[i + 3] {
                score.four_of_a_kind = ranks[i];
                return score;
            }
        }

        // if 3 consec && 2 consec have same rank
        let rank_counts = {
            let mut map = HashMap::new();
            for rank in &ranks {
                map.entry(*rank).and_modify(|i| *i+=1 ).or_insert(1);
            }
            map
        };
        let high_triple = rank_counts.iter().fold(None, |acc, (rank, count)| {
            if *count == 3 && Some(rank) > acc { Some(rank)} else {acc}
        } );
        let high_pair  = rank_counts.iter().fold(None, |acc, (rank, count)| {
            if *count >= 2 && Some(rank) > acc && Some(rank) != high_triple { Some(rank)} else {acc}
        } );
        if let (Some(triple), Some(pair)) = (high_triple, high_pair) {
                score.full_house = (*triple, *pair);
                return score;
        }

        let suits = {
            let mut vec = cards.iter().map(|card| card.suit).collect::<Vec<Suit>>();
            vec.sort();
            vec.reverse();
            vec
        };
        for i in 0..2 {
            // if five times same color
            if suits[i] == suits[i + 5] {
                score.flush = cards
                    .iter()
                    .filter(|&card| card.suit == suits[i])
                    .max()
                    .unwrap().rank;
                return score;
            }
        }


        let straight_ranks = {
            let mut vec = straight_cards.iter().map(|card| card.rank).collect::<Vec<u8>>();
            vec.sort();
            vec.reverse();
            vec
        };
        for i in 0..straight_ranks.len()-5 {
            if straight_ranks.iter().skip(i).take(5).enumerate().all(|(j, r)| *r + j as u8 == straight_ranks[0]) {
                score.straight = straight_ranks[i];
                return score;
            }
        }

        if let Some(triple) = high_triple {
            score.three_of_a_kind = *triple;
            return score;
        }

        let low_pair  = rank_counts.iter().fold(None, |acc, (rank, count)| {
            if *count >= 2 && Some(rank) > acc && Some(rank) != high_triple && Some(rank) != high_pair { Some(rank)} else {acc}
        } );


        if let Some(pair) = high_pair {
            if let Some(other_pair) = low_pair {
                score.two_pair = (*pair, *other_pair)
            } else {
                score.one_pair = *pair;
            }
            return score;
        }

        score.high_card.copy_from_slice(&ranks[0..5]);
        score
    }
}

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

    fn can_bet(&self, bet: Money) -> bool {
        self.stack + self.bet >= bet && self.hole_cards.is_some()
    }

    fn active(&self) -> bool {
        self.stack > ZERO_MONEY || self.bet > ZERO_MONEY
    }

    fn raise(&mut self, bet: Money) -> Result<(),()> {
        if bet <= self.stack {
            self.stack -= bet;
            self.bet += bet;
            Ok(())
        } else {
            Err(())
        }
    }

    /// if calling on more than you have, you are all in
    fn call(&mut self, bet: Money)  {
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
        let mut cards = (0..52).map(|i| Card{suit: Suit::from_u8(i/13).unwrap(), rank: 2+i%13}).collect::<Vec<Card>>();
        cards.as_mut_slice().shuffle(&mut thread_rng());
        Deck { cards }
    }

    fn draw(&mut self) -> Card {
        self.cards.remove(0)
    }
}
 
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_score_royal_flush() {
        let score = Score{
            royal_flush: true,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 12}, 
            Card{suit: Suit::Hearts, rank: 11}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 8}, 
            Card{suit: Suit::Hearts, rank: 10}, 
            Card{suit: Suit::Hearts, rank: 13}, 
            Card{suit: Suit::Hearts, rank: 14}
            ]);
        assert_eq!(calculated, score);
    }
    #[test]
    fn test_score_straight_flush() {
        let score = Score{
            royal_flush: false,
            straight_flush: 11,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 11}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 8}, 
            Card{suit: Suit::Hearts, rank: 10}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Hearts, rank: 6}, 
            Card{suit: Suit::Hearts, rank: 5}
            ]);
        assert_eq!(calculated, score);
    }
    #[test]
    fn test_score_four_of_a_kind() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 9,
            full_house: (0,0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Spades, rank: 2}, 
            Card{suit: Suit::Spades, rank: 9}, 
            Card{suit: Suit::Clubs, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 8}, 
            Card{suit: Suit::Spades, rank: 7}, 
            Card{suit: Suit::Diamonds, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 9}
            ]);
        assert_eq!(calculated, score);
    }
        
    #[test]
    fn test_score_full_house() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (9,2),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 2}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Diamonds, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Clubs, rank: 9}, 
            Card{suit: Suit::Diamonds, rank: 2}, 
            Card{suit: Suit::Hearts, rank: 6}
            ]);
        assert_eq!(calculated, score);
    }
        
    #[test]
    fn test_score_flush() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 12,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 2}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 4}, 
            Card{suit: Suit::Hearts, rank: 12}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Hearts, rank: 8}, 
            Card{suit: Suit::Hearts, rank: 5}
            ]);
        assert_eq!(calculated, score);
    }
        
    #[test]
    fn test_score_bland_straight() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 0,
            straight: 10,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 2}, 
            Card{suit: Suit::Hearts, rank: 8}, 
            Card{suit: Suit::Clubs, rank: 9}, 
            Card{suit: Suit::Spades, rank: 10}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Diamonds, rank: 6}, 
            Card{suit: Suit::Hearts, rank: 5}
            ]);
        assert_eq!(calculated, score);
    }
        
    #[test]
    fn test_score_three_of_a_kind() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 9,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 2}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Diamonds, rank: 9}, 
            Card{suit: Suit::Clubs, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 10}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Diamonds, rank: 6}
            ]);
        assert_eq!(calculated, score);
    }
        
    #[test]
    fn test_score_two_pair() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (9,7),
            one_pair: 0,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 2}, 
            Card{suit: Suit::Clubs, rank: 7}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Spades, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Diamonds, rank: 3}, 
            Card{suit: Suit::Hearts, rank: 5}
            ]);
        assert_eq!(calculated, score);
    }
        
    #[test]
    fn test_score_one_pair() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 9,
            high_card: [0,0,0,0,0],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 2}, 
            Card{suit: Suit::Clubs, rank: 8}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Spades, rank: 9}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Diamonds, rank: 3}, 
            Card{suit: Suit::Hearts, rank: 5}
            ]);
        assert_eq!(calculated, score);
    }
        
    #[test]
    fn test_score_high_card() {
        let score = Score{
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0,0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0,0),
            one_pair: 0,
            high_card: [12,11,10,9,7],        
        };
        let calculated = Score::calculate(vec![
            Card{suit: Suit::Hearts, rank: 12}, 
            Card{suit: Suit::Clubs, rank: 11}, 
            Card{suit: Suit::Hearts, rank: 9}, 
            Card{suit: Suit::Spades, rank: 10}, 
            Card{suit: Suit::Hearts, rank: 7}, 
            Card{suit: Suit::Diamonds, rank: 3}, 
            Card{suit: Suit::Hearts, rank: 5}
            ]);;
        assert_eq!(calculated, score);
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
