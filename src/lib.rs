extern crate ws;
use ws::{ Handshake, Handler, Sender, Message, Error, ErrorKind, util::Token };

use std::collections::{ BTreeSet, HashMap };
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Cow;

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

    pub fn add_client(&mut self, client: &Sender) -> () {
        let client_token = client.token().clone();
        self.clients.insert(client_token, client.clone());
    }

    pub fn sub_client(&mut self, client: &Token, channel: String) -> Result<(), Error> {
        // Cloning the channel string here for now so I can use it below, will come back and reassess
        let subbed_clients = self.channels.entry(channel.clone()).or_insert(BTreeSet::new());
        let result = subbed_clients.insert(client.clone());

        return if result {
            Ok(())
        } else {
            Err(Error { kind: ErrorKind::Internal, details: Cow::Borrowed("The client was already subscribed to this channel.") })
            // Err(format!("Client {:?} already subscribed to channel {}.", client, channel))
        }
    }

    pub fn unsub_client(&mut self, client: &Token, channel: String) -> Result<(), Error> {
        // Illuminating distinction about how Rust works, the methods above needed owned vars - these need refs.
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            subbed_clients.remove(client);
            return Ok(())
        } else {
            Err(Error { kind: ErrorKind::Internal, details: Cow::Borrowed("The client was never subscribed to this channel.") })
            // return Err(format!("Client {:?} was never subscribed to channel {}.", client, channel))
        }
    }
}

pub struct ClientHandle {
    pub client: Sender,
    pub ws_server_ref: Rc<RefCell<Server>>
}

pub enum PubSubMessage {
    // For now I won't worry about subbing/unsubbing to an array of channels
    SUBSCRIBE { channel: String },
    UNSUBSCRIBE { channel: String },
    PUBLISH { channel: String, msg: String},
    PSUBSCRIBE { pattern: String },
    PUNSUBSCRIBE { pattern: String},
}

impl ClientHandle {
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

impl Handler for ClientHandle {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        self.ws_server_ref.borrow_mut().add_client(&self.client);
        Ok(())
    }
    
    fn on_message(&mut self, msg: Message) -> Result<(), Error> {
        return match self.parse_message(msg.into_text().unwrap()).unwrap() {
            // Probably going to need to use Rc's here as well since BTreeSet::insert needs to take ownership
            PubSubMessage::SUBSCRIBE { channel } => self.ws_server_ref.borrow_mut().sub_client(&self.client.token(), channel),
            PubSubMessage::UNSUBSCRIBE { channel } => self.ws_server_ref.borrow_mut().unsub_client(&self.client.token(), channel),
            PubSubMessage::PUBLISH { channel, msg } => Ok(()),
            _ => Err(Error { kind: ErrorKind::Protocol, details: Cow::Borrowed("Some other WS message") })
        }
    }
}