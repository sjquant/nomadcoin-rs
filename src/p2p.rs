use futures::lock::Mutex as FutureMutex;
use futures::stream::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use crate::{Block, BlockChain, Transaction};

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
    pub fn all(&self) -> Vec<Peer> {
        self.map.values().cloned().collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Peer {
    pub id: String,
    pub address: String,
}

impl Peer {
    pub fn new(id: &str, address: &str) -> Self {
        Peer {
            id: id.to_string(),
            address: address.to_string(),
        }
    }
}

pub async fn add_peer_to_peers(
    app_id: String,
    chain: &mut BlockChain,
    peers: Arc<FutureMutex<Peers>>,
    peer: &Peer,
    openport: u16,
    should_broadcast: bool,
) {
    {
        let mut peers = peers.lock().await;
        if peers.contains(&peer.address) {
            return;
        }
        peers.add(peer);
    }

    let address = peer.address.clone();
    let newest_block = chain.newest_block();
    let peer = peer.clone();

    tokio::spawn(async move {
        let mut es = EventSource::get(format!("http://{}/sse?openport={}", &address, openport));
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => {
                    println!("Connection Open!");
                    send_newest_block(app_id.clone(), &address, newest_block.clone()).await;
                    if should_broadcast {
                        broadcast_new_peer(app_id.clone(), peers.clone(), peer.clone()).await;
                    }
                }
                Err(err) => {
                    println!("Error: {}", err);
                    es.close();
                }
                _ => {}
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
    AllBlocksRecevied,
    NewBlockNotified,
    NewTxnNotified,
    NewPeerNotified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub event: P2PEvent,
    pub payload: Option<String>,
    pub sender_id: String,
}

async fn send_message(address: &str, msg: P2PMessage) {
    reqwest::Client::new()
        .post(format!("http://{}/sse", address))
        .json(&msg)
        .send()
        .await
        .unwrap();
}

async fn broadcast_message(peers: Arc<FutureMutex<Peers>>, msg: P2PMessage) {
    let peers = peers.lock().await;
    for address in peers.map.keys() {
        send_message(address, msg.clone()).await;
    }
}

pub async fn on_p2p_event(
    app_id: String,
    chain: &mut BlockChain,
    peer: &Peer,
    msg: &P2PMessage,
    peers: Arc<FutureMutex<Peers>>,
    openport: u16,
) {
    match msg.event {
        P2PEvent::NewestBlockReceived => {
            on_newest_block_received(app_id, chain, peer, msg, peers, openport).await;
        }
        P2PEvent::AllBlocksRequested => {
            on_all_blocks_requested(app_id, chain, peer, msg, peers, openport).await;
        }
        P2PEvent::AllBlocksRecevied => {
            on_all_blocks_received(app_id, chain, peer, msg, peers, openport).await;
        }
        P2PEvent::NewBlockNotified => {
            on_new_block_notified(app_id, chain, peer, msg, peers, openport).await;
        }
        P2PEvent::NewTxnNotified => {
            on_new_txn_notified(app_id, chain, peer, msg, peers, openport).await;
        }
        P2PEvent::NewPeerNotified => {
            on_new_peer_notified(app_id, chain, peer, msg, peers, openport).await;
        }
    }
}

async fn on_new_peer_notified(
    app_id: String,
    chain: &mut BlockChain,
    peer: &Peer,
    msg: &P2PMessage,
    peers: Arc<FutureMutex<Peers>>,
    openport: u16,
) {
    println!("New peer notified from {}", peer.address.as_str());
    let new_peer = serde_json::from_str::<Peer>(msg.payload.as_ref().unwrap()).unwrap();
    add_peer_to_peers(app_id, chain, peers, &new_peer, openport, false).await;
}

async fn broadcast_new_peer(app_id: String, peers: Arc<FutureMutex<Peers>>, new_peer: Peer) {
    println!("Broadcast new peer {}", new_peer.address.as_str());
    let msg = P2PMessage {
        event: P2PEvent::NewPeerNotified,
        payload: Some(serde_json::to_string(&new_peer).unwrap()),
        sender_id: app_id.clone(),
    };
    let peers = peers.lock().await;
    for address in peers.map.keys() {
        if &new_peer.address != address {
            send_message(address, msg.clone()).await;
        }
    }
}

async fn on_newest_block_received(
    app_id: String,
    chain: &mut BlockChain,
    peer: &Peer,
    msg: &P2PMessage,
    _peers: Arc<FutureMutex<Peers>>,
    _openport: u16,
) {
    println!("Got newest block from {}", peer.address);
    let peer_newest_block = msg.payload.as_ref().map_or(None, |payload| {
        Some(serde_json::from_str::<Block>(payload).unwrap())
    });
    let own_newest_block = chain.newest_block();
    if let Some(peer_newest_block) = peer_newest_block {
        // TODO: improve this to send message after all connection is established
        // Give time for connection to be established
        // I know it's bad idea, but I'am not sure how to handle it better in SSE instead of websockets
        thread::sleep(Duration::from_millis(1000));
        if own_newest_block.is_none()
            || own_newest_block.is_some()
                && peer_newest_block.height >= own_newest_block.as_ref().unwrap().height
        {
            request_all_blocks(app_id, &peer.address).await;
        } else {
            send_newest_block(app_id, &peer.address, own_newest_block).await;
        }
    }
}

async fn request_all_blocks(app_id: String, address: &str) {
    println!("Requesting all blocks from {}", address);
    let payload = P2PMessage {
        event: P2PEvent::AllBlocksRequested,
        payload: None,
        sender_id: app_id,
    };
    send_message(address, payload).await;
}

async fn send_newest_block(app_id: String, address: &str, newest_block: Option<Block>) {
    println!("Send newest block to {}", address);
    let payload = P2PMessage {
        event: P2PEvent::NewestBlockReceived,
        payload: newest_block.map_or(None, |block| Some(serde_json::to_string(&block).unwrap())),
        sender_id: app_id,
    };
    send_message(address, payload).await;
}

async fn on_all_blocks_requested(
    app_id: String,
    chain: &mut BlockChain,
    peer: &Peer,
    _msg: &P2PMessage,
    _peers: Arc<FutureMutex<Peers>>,
    _openport: u16,
) {
    println!("All blocks requested from {}", peer.address);
    let blocks = chain.all_blocks();
    send_all_blocks(app_id, &peer.address, blocks).await;
}

async fn send_all_blocks(app_id: String, address: &str, all_blocks: Vec<Block>) {
    println!("Send all blocks to {}", address);
    let payload = P2PMessage {
        event: P2PEvent::AllBlocksRecevied,
        payload: Some(serde_json::to_string(&all_blocks).unwrap()),
        sender_id: app_id,
    };
    send_message(address, payload).await;
}

async fn on_all_blocks_received(
    _app_id: String,
    chain: &mut BlockChain,
    peer: &Peer,
    msg: &P2PMessage,
    _peers: Arc<FutureMutex<Peers>>,
    _openport: u16,
) {
    println!("Got all blocks from {}", peer.address);
    let blocks: Option<Vec<Block>> = msg
        .payload
        .as_ref()
        .map(|payload| serde_json::from_str(payload).unwrap());

    if let Some(blocks) = blocks {
        if blocks.is_empty() {
            return;
        }
        chain.replace(blocks);
    }
}

pub async fn broadcast_new_block(app_id: String, peers: Arc<FutureMutex<Peers>>, block: Block) {
    println!("Broadcast new block");
    let msg = P2PMessage {
        event: P2PEvent::NewBlockNotified,
        payload: Some(serde_json::to_string(&block).unwrap()),
        sender_id: app_id.clone(),
    };
    broadcast_message(peers, msg).await;
}

async fn on_new_block_notified(
    _app_id: String,
    chain: &mut BlockChain,
    peer: &Peer,
    msg: &P2PMessage,
    _peers: Arc<FutureMutex<Peers>>,
    _openport: u16,
) {
    println!("Got new block from {}", peer.address);
    let block: Option<Block> = msg
        .payload
        .as_ref()
        .map(|payload| serde_json::from_str(payload).unwrap());

    if let Some(block) = block {
        chain.add_block(block);
    }
}

pub async fn broadcast_new_txn(app_id: String, peers: Arc<FutureMutex<Peers>>, txn: Transaction) {
    println!("Broadcast new block");
    let msg = P2PMessage {
        event: P2PEvent::NewTxnNotified,
        payload: Some(serde_json::to_string(&txn).unwrap()),
        sender_id: app_id.clone(),
    };
    broadcast_message(peers, msg).await;
}

async fn on_new_txn_notified(
    _app_id: String,
    chain: &mut BlockChain,
    peer: &Peer,
    msg: &P2PMessage,
    _peers: Arc<FutureMutex<Peers>>,
    _openport: u16,
) {
    println!("Got new txn from {}", peer.address);
    let txn: Option<Transaction> = msg
        .payload
        .as_ref()
        .map(|payload| serde_json::from_str(payload).unwrap());
    if let Some(txn) = txn {
        chain.add_txn_to_mempool(txn);
    }
}
