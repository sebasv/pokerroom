use poker::engine;

struct DumbCallback {}

impl engine::Callback for DumbCallback {
    fn callback(&mut self, message: engine::Message) -> engine::Response {
        println!("{:?}", message);
        match message {
            engine::Message::RequestAction { .. } => {
                engine::Response::Action(engine::PlayerAction::Call)
            }
            other => engine::Response::Ack,
        }
    }
}

fn main() {
    let callback = DumbCallback {};
    let mut table = engine::Table::new(engine::GameType::NoLimit, 1, vec![100, 100, 100], callback);
    table.play_until_end();
}
