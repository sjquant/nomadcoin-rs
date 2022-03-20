#[macro_use]
extern crate rocket;
use futures::lock::Mutex;
use nomadcoin::p2p::{add_peer_to_peers, handle_message, P2PMessage, Peer, Peers};
use nomadcoin::{transaction::UTxnOut, Block, BlockChain, Transaction, Wallet};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{routes, Shutdown, State};
use std::net::IpAddr;
use std::sync::Arc;

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
async fn documentation() -> Json<Vec<URLDescription>> {
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
async fn fetch_blocks(chain_state: &State<Mutex<BlockChain>>) -> Json<Vec<Block>> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let blocks = chain.all_blocks(&mut db);
    Json(blocks)
}

#[post("/blocks", data = "<body>")]
async fn add_block(body: Json<MineBlockBody>, chain_state: &State<Mutex<BlockChain>>) -> Status {
    let mut chain = chain_state.lock().await;
    let mut db = get_db();
    chain.add_block(&mut db, body.address.as_str());
    Status::Created
}

#[get("/blocks/<hash>")]
async fn get_block(chain_state: &State<Mutex<BlockChain>>, hash: String) -> Option<Json<Block>> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    match chain.get_block(&mut db, hash) {
        Some(block) => Some(Json(block)),
        None => None,
    }
}

#[get("/addresses/<address>/txnouts")]
async fn fetch_txnouts(
    chain_state: &State<Mutex<BlockChain>>,
    address: String,
) -> Json<Vec<UTxnOut>> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let txnouts = chain.unspent_txnouts_by_address(&mut db, address.as_str());
    Json(txnouts)
}

#[get("/addresses/<address>/balance")]
async fn get_balance(
    chain_state: &State<Mutex<BlockChain>>,
    address: String,
) -> Json<BalanceRespone> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let balance = chain.balance_by_address(&mut db, address.as_str());
    Json(BalanceRespone { address, balance })
}

#[get("/mempool")]
async fn mempool(chain_state: &State<Mutex<BlockChain>>) -> Json<Vec<Transaction>> {
    let chain = chain_state.lock().await;
    let mempool = chain.mempool.clone();
    Json(mempool)
}

#[post("/transactions", data = "<body>")]
async fn make_transaction(
    body: Json<MakeTransactionBody>,
    chain_state: &State<Mutex<BlockChain>>,
) -> Status {
    let mut chain = chain_state.lock().await;
    let mut db = get_db();
    match chain.make_transaction(&mut db, body.from.as_str(), body.to.as_str(), body.amount) {
        Ok(()) => Status::Created,
        Err(_) => Status::BadRequest,
    }
}

#[get("/my-wallet")]
async fn my_wallet() -> String {
    let wallet = Wallet::get("nico.wallet");
    wallet.address
}

#[get("/sse?<openport>")]
async fn sse_get(
    chain_state: &State<Mutex<BlockChain>>,
    peers_state: &State<Arc<Mutex<Peers>>>,
    queue: &State<Sender<P2PMessage>>,
    shutdown: Shutdown,
    client_addr: IpAddr,
    rocket_config: &rocket::Config,
    openport: Option<u16>,
) -> EventStream![] {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let mut rx = queue.subscribe();

    if let Some(openport) = openport {
        let addr = format!("{}:{}", client_addr, openport);
        let peer = Peer::new(addr.as_str());
        add_peer_to_peers(
            &chain,
            &mut db,
            peers_state.inner().clone(),
            &peer,
            rocket_config.port,
        )
        .await;
    }

    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = shutdown.clone() => {
                    println!("shutdown");
                    break;
                }
            };
            handle_message(&msg).await;
            yield Event::json(&msg.event);
        }
    }
}

#[post("/sse", data = "<body>")]
async fn sse_post(queue: &State<Sender<P2PMessage>>, body: Json<P2PMessage>) {
    let msg = body.into_inner();
    println!("Sent a message: {:?}", msg);
    let _ = queue.send(msg);
}

#[get("/peers")]
async fn peers(peers_state: &State<Arc<Mutex<Peers>>>) -> Json<Vec<String>> {
    let peers = peers_state.lock().await;
    Json(peers.addresses())
}

#[post("/peers", data = "<body>")]
async fn add_peer(
    chain_state: &State<Mutex<BlockChain>>,
    peers_state: &State<Arc<Mutex<Peers>>>,
    rocket_config: &rocket::Config,
    body: Json<Peer>,
) {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let peer = Peer::new(body.address.as_str());
    println!("Adding peer: {}", peer.address);
    add_peer_to_peers(
        &chain,
        &mut db,
        peers_state.inner().clone(),
        &peer,
        rocket_config.port,
    )
    .await;
}

#[launch]
fn rocket() -> _ {
    let mut db = get_db();
    let chain = BlockChain::get(&mut db);
    let chain_mutex = Mutex::new(chain);
    let queue = channel::<P2PMessage>(1024).0;
    let peers = Arc::new(Mutex::new(Peers::new()));
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
