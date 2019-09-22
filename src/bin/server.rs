use ::poker::run_server;
use clap::{App, Arg};
use std::thread;

fn main() -> Result<(), ()> {
    let matches = App::new("Pokerroom server")
        .version("0.1")
        .author("Sebastiaan Vermeulen <mail@sebastiaanvermeulen.nl>")
        .about("A pokerroom websocket server")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .value_name("URL")
                .help("server address, defaults to 127.0.0.1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("NUMBER")
                .help("port to use, defaults to 2794")
                .takes_value(true),
        )
        .get_matches();

    let address = matches.value_of("address").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or("2794");
    let loc = String::from(address) + ":" + port;

    let server = thread::spawn(move || {
        println!("server started");
        run_server(&loc);
    });

    // do not end program
    server.join().or(Err(()))
}
