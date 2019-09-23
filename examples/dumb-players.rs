use ::poker::{
    run_server, GameType, Message as PokerMessage, PlayerAction, RequestTable, Response,
    TableRequest,
};
use std::thread;
use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

const CONNECTION: &str = "ws://127.0.0.1:2794";

fn main() -> Result<(), ()> {
    let server = thread::spawn(move || {
        run_server("127.0.0.1:2794");
    });

    let n_players = 1;

    for _ in 0..n_players {
        thread::spawn(move || {
            run_player();
        });
    }
    // do not end program
    server.join().or(Err(()))
}

fn run_player() {
    let mut client = ClientBuilder::new(CONNECTION)
        .unwrap()
        .add_protocol("rust-websocket")
        .connect_insecure()
        .unwrap();

    let serialized = serde_json::to_string(&RequestTable::Table(TableRequest {
        n_players: 1,
        small_blind: 1,
        big_blind: 2,
        stack: 100,
        game_type: GameType::NoLimit,
    }))
    .unwrap();
    println!("{:?}", serialized);
    client.send_message(&Message::text(serialized)).ok();

    let mut count = 0;
    while let Ok(msg) = client.recv_message() {
        count += 1;
        if count > 1000 {
            println!("Got a thousand messages, you get the point. Shutting down client");
            client.send_message(&Message::close()).ok();
            break;
        }
        match msg {
            OwnedMessage::Text(t) => {
                println!("{:?}", t);
                if let Ok(PokerMessage::RequestAction { .. }) =
                    serde_json::from_str::<PokerMessage>(&t)
                {
                    let serialized =
                        serde_json::to_string(&Response::Action(PlayerAction::Raise(2))).unwrap();
                    println!("{:?}", serialized);
                    let msg = Message::text(serialized);
                    client.send_message(&msg).ok();
                }
            }
            _ => break,
        }
    }
}
