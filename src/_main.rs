mod api;
mod communication;
mod engine;

use std::thread;

/*  TODO
* docs
**/


fn main() -> Result<(), ()> {
    let server = thread::spawn(move || {
        api::run_server("127.0.0.1:2794");
    });

    // do not end program
    server.join().or(Err(()))
}