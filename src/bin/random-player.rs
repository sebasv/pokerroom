use ::poker::{
    GameType, Message as PokerMessage, PlayerAction, RequestTable, Response, TableRequest,
};
use std::thread;
use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

const CONNECTION: &str = "ws://ws.sebastiaanvermeulen.nl/pokerroom";

fn main() {
    let n_players = 2;

    let threads = (0..n_players)
        .map(|i| {
            thread::spawn(move || {
                run_player(i);
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        thread.join().expect("child thread panicked");
    }
}

fn run_player(player: usize) {
    let mut client = ClientBuilder::new(CONNECTION)
        .expect("could not build connection")
        .add_protocol("rust-websocket")
        .connect_insecure()
        .expect("connect failed");

    // request to join a table
    let serialized = serde_json::to_string(&RequestTable::Table(TableRequest {
        n_players: 2,
        small_blind: 1,
        big_blind: 2,
        stack: 40,
        game_type: GameType::NoLimit,
    }))
    .unwrap();
    println!("[Player {}]     <sent> {:?}", player, serialized);
    client.send_message(&Message::text(serialized)).ok();

    let mut count = 0;
    while let Ok(msg) = client.recv_message() {
        count += 1;
        if count > 1000 {
            println!("[Player {}] ########## Got a thousand messages, you get the point. Shutting down client", player);
            client.send_message(&Message::close()).ok();
            break;
        }
        match msg {
            OwnedMessage::Text(t) => {
                println!("[Player {}] <received> {:?}", player, t);
                if let Ok(PokerMessage::RequestAction { .. }) =
                    serde_json::from_str::<PokerMessage>(&t)
                {
                    let action = match rand::random::<u8>() {
                        0..=55 => PlayerAction::Raise(2),
                        56..=100 => PlayerAction::Fold,
                        _ => PlayerAction::Call,
                    };
                    let serialized = serde_json::to_string(&Response::Action(action)).unwrap();
                    println!("[Player {}]     <sent> {:?}", player, serialized);
                    let msg = Message::text(serialized);
                    client.send_message(&msg).ok();
                }
            }
            _ => break,
        }
    }
}
