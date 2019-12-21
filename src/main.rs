extern crate ws;
use ws::listen;

use std::{env, process};
use std::rc::Rc;
use std::cell::RefCell;

use rust_pubsub::{ Server, ClientHandle };

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[1];

    let ws_server = Rc::new(RefCell::new(Server::new())); // Will provide internal mutability to all of the client handlers

    if let Err(error) = listen(format!("127.0.0.1:{}", port), |client| {
        ClientHandle { client, ws_server_ref: ws_server.clone() }
    }) {
        println!("Failed to start WebSocket server on port {}.", port);
        println!("Error: {}", error);
        
        process::exit(1);

    }
}