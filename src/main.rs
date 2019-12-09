extern crate ws;
use ws::{ listen, Message };

use std::{env, process};

use rust_pubsub::Server;
use rust_pubsub::{ parse_message, PubSubMessage };

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[1];

    let mut ws_server = Server::new();

    if let Err(error) = listen(format!("127.0.0.1:{}", port), |client| { 
        let client_token = client.token().clone(); // Probably temporary as well
        ws_server.add_client(client);

        move |msg: Message| {
            println!("Got message from client {:?}.", client_token);
            println!("{}", msg);

            match parse_message(msg.into_text().unwrap()).unwrap() {
                PubSubMessage::SUBSCRIBE { channel } => ws_server.sub_client(client_token, channel).unwrap(),
                PubSubMessage::UNSUBSCRIBE { channel } => ws_server.unsub_client(client_token, channel).unwrap(),
                PubSubMessage::PUBLISH { channel, msg } => println!("Publishing {} on channel {}", msg, channel),
                _ => println!("Some other WS message")
            }

            Ok(())
        }
    }) {
        println!("Failed to start WebSocket server on port {}.", port);
        println!("Error: {}", error);
        
        process::exit(1);

    }
}
