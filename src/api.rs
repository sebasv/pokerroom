use std::thread;
// use serde_json::json;
use websocket::sync::{Client, Server, Stream};
use websocket::{Message as WsMessage, OwnedMessage};

use crate::communication::{Callback, ErrorMessage, GameType, Message, PlayerAction, Response};
use crate::engine::Table;
use std::sync::mpsc::{channel, Sender};

/* TODO
* ERROR HANDLING
* communicate all relevant updates to players
* make game-mode selectable
* make table size selectable
* deal with lost connections
**/

/// runs indefinitely
pub fn run_server(address: &str, n_players: usize) {
    let server = Server::bind(address).unwrap();
    let (tx, rx) = channel();

    // set up server in separate thread to send new clients over channel
    let tx2 = tx.clone();
    thread::spawn(move || {
        for connection in server.filter_map(Result::ok) {
            if let Ok(client) = connection.accept() {
                tx2.send(client);
            }
        }
    });

    // listen to clients from server and from finished games
    let mut clients = Vec::new();
    while let Ok(client) = rx.recv() {
        // queue clients until n_players reached
        clients.push(client);
        if clients.len() == n_players {
            // play game with n_players. if
            let tx3 = tx.clone();
            let mut temp = Vec::new();
            for _ in 0..n_players {
                temp.push(clients.pop().unwrap());
            }
            thread::spawn(move || {
                play_game(temp, tx3);
            });
        }
    }
}

/// Adapter adapts websocket clients to a game and dispatches messages accordingly
struct Adapter<'a, S>
where
    S: Stream + Send,
{
    clients: &'a mut Vec<Client<S>>,
}

impl<'a, S> Adapter<'a, S>
where
    S: Stream + Send + 'static,
{
    fn close_connections(&mut self) {
        while let Some(mut client) = self.clients.pop() {
            thread::spawn(move || {
                for _ in 0..3 {
                    if client.send_message(&WsMessage::close()).is_ok() {
                        break;
                    }
                }
            });
        }
    }
}

impl<'a, S> Callback for Adapter<'a, S>
where
    S: Stream + Send + 'static,
{
    fn callback(&mut self, message: Message) -> Response {
        match message {
            Message::RequestAction { player_id: id, .. } => {
                self.clients[id]
                    .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()));
                self.clients[id]
                    .recv_message()
                    .ok()
                    .and_then(|m| {
                        if let OwnedMessage::Text(t) = m {
                            serde_json::from_str::<Response>(&t).ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Response::Ack)
            }
            Message::GameOver => {
                self.close_connections();
                Response::Ack
            }
            Message::Hole { player, .. } => {
                self.clients[player]
                    .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()));
                Response::Ack
            }
            Message::Flop(..)
            | Message::River(..)
            | Message::Turn(..)
            | Message::Showdown { .. } => {
                for client in self.clients.iter_mut() {
                    client.send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()));
                }
                Response::Ack
            }
            Message::Error(e) => {
                println!("{:?}", e);
                // handle error
                Response::Ack
            }
            other => {
                println!("{:?}", other);
                Response::Ack
            }
        }
    }
}

fn play_game<S>(mut clients: Vec<Client<S>>, tx: Sender<Client<S>>)
where
    S: Stream + Send + 'static,
{
    {
        let n = clients.len();
        let callback = Adapter {
            clients: &mut clients,
        };
        let mut table = Table::new(GameType::NoLimit, 1, vec![100; n], callback);
        // table.play_until_end();
        table.play_n_rounds(2);
    }
    while let Some(client) = clients.pop() {
        tx.send(client);
    }
}
