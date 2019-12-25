extern crate ws;
use ws::{ 
    Handshake,
    Handler,
    Sender,
    Message,
    Error,
    ErrorKind,
    CloseCode,
    util::Token
};

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

    pub fn add_client(&mut self, client: &Sender) {
        let client_token = client.token().clone();
        self.clients.insert(client_token, client.clone());
    }

    pub fn remove_client(&mut self, token: &Token) {
        self.clients.remove(token);
        // would it be more efficient to leave these tokens in each channel and remove them
        // on the next pub message once we see that the client has been removed?
        for subbed_clients in self.channels.values_mut() {
            subbed_clients.remove(token);
        }
    }

    pub fn sub_client(&mut self, client: &Token, channel: String) -> Result<(), Error> {
        let subbed_clients = self.channels.entry(channel.clone()).or_insert(BTreeSet::new());
        let result = subbed_clients.insert(client.clone());

        return if result {
            Ok(())
        } else {
            Err(Error { kind: ErrorKind::Internal, details: Cow::Borrowed("The client was already subscribed to this channel.") })
        }
    }

    pub fn unsub_client(&mut self, client: &Token, channel: String) -> Result<(), Error> {
        // Illuminating distinction about how Rust works, the methods above needed owned vars - these need refs.
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            subbed_clients.remove(client);
            return Ok(())
        } else {
            Err(Error { kind: ErrorKind::Internal, details: Cow::Borrowed("The client was never subscribed to this channel.") })
        }
    }

    pub fn pub_message(&mut self, channel: String, msg: String ) -> Result<(), Error> {
        // Probably bad practice to pass an owned variable into the function but use its reference, will come back
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            for client_token in subbed_clients.iter() {
                if let Some(client) = self.clients.get(client_token) {
                    client.send(msg.clone())?;
                }
            }

            Ok(())
        } else {
            Ok(())
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
    // PSUBSCRIBE { pattern: String },
    // PUNSUBSCRIBE { pattern: String},
}

impl ClientHandle {
    fn parse_message(&self, msg: String) -> Result<PubSubMessage, Error> {
        let msg_contents: Vec<&str> = msg.split(" ").collect();
    
        return match msg_contents.as_slice() {
            ["SUBSCRIBE", channel] => Ok(PubSubMessage::SUBSCRIBE { channel: channel.to_string() }),
            ["UNSUBSCRIBE", channel] => Ok(PubSubMessage::UNSUBSCRIBE { channel: channel.to_string() }),
            ["PUBLISH", channel, msg] => Ok(PubSubMessage::PUBLISH { channel: channel.to_string(), msg: msg.to_string() }),
            _ => Err(Error { kind: ErrorKind::Protocol, details: Cow::Borrowed("Received invalid WS message.") })
        }
    }
}

impl Handler for ClientHandle {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        self.ws_server_ref.borrow_mut().add_client(&self.client);
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        self.ws_server_ref.borrow_mut().remove_client(&self.client.token());
    }
    
    fn on_message(&mut self, msg: Message) -> Result<(), Error> {
        return match self.parse_message(msg.into_text().unwrap())? {
            PubSubMessage::SUBSCRIBE { channel } => self.ws_server_ref.borrow_mut().sub_client(&self.client.token(), channel),
            PubSubMessage::UNSUBSCRIBE { channel } => self.ws_server_ref.borrow_mut().unsub_client(&self.client.token(), channel),
            PubSubMessage::PUBLISH { channel, msg } => self.ws_server_ref.borrow_mut().pub_message(channel, msg),
        }
    }
}