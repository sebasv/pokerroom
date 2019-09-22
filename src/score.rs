use crate::communication::{Card, Suit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Ord, Eq, PartialEq, PartialOrd, Default, Clone, Copy, Debug, Serialize, Deserialize)]
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
    pub fn folded() -> Score {
        Score::default()
    }

    pub fn calculate(mut cards: Vec<Card>) -> Score {
        // TODO count ace both as high and low in straights
        let mut score = Score::default();
        cards.sort();
        cards.reverse();

        // add ace as low ace as well, only for straights
        let straight_cards = {
            let mut vec = cards.clone();
            vec.append(
                &mut cards
                    .iter()
                    .filter_map(|c| {
                        if c.rank == 14 {
                            Some(Card {
                                suit: c.suit,
                                rank: 1,
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            );
            vec.sort();
            vec.reverse();
            vec
        };

        for i in 0..straight_cards.len() - 5 {
            // if 5 consecutive and colors match
            if straight_cards
                .iter()
                .skip(i)
                .take(5)
                .enumerate()
                .all(|(j, c)| j as u8 + c.rank == straight_cards[i].rank)
                && straight_cards[i + 4].suit == straight_cards[i].suit
            {
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
                map.entry(*rank).and_modify(|i| *i += 1).or_insert(1);
            }
            map
        };
        let high_triple = rank_counts.iter().fold(None, |acc, (rank, count)| {
            if *count == 3 && Some(rank) > acc {
                Some(rank)
            } else {
                acc
            }
        });
        let high_pair = rank_counts.iter().fold(None, |acc, (rank, count)| {
            if *count >= 2 && Some(rank) > acc && Some(rank) != high_triple {
                Some(rank)
            } else {
                acc
            }
        });
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
                    .unwrap()
                    .rank;
                return score;
            }
        }

        let straight_ranks = {
            let mut vec = straight_cards
                .iter()
                .map(|card| card.rank)
                .collect::<Vec<u8>>();
            vec.sort();
            vec.reverse();
            vec
        };
        for i in 0..straight_ranks.len() - 5 {
            if straight_ranks
                .iter()
                .skip(i)
                .take(5)
                .enumerate()
                .all(|(j, r)| *r + j as u8 == straight_ranks[0])
            {
                score.straight = straight_ranks[i];
                return score;
            }
        }

        if let Some(triple) = high_triple {
            score.three_of_a_kind = *triple;
            return score;
        }

        let low_pair = rank_counts.iter().fold(None, |acc, (rank, count)| {
            if *count >= 2
                && Some(rank) > acc
                && Some(rank) != high_triple
                && Some(rank) != high_pair
            {
                Some(rank)
            } else {
                acc
            }
        });

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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_score_royal_flush() {
        let score = Score {
            royal_flush: true,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 12,
            },
            Card {
                suit: Suit::Hearts,
                rank: 11,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 8,
            },
            Card {
                suit: Suit::Hearts,
                rank: 10,
            },
            Card {
                suit: Suit::Hearts,
                rank: 13,
            },
            Card {
                suit: Suit::Hearts,
                rank: 14,
            },
        ]);
        assert_eq!(calculated, score);
    }
    #[test]
    fn test_score_straight_flush() {
        let score = Score {
            royal_flush: false,
            straight_flush: 11,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 11,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 8,
            },
            Card {
                suit: Suit::Hearts,
                rank: 10,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Hearts,
                rank: 6,
            },
            Card {
                suit: Suit::Hearts,
                rank: 5,
            },
        ]);
        assert_eq!(calculated, score);
    }
    #[test]
    fn test_score_four_of_a_kind() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 9,
            full_house: (0, 0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Spades,
                rank: 2,
            },
            Card {
                suit: Suit::Spades,
                rank: 9,
            },
            Card {
                suit: Suit::Clubs,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 8,
            },
            Card {
                suit: Suit::Spades,
                rank: 7,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
        ]);
        assert_eq!(calculated, score);
    }

    #[test]
    fn test_score_full_house() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (9, 2),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 2,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Clubs,
                rank: 9,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 2,
            },
            Card {
                suit: Suit::Hearts,
                rank: 6,
            },
        ]);
        assert_eq!(calculated, score);
    }

    #[test]
    fn test_score_flush() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 12,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 2,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 4,
            },
            Card {
                suit: Suit::Hearts,
                rank: 12,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Hearts,
                rank: 8,
            },
            Card {
                suit: Suit::Hearts,
                rank: 5,
            },
        ]);
        assert_eq!(calculated, score);
    }

    #[test]
    fn test_score_bland_straight() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 0,
            straight: 10,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 2,
            },
            Card {
                suit: Suit::Hearts,
                rank: 8,
            },
            Card {
                suit: Suit::Clubs,
                rank: 9,
            },
            Card {
                suit: Suit::Spades,
                rank: 10,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 6,
            },
            Card {
                suit: Suit::Hearts,
                rank: 5,
            },
        ]);
        assert_eq!(calculated, score);
    }

    #[test]
    fn test_score_three_of_a_kind() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 9,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 2,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 9,
            },
            Card {
                suit: Suit::Clubs,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 10,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 6,
            },
        ]);
        assert_eq!(calculated, score);
    }

    #[test]
    fn test_score_two_pair() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (9, 7),
            one_pair: 0,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 2,
            },
            Card {
                suit: Suit::Clubs,
                rank: 7,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Spades,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 3,
            },
            Card {
                suit: Suit::Hearts,
                rank: 5,
            },
        ]);
        assert_eq!(calculated, score);
    }

    #[test]
    fn test_score_one_pair() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 9,
            high_card: [0, 0, 0, 0, 0],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 2,
            },
            Card {
                suit: Suit::Clubs,
                rank: 8,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Spades,
                rank: 9,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 3,
            },
            Card {
                suit: Suit::Hearts,
                rank: 5,
            },
        ]);
        assert_eq!(calculated, score);
    }

    #[test]
    fn test_score_high_card() {
        let score = Score {
            royal_flush: false,
            straight_flush: 0,
            four_of_a_kind: 0,
            full_house: (0, 0),
            flush: 0,
            straight: 0,
            three_of_a_kind: 0,
            two_pair: (0, 0),
            one_pair: 0,
            high_card: [12, 11, 10, 9, 7],
        };
        let calculated = Score::calculate(vec![
            Card {
                suit: Suit::Hearts,
                rank: 12,
            },
            Card {
                suit: Suit::Clubs,
                rank: 11,
            },
            Card {
                suit: Suit::Hearts,
                rank: 9,
            },
            Card {
                suit: Suit::Spades,
                rank: 10,
            },
            Card {
                suit: Suit::Hearts,
                rank: 7,
            },
            Card {
                suit: Suit::Diamonds,
                rank: 3,
            },
            Card {
                suit: Suit::Hearts,
                rank: 5,
            },
        ]);;
        assert_eq!(calculated, score);
    }
}
