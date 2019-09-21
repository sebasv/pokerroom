mod api;
mod engine;
mod communication;

use std::thread;
use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

/*  TODO
* test other rules
* docs
**/

const CONNECTION: &str = "ws://127.0.0.1:2794";

fn main() {
    let n_players = 1;
    let server = thread::spawn(move || {
        api::run_server("127.0.0.1:2794", n_players);
    });

    for _ in 0..n_players {
        thread::spawn(move || {
            let mut client = ClientBuilder::new(CONNECTION)
                .unwrap()
                .add_protocol("rust-websocket")
                .connect_insecure()
                .unwrap();
            let mut count = 0;
            while let Ok(msg) = client.recv_message() {
                match msg {
                    OwnedMessage::Text(t) => {
                        println!("{:?}", t);
                        if let Ok(communication::Message::RequestAction { .. }) =
                            serde_json::from_str::<communication::Message>(&t)
                        {
                            let serialized = serde_json::to_string(&communication::Response::Action(
                                communication::PlayerAction::Raise(2),
                            ))
                            .unwrap();
                            println!("{:?}", serialized);
                            let msg = Message::text(serialized);
                            client.send_message(&msg);
                        }
                    }
                    _ => break,
                }
            }
        });
    }

    server.join();
}
