#[macro_use]
extern crate rocket;
use futures::lock::Mutex as FutureMutex;
use nomadcoin::p2p::{Peer, Peers};
use nomadcoin::{transaction::UTxnOut, Block, BlockChain, Transaction, Wallet};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::State;
use std::sync::Mutex;

#[derive(Serialize)]
struct URLDescription {
    url: String,
    method: String,
    description: String,
}

#[derive(Serialize)]
struct BalanceRespone {
    address: String,
    balance: u64,
}

#[derive(Deserialize)]
struct MineBlockBody {
    address: String,
}

#[derive(Deserialize)]
struct MakeTransactionBody {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SSEMessage {
    message: String,
}

fn url(path: &str) -> String {
    format!("http://localhost:8000{}", path)
}

fn get_db() -> PickleDb {
    match PickleDb::load(
        "blockchain.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    ) {
        Ok(load) => load,
        Err(_) => PickleDb::new(
            "blockchain.db",
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        ),
    }
}

#[get("/")]
fn documentation() -> Json<Vec<URLDescription>> {
    let data = vec![
        URLDescription {
            url: url("/"),
            method: String::from("GET"),
            description: String::from("See Documentation"),
        },
        URLDescription {
            url: url("/blocks"),
            method: String::from("GET"),
            description: String::from("See All Blocks"),
        },
        URLDescription {
            url: url("/blocks"),
            method: String::from("POST"),
            description: String::from("Add A Block"),
        },
        URLDescription {
            url: url("/blocks/<hash>"),
            method: String::from("GET"),
            description: String::from("See A Block"),
        },
        URLDescription {
            url: url("/addresses/<address>/txnouts"),
            method: String::from("GET"),
            description: String::from("Get transaction outputs for an address"),
        },
        URLDescription {
            url: url("/addresses/<address>/balance"),
            method: String::from("GET"),
            description: String::from("Get balance for an address"),
        },
        URLDescription {
            url: url("/mempool"),
            method: String::from("GET"),
            description: String::from("Get transactions inside blockchain memory pool"),
        },
        URLDescription {
            url: url("/transactions"),
            method: String::from("POST"),
            description: String::from("Make a transaction"),
        },
        URLDescription {
            url: url("/wallet"),
            method: String::from("GET"),
            description: String::from("See my wallet"),
        },
        URLDescription {
            url: url("/peers"),
            method: String::from("GET"),
            description: String::from("See peers"),
        },
        URLDescription {
            url: url("/peers"),
            method: String::from("POST"),
            description: String::from("Add a peer"),
        },
    ];
    Json(data)
}

#[get("/blocks")]
fn fetch_blocks(chain_state: &State<Mutex<BlockChain>>) -> Json<Vec<Block>> {
    let chain = chain_state.lock().unwrap();
    let mut db = get_db();
    let blocks = chain.all_blocks(&mut db);
    Json(blocks)
}

#[post("/blocks", data = "<body>")]
fn add_block(body: Json<MineBlockBody>, chain_state: &State<Mutex<BlockChain>>) -> Status {
    let mut chain = chain_state.lock().unwrap();
    let mut db = get_db();
    chain.add_block(&mut db, body.address.as_str());
    Status::Created
}

#[get("/blocks/<hash>")]
fn get_block(chain_state: &State<Mutex<BlockChain>>, hash: String) -> Option<Json<Block>> {
    let chain = chain_state.lock().unwrap();
    let mut db = get_db();
    match chain.get_block(&mut db, hash) {
        Some(block) => Some(Json(block)),
        None => None,
    }
}

#[get("/addresses/<address>/txnouts")]
fn fetch_txnouts(chain_state: &State<Mutex<BlockChain>>, address: String) -> Json<Vec<UTxnOut>> {
    let chain = chain_state.lock().unwrap();
    let mut db = get_db();
    let txnouts = chain.unspent_txnouts_by_address(&mut db, address.as_str());
    Json(txnouts)
}

#[get("/addresses/<address>/balance")]
fn get_balance(chain_state: &State<Mutex<BlockChain>>, address: String) -> Json<BalanceRespone> {
    let chain = chain_state.lock().unwrap();
    let mut db = get_db();
    let balance = chain.balance_by_address(&mut db, address.as_str());
    Json(BalanceRespone { address, balance })
}

#[get("/mempool")]
fn mempool(chain_state: &State<Mutex<BlockChain>>) -> Json<Vec<Transaction>> {
    let chain = chain_state.lock().unwrap();
    let mempool = chain.mempool.clone();
    Json(mempool)
}

#[post("/transactions", data = "<body>")]
fn make_transaction(
    body: Json<MakeTransactionBody>,
    chain_state: &State<Mutex<BlockChain>>,
) -> Status {
    let mut chain = chain_state.lock().unwrap();
    let mut db = get_db();
    match chain.make_transaction(&mut db, body.from.as_str(), body.to.as_str(), body.amount) {
        Ok(()) => Status::Created,
        Err(_) => Status::BadRequest,
    }
}

#[get("/my-wallet")]
fn my_wallet() -> String {
    let wallet = Wallet::get("nico.wallet");
    wallet.address
}

#[get("/sse")]
async fn sse_get(queue: &State<Sender<SSEMessage>>) -> EventStream![] {
    println!("Connected");
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                data = rx.recv() => match data {
                    Ok(data) => data.message,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                }
            };
            println!("Got message: {}", msg);
            yield Event::json(&msg);
        }
    }
}

#[post("/sse", data = "<body>")]
async fn sse_post(queue: &State<Sender<SSEMessage>>, body: Json<SSEMessage>) {
    println!("Sent a message: {}", body.message);
    let _ = queue.send(body.into_inner());
}

#[get("/peers")]
async fn peers(peers_state: &State<FutureMutex<Peers>>) -> Json<Vec<String>> {
    let peers = peers_state.lock().await;
    Json(peers.peers.keys().cloned().collect())
}

#[post("/peers", data = "<body>")]
async fn add_peer(peers_state: &State<FutureMutex<Peers>>, body: Json<Peer>) {
    let peer = Peer::new(body.address.as_str());
    println!("Adding peer: {}", peer.address);
    let mut peers = peers_state.lock().await;
    peers.add(peer).await;
}

#[launch]
fn rocket() -> _ {
    let mut db = get_db();
    let chain = BlockChain::get(&mut db);
    let chain_mutex = Mutex::new(chain);
    let queue = channel::<SSEMessage>(1024).0;
    let peers = FutureMutex::new(Peers::new());

    rocket::build()
        .mount(
            "/",
            routes![
                documentation,
                add_block,
                fetch_blocks,
                get_block,
                fetch_txnouts,
                get_balance,
                mempool,
                make_transaction,
                my_wallet,
                sse_get,
                sse_post,
                peers,
                add_peer
            ],
        )
        .manage(chain_mutex)
        .manage(queue)
        .manage(peers)
}
