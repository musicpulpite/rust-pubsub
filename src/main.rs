extern crate ws;
use ws::listen;

use std::{env, process};

use rust_pubsub::Server;

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[1];

    let mut ws_server = Server::new();

    if let Err(error) = listen(format!("127.0.0.1:{}", port), |client| { 
        let client_token = client.token().clone();
        ws_server.add_client(client);

        move |msg| {
            println!("Got message from client {:?}.", client_token);
            println!("{}", msg);

            return match parse_message(msg) {

            }
        }
    }) {
        println!("Failed to start WebSocket server on port {}.", port);
        println!("Error: {}", error);
        
        process::exit(1);

    }
}
