use std::thread;
use websocket::{OwnedMessage, Message as WsMessage};
use websocket::sync::{Server, Client, Stream};

use crate::engine::{GameType, ActionCallback, PlayerAction, Message, ErrorMessage, Table};

/* TODO 
* ERROR HANDLING
* communicate all relevant updates to players
* make game-mode selectable
* deal with lost connections
**/

/// runs indefinitely
pub fn run_server(address: &str, n_players: usize) {
    let server = Server::bind(address).unwrap();
    let mut clients = Vec::new();

    for connection in server.filter_map(Result::ok) {
        clients.push(connection.accept().unwrap());
        if clients.len() == n_players {
            let mut temp = Vec::new();
            for _ in 0..n_players {
                temp.push(clients.pop().unwrap());
            }
            thread::spawn(move || {
                play_game(temp);
            });
        } else {
            let n = n_players - clients.len();
            for client in &mut clients {
                client.send_message(&WsMessage::text(format!("waiting for {} more players", n)));
            }
        }
    }
}

/// The protocol for DumbCallback is simple:
/// clients listen. If they read "?action" they respond with their preferred action, which is {"fold", "call", n} where n is the number of chips they want to raise.
struct DumbCallback<S>
where S: Stream {
    clients: Vec<Client<S>>,
}

impl<S> DumbCallback<S>
where S: Stream {
    fn close_connections(&mut self) {
        for client in &mut self.clients {
            client.send_message(&OwnedMessage::Close(None));
        }
    }
}

impl<S> ActionCallback for DumbCallback<S> 
where S: Stream {
    fn callback(&mut self, message: Message) -> Message {

    // Flop(Card,Card,Card),
    // River(Card),
    // Turn(Card),
    // Showdown{score: Score, pot: Money, players: Vec<usize>},
    // Player{id: usize, action: PlayerAction},
    // RequestAction(usize),
    // Error(ErrorMessage),
    // Ack,
        match message {
            Message::RequestAction(id) => {
                self.clients[id].send_message(&WsMessage::text("?action"));
                match self.clients[id].recv_message() {
                    Ok(OwnedMessage::Text(ref s)) if s == "fold" => {Message::Player{id, action: PlayerAction::Fold}},
                    Ok(OwnedMessage::Text(ref s)) if s == "call" => {Message::Player{id, action: PlayerAction::Call}},
                    Ok(OwnedMessage::Text(s)) => {
                        if let Ok(bet) = s.parse() {
                            Message::Player{id, action: PlayerAction::Raise(bet)}
                        } else {
                            Message::Error(ErrorMessage::InvalidResponse)
                        }
                        },
                    _ => {Message::Error(ErrorMessage::InvalidResponse)},
                }
            },
            Message::GameOver => {self.close_connections(); Message::Ack},
            other => {
                println!("{:?}", other);
                Message::Ack
            }
        }
    }
}

fn play_game<S>(clients: Vec<Client<S>>) 
where S: Stream {
    let n = clients.len();
    let callback = DumbCallback{clients};
    let mut table = Table::new(GameType::NoLimit, 1, vec![100; n], callback);

    // table.play_until_end();
    table.play_n_rounds(1000);
}
