extern crate ws;
use ws::{ listen, Message };

use std::{env, process};
use std::rc::Rc;
use std::cell::RefCell;

use rust_pubsub::{ Server, SenderHandle };

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[1];

    let mut ws_server = Rc::new(RefCell::new(Server::new())); // Will provide internal mutability to all of the client handlers
    // let mut ws_server = Arc::new(Mutex::new(Server::new()))
    if let Err(error) = listen(([127, 0, 0, 1], port)), |sender| {
        SenderHandle { sender, server_ref: ws_server.clone() }
    }) {
        println!("Failed to start WebSocket server on port {}.", port);
        println!("Error: {}", error);
        
        process::exit(1);

    }
}
/**

|sender| async {
    ws_server.lock();
    some_op().await;
    some_other_ope(ws_server).await
}



*/