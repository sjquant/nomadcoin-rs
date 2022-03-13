use futures::stream::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Peers {
    pub peers: HashMap<String, Peer>,
}

impl Peers {
    pub fn new() -> Peers {
        Peers {
            peers: HashMap::new(),
        }
    }

    pub async fn add(&mut self, peer: Peer, openport: u16) {
        if self.peers.contains_key(&peer.address) {
            return;
        }
        self.peers.insert(peer.address.clone(), peer.clone());
        tokio::spawn(async move {
            let mut es = EventSource::get(format!(
                "http://{}/sse?openport={}",
                &peer.address, openport
            ));
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
        });
    }

    pub fn remove(&mut self, address: &str) {
        self.peers.remove(address);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
