use futures::lock::Mutex as FutureMutex;
use futures::stream::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub struct Peers {
    map: HashMap<String, Peer>,
}

impl Peers {
    pub fn new() -> Peers {
        Peers {
            map: HashMap::new(),
        }
    }
    pub fn add(&mut self, peer: &Peer) {
        self.map.insert(peer.address.to_string(), peer.to_owned());
    }
    pub fn remove(&mut self, address: &str) {
        self.map.remove(address);
    }
    pub fn contains(&self, address: &str) -> bool {
        self.map.contains_key(address)
    }
    pub fn addresses(&self) -> Vec<String> {
        self.map.keys().cloned().collect()
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

pub async fn add_peer_to_peers(peers: Arc<FutureMutex<Peers>>, peer: &Peer, openport: u16) {
    {
        let mut peers = peers.lock().await;
        if peers.contains(&peer.address) {
            return;
        }
        peers.add(peer);
    }

    let address = peer.address.clone();

    tokio::spawn(async move {
        let mut es = EventSource::get(format!("http://{}/sse?openport={}", &address, openport));
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => println!("Connection Open!"),
                Ok(Event::Message(message)) => println!("Message: {:#?}", message),
                Err(err) => {
                    println!("Error: {}", err);
                    es.close();
                }
            }
        }
        let mut peers = peers.lock().await;
        peers.remove(&address);
    });
}
