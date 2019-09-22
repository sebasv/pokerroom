use std::thread;
use websocket::sync::{Client, Server, Stream};
use websocket::{Message as WsMessage, OwnedMessage};

use crate::communication::{
    Callback, Error, ErrorMessage, Message, RequestTable, Response, TableRequest,
};
use crate::engine::Table;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};

/* TODO
* switch to async websockets for speed
**/

/// runs indefinitely. starts a child thread that listens for new connections.
/// once enough players have been collected, starts a new game in a separate
/// thread. Each game sends all clients back to the main thread at the end of
/// the game.
pub fn run_server(address: &str) {
    let server = Server::bind(address).unwrap();
    let (tx, rx) = channel();

    // set up server in separate thread to accept new clients and send them to dispatcher
    let tx2 = tx.clone();
    thread::spawn(move || {
        for connection in server.filter_map(Result::ok) {
            if let Ok(mut client) = connection.accept() {
                println!("accepted a connection from {:?}", client.peer_addr());
                let tx3 = tx2.clone();
                // fire up separate thread just for requesting the table type
                // to prevent blocking the server or the dispatcher.
                // Optimal: no. Works: yes.
                thread::spawn(move || {
                    if client
                        .send_message(&WsMessage::text(
                            serde_json::to_string(&RequestTable::RequestTable).unwrap(),
                        ))
                        .is_ok()
                    {
                        if let Ok(OwnedMessage::Text(msg)) = client.recv_message() {
                            if let Ok(RequestTable::Table(request)) =
                                serde_json::from_str::<RequestTable>(&msg)
                            {
                                tx3.send((request, client)).ok();
                            }
                        }
                    }
                });
            }
        }
    });

    // listen to clients from server and from stopped games
    let mut queue = HashMap::new();
    while let Ok((table, client)) = rx.recv() {
        let q = queue.entry(table).or_insert_with(Vec::new);
        q.push(client);
        if q.len() == table.n_players {
            // play game with n_players.
            let tx3 = tx.clone();
            let mut clients = Vec::new();
            for _ in 0..table.n_players {
                clients.push(q.pop().unwrap());
            }
            thread::spawn(move || {
                do_game(table, clients, tx3);
            });
        }
    }
}

/// Single-game-type logic. Create a table and keep playing until one of the
/// players generates an error. Kick that player and return the other players
/// to the queue.
fn do_game<S>(
    // game_type: GameType,
    // small_blind: Money,
    // big_blind: Money,
    // stack: Money,
    table_request: TableRequest,
    mut clients: Vec<Client<S>>,
    tx3: Sender<(TableRequest, Client<S>)>,
) where
    S: Stream + Send + 'static,
{
    Table::new(
        table_request.game_type,
        table_request.small_blind,
        table_request.big_blind,
        vec![table_request.stack; clients.len()],
        Adapter {
            clients: &mut clients,
        },
    )
    .play();
    // one of the players got kicked for erroring, return other players
    while let Some(client) = clients.pop() {
        tx3.send((table_request, client)).ok();
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
    fn callback(&mut self, message: Message) -> Result<Response, Error> {
        match message {
            Message::RequestAction { player, .. } => {
                self.clients[player]
                    .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()))
                    .or(Err(Error {
                        player,
                        error: ErrorMessage::WebSocketError,
                    }))?;
                self.clients[player]
                    .recv_message()
                    .or(Err(Error {
                        player,
                        error: ErrorMessage::WebSocketError,
                    }))
                    .and_then(|m| {
                        if let OwnedMessage::Text(t) = m {
                            serde_json::from_str::<Response>(&t).or(Err(Error {
                                player,
                                error: ErrorMessage::InvalidResponse,
                            }))
                        } else {
                            Err(Error {
                                player,
                                error: ErrorMessage::InvalidResponse,
                            })
                        }
                    })
            }
            Message::Hole { player, .. } => self.clients[player]
                .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()))
                .and(Ok(Response::Ack))
                .or(Err(Error {
                    player,
                    error: ErrorMessage::WebSocketError,
                })),
            Message::Flop(..)
            | Message::River(..)
            | Message::Turn(..)
            | Message::GameOver
            | Message::Showdown { .. } => {
                for (player, client) in self.clients.iter_mut().enumerate() {
                    client
                        .send_message(&WsMessage::text(serde_json::to_string(&message).unwrap()))
                        .or(Err(Error {
                            player,
                            error: ErrorMessage::WebSocketError,
                        }))?;
                }
                Ok(Response::Ack)
            }
            Message::Error(Error { player, error }) => {
                println!("player {:?} messed up: {:?}", player, error);
                self.clients.remove(player);
                // handle error
                Ok(Response::Ack)
            }
        }
    }
}
