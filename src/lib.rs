extern crate ws;
use ws::{ Handler, Sender, Message, Error, ErrorKind, util::Token };

use std::collections::{ BTreeSet, HashMap };
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Cow;
use std::cmp::{ Ord, Ordering };

// Attempt to implement cmp for ws::Sender, hopefully this will satisfy the BTreeSet's need
// for the std::cmp::Ord trait to be implemented

pub trait SenderExt<T> {
    fn cmp(&self, other: &Self) -> Ord
}

pub struct Server {
    pub channels: HashMap<String, BTreeSet<Sender>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            channels: HashMap::new()
        }
    }

    pub fn sub_client(&mut self, sender: Sender, channel: String) -> Result<(), String> {
        // Cloning the channel string here for now so I can use it below, will come back and reassess
        let subbed_clients = self.channels.entry(channel.clone()).or_insert(BTreeSet::new());
        let result = subbed_clients.insert(sender);

        return if result {
            Ok(())
        } else {
            Err(format!("Client {:?} already subscribed to channel {}.", sender, channel))
        }
    }

    pub fn unsub_client(&mut self, sender: &Sender, channel: String) -> Result<(), String> {
        // Illuminating distinction about how Rust works, the methods above needed owned vars - these need refs.
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            subbed_clients.remove(sender);
            return Ok(())
        } else {
            return Err(format!("Client {:?} was never subscribed to channel {}.", sender, channel))
        }
    }
}

// I may need to "Cover" the ws::Sender type in order to be able to impl the ordering trait for it
// So that BTreeSet can maintain an ordered set of Sender structs
// struct CustomSender<T>(T);

// impl CustomSender { // Probably wrong
//     fn token(&self) -> Token {
//         self.T.token()
//     }
// }

// impl<T> Ord for CustomSender<T> {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.token().cmp(&other.token())
//     }
// }

pub struct SenderHandle {
    sender: Sender,
    server_ref: Rc<RefCell<Server>>
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
    fn on_message(&mut self, msg: Message) -> Result<(), Error> {
        return match self.parse_message(msg.into_text().unwrap()).unwrap() {
            // Probably going to need to use Rc's here as well since BTreeSet::insert needs to take ownership
            PubSubMessage::SUBSCRIBE { channel } => self.server_ref.sub_client(self.sender, channel),
            PubSubMessage::UNSUBSCRIBE { channel } => self.server_ref.unsub_client(&self.sender, channel),
            PubSubMessage::PUBLISH { channel, msg } => Ok(()),
            _ => Error { kind: ErrorKind::Protocol, details: Cow::Borrowed("Some other WS message") }
        }
    }
}