extern crate ws;
use ws::{Handler, Sender, util::Token};

use std::collections::{BTreeSet, HashMap};

pub enum PubSubMessage {
    // For now I won't worry about subbing/unsubbing to an array of channels
    SUBSCRIBE { channel: String },
    UNSUBSCRIBE { channel: String },
    PUBLISH { channel: String, msg: String},
    PSUBSCRIBE { pattern: String },
    PUNSUBSCRIBE { pattern: String},
}

pub fn parse_message(msg: String) -> Result<PubSubMessage, String> {
    let msg_contents: Vec<String> = msg.split(" ").collect();

    return match msg_contents.as_slice() {
        ["SUBSCRBE", channel] => Ok(PubSubMessage::SUBSCRIBE { channel }),
        ["UNSUBSCRIBE", channel] => Ok(PubSubMessage::UNSUBSCRIBE { channel }),
        ["PUBLISH", channel, msg] => Ok(PubSubMessage::PUBLISH { channel, msg }),
        _ => Err("Could not parse ws message.")

    }
}

pub struct Server {
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
        let mut subbed_clients = self.channels.entry(channel.clone()).or_insert(BTreeSet::new());
        let result = subbed_clients.insert(client_token);

        return if result {
            Ok(())
        } else {
            Err(format!("Client {:?} already subscribed to channel {}.", client_token, channel))
        }
    }
}