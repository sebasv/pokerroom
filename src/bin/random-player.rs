use ::poker::{
    GameType, Message as PokerMessage, PlayerAction, RequestTable, Response, TableRequest,
};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

// const CONNECTION: &str = "wss://wss.sebastiaanvermeulen.nl/pokerroom";
const CONNECTION: &str = "ws://127.0.0.1:2794";

fn main() {
    let n_players = 4;
    let running = Arc::new(AtomicBool::new(true));
    let (tx, rx) = channel();

    let threads = (0..n_players)
        .map(|i| {
            let b = running.clone();
            let t = tx.clone();
            thread::spawn(move || {
                run_player(i, b, t);
            })
        })
        .collect::<Vec<_>>();
    let mut file = File::create("out.log").unwrap();
    while let Ok(m) = rx.recv() {
        writeln!(file, "{}", m).unwrap();
    }
    for thread in threads {
        thread.join().expect("child thread panicked");
    }
}

fn run_player(player: usize, running: Arc<AtomicBool>, tx: Sender<String>) {
    let mut client = ClientBuilder::new(CONNECTION)
        .expect("could not build connection")
        .add_protocol("rust-websocket")
        .connect(None)
        .expect("connect failed");

    // request to join a table
    let serialized = serde_json::to_string(&RequestTable::Table(TableRequest {
        n_players: 3,
        small_blind: 1,
        big_blind: 2,
        stack: 40,
        game_type: GameType::NoLimit,
    }))
    .unwrap();
    tx.send(format!("[Player {}]     <sent> {}", player, serialized))
        .unwrap();
    client.send_message(&Message::text(serialized)).ok();

    let mut count = 0;
    while let Ok(msg) = client.recv_message() {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        count += 1;
        if count > 100 {
            tx.send(format!("[Player {}] ########## Got a thousand messages, you get the point. Shutting down client", player)).unwrap();
            client.send_message(&Message::close()).ok();
            running.store(false, Ordering::SeqCst);
            break;
        }
        match msg {
            OwnedMessage::Text(t) => {
                tx.send(format!("[Player {}] <received> {}", player, t))
                    .unwrap();
                if let Ok(PokerMessage::RequestAction { .. }) =
                    serde_json::from_str::<PokerMessage>(&t)
                {
                    let action = match rand::random::<u8>() {
                        0..=55 => PlayerAction::Raise(2),
                        56..=100 => PlayerAction::Fold,
                        _ => PlayerAction::Call,
                    };
                    let serialized = serde_json::to_string(&Response::Action(action)).unwrap();
                    tx.send(format!("[Player {}]     <sent> {}", player, serialized))
                        .unwrap();
                    let msg = Message::text(serialized);
                    client.send_message(&msg).ok();
                }
            }
            _ => break,
        }
    }
}
