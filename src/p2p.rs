use futures::stream::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use std::collections::HashMap;

pub struct Peers {
    peers: HashMap<String, Peer>,
}

impl Peers {
    pub fn new() -> Peers {
        Peers {
            peers: HashMap::new(),
        }
    }

    pub async fn add(&mut self, peer: Peer) {
        self.peers.insert(peer.address.clone(), peer.clone());
        let mut es = EventSource::get(format!("{}/sse", &peer.address));
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => println!("Connection Open!"),
                Ok(Event::Message(message)) => println!("Message: {:#?}", message),
                Err(err) => {
                    println!("Error: {}", err);
                    // es.close();
                }
            }
        }
    }

    pub fn remove(&mut self, address: &str) {
        self.peers.remove(address);
    }
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub address: String,
}

impl Peer {
    pub fn new(address: &str) -> Self {
        Peer {
            address: address.to_string(),
        }
    }
}
