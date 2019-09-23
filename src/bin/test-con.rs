use ::poker::{
    run_server, GameType, Message as PokerMessage, PlayerAction, RequestTable, Response,
    TableRequest,
};
use std::thread;
use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

const CONNECTION: &str = "ws://ws.sebastiaanvermeulen.nl/pokerroom";

fn main() {
    let mut client = ClientBuilder::new(CONNECTION)
        .expect("address parse fail")
        .connect(None)
        .expect("connect failed");

    println!("{:?}", client.recv_message());

    client
        .send_message(&Message::close())
        .expect("failed to close connection");
}
