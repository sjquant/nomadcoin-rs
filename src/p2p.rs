use futures::lock::Mutex as FutureMutex;
use futures::stream::StreamExt;
use pickledb::PickleDb;
use reqwest_eventsource::{Event, EventSource};
use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{Block, BlockChain};

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

pub async fn add_peer_to_peers(
    chain: &BlockChain,
    db: &mut PickleDb,
    peers: Arc<FutureMutex<Peers>>,
    peer: &Peer,
    openport: u16,
) {
    {
        let mut peers = peers.lock().await;
        if peers.contains(&peer.address) {
            return;
        }
        peers.add(peer);
    }

    let address = peer.address.clone();
    let newest_block = chain.get_block(db, chain.newest_hash.clone());

    tokio::spawn(async move {
        let mut es = EventSource::get(format!("http://{}/sse?openport={}", &address, openport));
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => {
                    println!("Connection Open!");
                    send_newest_block(&address, newest_block.clone()).await;
                }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum P2PEvent {
    NewestBlockReceived,
    AllBlocksRequested,
    ALlBlocksRecevied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub event: P2PEvent,
    pub payload: Option<String>,
}

async fn send_message(address: &str, msg: P2PMessage) {
    reqwest::Client::new()
        .post(format!("http://{}/sse", address))
        .json(&msg)
        .send()
        .await
        .unwrap();
}

pub async fn handle_message(chain: &BlockChain, db: &mut PickleDb, peer: &Peer, msg: &P2PMessage) {
    match msg.event {
        P2PEvent::NewestBlockReceived => {
            on_newest_block_received(msg, chain, db, peer).await;
        }
        P2PEvent::AllBlocksRequested => {
            println!("All blocks requested");
        }
        P2PEvent::ALlBlocksRecevied => {
            println!("All blocks received");
        }
    }
}

async fn on_newest_block_received(
    msg: &P2PMessage,
    chain: &BlockChain,
    db: &mut PickleDb,
    peer: &Peer,
) {
    let peer_newest_block = msg.payload.as_ref().map_or(None, |payload| {
        Some(serde_json::from_str::<Block>(payload).unwrap())
    });
    let own_newest_block = chain.get_block(db, chain.newest_hash.clone());
    if let Some(peer_newest_block) = peer_newest_block {
        if own_newest_block.is_none()
            || own_newest_block.is_some()
                && peer_newest_block.height >= own_newest_block.as_ref().unwrap().height
        {
            request_all_blocks(&peer.address).await;
        } else {
            send_newest_block(&peer.address, own_newest_block).await;
        }
    }
}

async fn request_all_blocks(address: &str) {
    let payload = P2PMessage {
        event: P2PEvent::AllBlocksRequested,
        payload: None,
    };
    send_message(address, payload).await;
}

async fn send_newest_block(address: &str, newest_block: Option<Block>) {
    let payload = P2PMessage {
        event: P2PEvent::NewestBlockReceived,
        payload: newest_block.map_or(None, |block| Some(serde_json::to_string(&block).unwrap())),
    };
    send_message(address, payload).await;
}
