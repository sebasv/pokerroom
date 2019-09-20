mod engine;
mod api;


use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};
use std::thread;
/*  TODO
* test other rules
* docs
**/

const CONNECTION: &str = "ws://127.0.0.1:2794";


fn main() {
    engine_test();
    api_test();
}

fn engine_test() {

#[derive(Clone, Copy)]
struct DumbCallback {

}

impl engine::ActionCallback for DumbCallback {
    fn callback(&mut self, message: engine::Message) -> engine::Message {

    // Flop(Card,Card,Card),
    // River(Card),
    // Turn(Card),
    // Showdown{score: Score, pot: Money, players: Vec<usize>},
    // Player{id: usize, action: PlayerAction},
    // RequestAction(usize),
    // Error(ErrorMessage),
    // Ack,
        match message {
            engine::Message::RequestAction(id) => engine::Message::Player{id, action: engine::PlayerAction::Call},
            other => {
                println!("{:?}", other);
                engine::Message::Ack
            }
        }
    }
}

    let callback = DumbCallback{};
    let mut table = engine::Table::new(engine::GameType::NoLimit, 1, vec![100,100,100], callback);
    table.play_until_end();
}


fn api_test() {
    let server = thread::spawn(move || {
        api::run_server("127.0.0.1:2794", 6);    
    });

    for _ in 0..6 {
        thread::spawn(move || {
            let mut client = ClientBuilder::new(CONNECTION)
                .unwrap()
                .add_protocol("rust-websocket")
                .connect_insecure()
                .unwrap();
                loop {
                    match client.recv_message() {
                        Ok(OwnedMessage::Text(ref s)) if s == "?action" => {
                            client.send_message(&Message::text("call"));
                        },
                        Ok(OwnedMessage::Text(ref s)) => {
                            println!("{}", s);
                        },
                        _ => break
                    }
                }
        });
    }

    server.join();
}
