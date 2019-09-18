extern crate rand;
use rand::seq::SliceRandom;
use rand::thread_rng;

fn main() {
    println!("Hello, world!");
}

pub struct Table {
    game_type: GameType,
    n_players: usize,
    blind: Bet,
    dealer: usize,
    holes: Vec<Option<(Card, Card)>>,
    table_cards: Vec<Card>,
    deck: Deck,
    stacks: Vec<Bet>,
    pot: Bet,
}

impl Table {
    pub fn new(game_type: GameType, blind: Bet, stacks: Vec<Bet>) -> Table {
        Table {
            game_type,
            n_players: stacks.len(),
            blind,
            dealer: 0,
            pot: NOBET,
            holes: Vec::with_capacity(stacks.len()),
            table_cards: Vec::with_capacity(5),
            deck: Deck::new(),
            stacks,
        }
    }

    pub fn play_round(&mut self) {
        self.deck = Deck::new();
        self.table_cards.clear();
        self.pot = NOBET;
        self.holes = (0..self.n_players)
            .map(|_| Some((self.deck.draw(), self.deck.draw())))
            .collect();
        self.dealer = (self.dealer + 1) % self.n_players;
        // TODO pre-flop betting
        for _ in 0..3 {
            self.table_cards.push(self.deck.draw());
        }
        // TODO flop betting
        self.table_cards.push(self.deck.draw());
        // TODO river betting
        self.table_cards.push(self.deck.draw());
        // TODO turn betting
        // TODO showdown
        self.showdown()
    }

    fn showdown(&mut self) {
        if self.holes.len() == 1 {
            // TODO winner by default
        }
        let scores = self
            .holes
            .iter()
            .map(|o| match o {
                None => Score::folded(),
                Some((c1, c2)) => Score::calculate(vec![
                    *c1,
                    *c2,
                    self.table_cards[0],
                    self.table_cards[1],
                    self.table_cards[2],
                    self.table_cards[3],
                    self.table_cards[4],
                ]),
            })
            .collect::<Vec<Score>>();
        let max_score = scores.iter().max().unwrap();
        let splitters =
            scores
                .iter()
                .enumerate()
                .filter_map(|(i, s)| if s == max_score { Some(i) } else { None });
        // TODO pot is split amongst splitters
    }
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Default, Clone)]
struct Score {
    royal_flush: bool,
    // highest straight flush.
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
        let mut score = Score::default();
        cards.sort();
        cards.reverse();
        for i in 0..2 {
            // if 5 consecutive and colors match
            if cards[i] - cards[i + 4] == 5 && cards[i + 4] / 13 == cards[i] / 13 {
                match cards[i] % 13 {
                    12 => score.royal_flush = true,
                    i => score.straight_flush = i,
                };
                return score;
            }
        }

        let mut ranks = cards.iter().map(|card| card % 13).collect::<Vec<u8>>();
        ranks.sort();
        ranks.reverse();
        for i in 0..3 {
            // if 4 consecutive cards have the same rank (ordered by rank)
            if ranks[i] == ranks[i + 3] {
                score.four_of_a_kind = ranks[i];
                return score;
            }
        }

        // if 3 consec && 2 consec have same rank
        for i in 0..2 {
            if ranks[i] == ranks[i + 2] && ranks[i + 3] == ranks[i + 4] {
                score.full_house = (ranks[i], ranks[i + 3]);
                return score;
            }
            if ranks[i] == ranks[i + 1] && ranks[i + 2] == ranks[i + 4] {
                score.full_house = (ranks[i + 2], ranks[i]);
                return score;
            }
        }

        let mut suits = cards.iter().map(|card| card / 13).collect::<Vec<u8>>();
        suits.sort();
        suits.reverse();
        for i in 0..2 {
            // if five times same color
            if suits[i] == suits[i + 5] {
                score.flush = *cards
                    .iter()
                    .filter(|&card| card / 13 == suits[i])
                    .max()
                    .unwrap();
                return score;
            }
        }
        for i in 0..2 {
            if ranks[i] - ranks[i + 4] == 5 {
                score.straight = ranks[i];
                return score;
            }
        }
        for i in 0..4 {
            if ranks[i] == ranks[i + 2] {
                score.three_of_a_kind = ranks[i];
                return score;
            }
        }
        for i in 0..3 {
            if ranks[i] == ranks[i + 1] && ranks[i + 2] == ranks[i + 3] {
                score.two_pair = (ranks[i], ranks[i + 2]);
                return score;
            }
        }
        for i in 0..5 {
            if ranks[i] == ranks[i + 1] {
                score.one_pair = ranks[i];
                return score;
            }
        }

        score.high_card.copy_from_slice(&ranks[0..5]);
        score
    }
}

type Card = u8;
type Bet = u32;
const NOBET: u32 = 0u32;

// pub struct Game<T: PlayerAction> {
//     game_type: GameType,
//     blind: Bet,
//     players: Vec<Player<T>>,
//     game_state: GameState,
//     deck: Deck,
//     current_player: usize,
// }

// impl<T> Game<T>
// where T: PlayerAction+ Copy{
//     pub fn new(game_type: GameType, blind: Bet, actions: Vec<T>, dealer: usize)  -> Game<T> {
//         debug_assert!(actions.len()*2+3<=52); // TODO check this in api

//         let mut deck = Deck::new();
//         let players = actions.iter().enumerate()
//             .map(|(i, &action)| match i-dealer {
//                 1 => Player::new((deck.draw(), deck.draw()), blind, action),
//                 2 => Player::new((deck.draw(), deck.draw()), 2*blind, action),
//                 _ => Player::new((deck.draw(), deck.draw()), 0, action),
//             })
//             .collect::<Vec<Player<T>>>();
//         Game {
//             game_type,
//             blind,
//             game_state: GameState::PreFlop,
//             deck,
//             current_player: (dealer + 2) % players.len(),
//             players,
//         }
//     }

//     pub fn next(&mut self) {
//         match self.game_state {
//             GameState::PreFlop => {},
//             _ => {},
//         };
//     }
// }

pub enum GameType {
    NoLimit,
    FixedLimit,
    PotLimit,
}

// pub trait PlayerAction {
//     fn act<T: PlayerAction>(player: &Player<T>) -> Action;
// }

// pub enum Action {
//     Fold,
//     Call,
//     Raise(Bet),
// }

// struct Player<T: PlayerAction> {
//     hand: Option<(Card, Card)>,
//     bet: Bet,
//     action: T,
//     stock: Bet,
// }

// impl<T> Player<T>
// where
//     T: PlayerAction,
// {
//     fn new(action: T, stock: Bet) -> Player<T> {
//         Player {
//             hand: None,
//             bet: NoBet,
//             action,
//             stock,
//         }
//     }

//     fn deal(&mut self, hand: (Card, Card), blind: Bet) {
//         self.hand = Some(hand);
//         self.bet = blind;
//     }
// }

struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new() -> Deck {
        let mut cards = (0..52).map(|i| i as Card).collect::<Vec<Card>>();
        cards.as_mut_slice().shuffle(&mut thread_rng());
        Deck { cards }
    }

    fn draw(&mut self) -> Card {
        self.cards.remove(0)
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
