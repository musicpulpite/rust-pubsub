extern crate ws;
use ws::{Handler, Sender, Message};

use std::collections::{BTreeSet, HashMap};

pub struct Server {
    // This clients map should be temporary, eventually the clients (Sender struct)
    // will just be referenced directly from the channels
    pub clients: HashMap<Token, Sender>,
    pub channels: HashMap<String, BTreeSet<Token>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: HashMap::new(),
            channels: HashMap::new()
        }
    }

    // For now I won't return any success/failure indication
    pub fn add_client(&mut self, client: Sender) -> () {
        self.clients.insert(client.token(), client);
    }

    pub fn sub_client(&mut self, client_token: Token, channel: String) -> Result<(), String> {
        // Cloning the channel string here for now so I can use it below, will come back and reassess
        let subbed_clients = self.channels.entry(channel.clone()).or_insert(BTreeSet::new());
        let result = subbed_clients.insert(client_token);

        return if result {
            Ok(())
        } else {
            Err(format!("Client {:?} already subscribed to channel {}.", client_token, channel))
        }
    }

    pub fn unsub_client(&mut self, client_token: Token, channel: String) -> Result<(), String> {
        // Illuminating distinction about how Rust works, the methods above needed owned vars - these need refs.
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            subbed_clients.remove(&client_token);
            return Ok(())
        } else {
            return Err(format!("Client {:?} was never subscribed to channel {}.", client_token, channel))
        }
    }
}

pub struct SenderHandle {
    sender: Sender
}

pub enum PubSubMessage {
    // For now I won't worry about subbing/unsubbing to an array of channels
    SUBSCRIBE { channel: String },
    UNSUBSCRIBE { channel: String },
    PUBLISH { channel: String, msg: String},
    PSUBSCRIBE { pattern: String },
    PUNSUBSCRIBE { pattern: String},
}

impl SenderHandle {
    fn parse_message(&self, msg: String) -> Result<PubSubMessage, String> {
        let msg_contents: Vec<&str> = msg.split(" ").collect();
    
        return match msg_contents.as_slice() {
            ["SUBSCRIBE", channel] => Ok(PubSubMessage::SUBSCRIBE { channel: channel.to_string() }),
            ["UNSUBSCRIBE", channel] => Ok(PubSubMessage::UNSUBSCRIBE { channel: channel.to_string() }),
            ["PUBLISH", channel, msg] => Ok(PubSubMessage::PUBLISH { channel: channel.to_string(), msg: msg.to_string() }),
            _ => Err("Could not parse ws message.".to_string())
        }
    }
}

impl Handler for SenderHandle {
    fn on_message(&mut self, msg: Message) -> Result<(), ws::Err> {
        match self.parse_message(msg.into_text().unwrap()).unwrap() {
            PubSubMessage::SUBSCRIBE { channel } => ws_server.sub_client(client_token, channel).unwrap(),
            PubSubMessage::UNSUBSCRIBE { channel } => ws_server.unsub_client(client_token, channel).unwrap(),
            PubSubMessage::PUBLISH { channel, msg } => println!("Publishing {} on channel {}", msg, channel),
            _ => Err("Some other WS message".to_string())
        }

        Ok(())
    }
}