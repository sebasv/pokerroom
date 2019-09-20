mod engine;
use engine::{ActionCallback, ErrorMessage, Message, PlayerAction, Score, Table, GameType};
mod api;


/*  TODO
* test scoring rules
* test other rules
* bla
**/

#[derive(Clone, Copy)]
struct DumbCallback {

}

impl ActionCallback for DumbCallback {
    fn callback(&self, message: Message) -> Message {

    // Flop(Card,Card,Card),
    // River(Card),
    // Turn(Card),
    // Showdown{score: Score, pot: Money, players: Vec<usize>},
    // Player{id: usize, action: PlayerAction},
    // RequestAction(usize),
    // Error(ErrorMessage),
    // Ack,
        match message {
            Message::RequestAction(id) => Message::Player{id, action: PlayerAction::Call},
            other => {
                println!("{:?}", other);
                Message::Ack
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
    let callback = DumbCallback{};
    let mut table = Table::new(GameType::NoLimit, 1, vec![100,100,100], callback);
    table.play_n_rounds(1000);
}
