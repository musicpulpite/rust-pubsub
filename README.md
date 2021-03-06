## Introduction  
This repository contains the source code for a simple implementation of the Redis PubSub engine written in Rust. The architecture was based on the original source code (http://download.redis.io/redis-stable/src/pubsub.c) and the deep-dive article written here https://making.pusher.com/redis-pubsub-under-the-hood/. The first version of this project is a simple WebSocket server using the ws Rust crate. Clients can establish connections to this server and send subscribe/unscubscribe/publish messages to it to be processed. The second version of this project will abstract the message processing logic to a Rust library crate that will be compiled to a WASI module and used in a Node server that handles the WebSocket connections and any middleware logic.

## Core architecture  
At its core a PubSub Engine is a very simple program - it stores a hashmap of channel names that map to linked lists of subscribed clients. Likewise each client handle maintains a hashmap of its own subscribed channels. The process of publishing a message involves matching the publishing channel to its set of subscribed clients and sending the corresponding message to each one.  

The key difference in this implementation is that each set of subscribed clients is implemented as a BTreeSet (from Rust's standard library). Since this is an ordered set (ordered by the token object for each client) we do not need to maintain a separate data structure to track a clients subscribed channels (membership can be checked in logarithmic time). The one downside to this approach is that the UNSUBSCRIBE operation for any single client may slower (O(A * log B), where A is the number of channels and B is the number of subbed clients per channel vs. O(C * B), with C the number of subbed channels).

The data structure of my PubSub server (where Sender is the client handle and token is its unique identifier):
```
pub struct Server {
    pub clients: HashMap<Token, Sender>,
    pub channels: HashMap<String, BTreeSet<Token>>
}
```

## Next steps  
The next iteration of this project will be to abstract away the core sub/unsub logic of this rust crate into a new crate that will be compiled to a WASI module and incorporated into a Node server that handles the WebSocket logic. This will allow for faster customization since the core logic will remain untouched and will serve as a nice proof-of-concept of the capabilities of the WASI virtual machine code format.