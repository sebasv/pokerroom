use ::poker::run_server;
use std::thread;

/*  TODO
* docs
**/

fn main() -> Result<(), ()> {
    let server = thread::spawn(move || {
        run_server("127.0.0.1:2794");
    });

    // do not end program
    server.join().or(Err(()))
}
