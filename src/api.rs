use std::thread;
use std::time::Duration;
use websocket::sync::{Client, Server, Stream};
use websocket::{Message as WsMessage, OwnedMessage};

use crate::communication::{
    Callback, Error, ErrorMessage, Message, RequestTable, Response, TableRequest,
};
use crate::engine::Table;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};

/* TODO
* switch to async for the table-request code
**/

/// runs indefinitely. starts a child thread that listens for new connections.
/// once enough players have been collected, starts a new game in a separate
/// thread. Each game sends all clients back to the main thread at the end of
/// the game.
pub fn run_server(address: &str) {
    let (tx, rx) = channel();

    // set up server in separate thread to accept new clients and send them to dispatcher
    let server = Server::bind(address).unwrap();
    let tx2 = tx.clone();
    let (incoming_tx, incoming_rx) = channel();

    let (update_tx, update_rx) = channel();

    // connection-accepting thread
    thread::spawn(move || {
        for client in server
            .filter_map(Result::ok)
            .filter_map(|connection| connection.accept().ok())
        {
            println!("accepting a connection from {:?}", client.peer_addr());
            incoming_tx
                .send(client)
                .expect("incoming-rx thread hung up");
        }
    });

    // give-updates-and-listen-for-table-type thread
    thread::spawn(move || {
        let mut clients = Vec::new();
        loop {
            // if we receive a new client within 1 second, add them to the main queue
            if let Ok(client) = incoming_rx.recv_timeout(Duration::from_secs(1)) {
                client.set_nonblocking(true).unwrap();
                clients.push(client);
            }

            // If we receive an update, broadcast amongst all queued clients.
            // Drop clients whose connection fails.
            if let Ok(update) = update_rx.recv_timeout(Duration::from_secs(1)) {
                for i in (0..clients.len()).rev() {
                    if clients[i]
                        .send_message(&WsMessage::text(serde_json::to_string(&update).unwrap()))
                        .is_err()
                    {
                        clients.remove(i);
                    }
                }
            }

            // If any of the clients has decided on a table, send them to the tables queue. If we don't understand the message, drop the connection.
            for i in (0..clients.len()).rev() {
                if let Ok(OwnedMessage::Text(msg)) = clients[i].recv_message() {
                    let mut client = clients.remove(i);
                    if let Ok(RequestTable::Table(request)) =
                        serde_json::from_str::<RequestTable>(&msg)
                    {
                        client.set_nonblocking(false).ok();
                        tx2.send((request, client)).expect("main thread hung up");
                    } else {
                        client.send_message(&WsMessage::close()).ok();
                    }
                }
            }
        }
    });

    // listen to clients from server and from stopped games
    let mut queue = HashMap::new();
    while let Ok((table, client)) = rx.recv() {
        {
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
        // send update about queues
        update_tx
            .send(queue.iter().map(|(&k, v)| (k, v.len())).collect::<Vec<_>>())
            .ok();
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
