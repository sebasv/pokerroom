use std::thread;
// use serde_json::json;
use websocket::sync::{Client, Server, Stream};
use websocket::{Message as WsMessage, OwnedMessage};

use crate::communication::{Callback, ErrorMessage, GameType, Message, Response};
use crate::engine::Table;
use std::sync::mpsc::{channel, Sender};

/* TODO
* ERROR HANDLING
* communicate all relevant updates to players
* make game-mode selectable
* make table size selectable
* deal with lost connections
* switch to async websockets for speed
**/

/// runs indefinitely. starts a child thread that listens for new connections.
/// once enough players have been collected, starts a new game in a separate
/// thread. Each game sends all clients back to the main thread at the end of
/// the game.
pub fn run_server(address: &str, n_players: usize) {
    let server = Server::bind(address).unwrap();
    let (tx, rx) = channel();

    // set up server in separate thread to accept new clients and send them to dispatcher
    let tx2 = tx.clone();
    thread::spawn(move || {
        for connection in server.filter_map(Result::ok) {
            if let Ok(client) = connection.accept() {
                tx2.send(client).ok();
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

/// play the actual game. This is all the purely sequential logic of playing a
/// game, to keep the threading code as clean as possible.
fn play_game<S>(mut clients: Vec<Client<S>>, tx: Sender<Client<S>>)
where
    S: Stream + Send + 'static,
{
    {
        let n = clients.len();
        let callback = Adapter {
            clients: &mut clients,
        };
        let mut table = Table::new(GameType::NoLimit, 1, 2, vec![100; n], callback);
        // table.play_until_end();
        table.play_n_rounds(2);
    }
    while let Some(client) = clients.pop() {
        tx.send(client).ok();
    }
}

/// Adapter adapts websocket messages to game messages. In addition the adapter
/// manages communication, so the adapter receives all updates from the game
/// and decides how to dispatch them to the clients.
struct Adapter<'a, S>
where
    S: Stream + Send,
{
    clients: &'a mut Vec<Client<S>>,
}

impl<'a, S> Callback for Adapter<'a, S>
where
    S: Stream + Send + 'static,
{
    fn callback(&mut self, message: Message) -> Result<Response, ErrorMessage> {
        match message {
            Message::RequestAction { player, .. } => {
                self.clients[player]
                    .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()))
                    .or(Err(ErrorMessage::WebSocketError))?;
                self.clients[player]
                    .recv_message()
                    .or(Err(ErrorMessage::WebSocketError))
                    .and_then(|m| {
                        if let OwnedMessage::Text(t) = m {
                            serde_json::from_str::<Response>(&t)
                                .or(Err(ErrorMessage::InvalidResponse))
                        } else {
                            Err(ErrorMessage::InvalidResponse)
                        }
                    })
            }
            Message::Hole { player, .. } => self.clients[player]
                .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()))
                .and(Ok(Response::Ack))
                .or(Err(ErrorMessage::WebSocketError)),
            Message::Flop(..)
            | Message::River(..)
            | Message::Turn(..)
            | Message::GameOver
            | Message::Showdown { .. } => {
                for client in self.clients.iter_mut() {
                    client
                        .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()))
                        .or(Err(ErrorMessage::WebSocketError))?;
                }
                Ok(Response::Ack)
            }
            Message::Error(e) => {
                println!("{:?}", e);
                // handle error
                Ok(Response::Ack)
            }
        }
    }
}
