#[macro_use]
extern crate rocket;
use futures::lock::Mutex;
use nomadcoin::p2p::{
    add_peer_to_peers, broadcast_new_block, handle_message, P2PMessage, Peer, Peers,
};
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
use uuid::Uuid;

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

#[derive(Deserialize)]
struct AddPeerBody {
    address: String,
}

fn url(path: &str) -> String {
    format!("http://localhost:8000{}", path)
}

fn get_db() -> PickleDb {
    let port = std::env::var("ROCKET_PORT").unwrap();
    let db_path = format!("blockchain_{}.db", port);
    match PickleDb::load(
        db_path.as_str(),
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    ) {
        Ok(load) => load,
        Err(_) => PickleDb::new(
            db_path.as_str(),
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
        URLDescription {
            url: url("/app-id"),
            method: String::from("Get"),
            description: String::from("Get a app id"),
        },
    ];
    Json(data)
}

#[get("/blocks")]
async fn fetch_blocks(chain_state: &State<Arc<Mutex<BlockChain>>>) -> Json<Vec<Block>> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let blocks = chain.all_blocks(&mut db);
    Json(blocks)
}

#[post("/blocks", data = "<body>")]
async fn add_block(
    body: Json<MineBlockBody>,
    chain_state: &State<Arc<Mutex<BlockChain>>>,
    peers_state: &State<Arc<Mutex<Peers>>>,
    app_id_state: &State<String>,
) -> Status {
    let mut chain = chain_state.lock().await;
    let mut db = get_db();
    let block = chain.mine_block(&mut db, body.address.as_str());
    broadcast_new_block(
        app_id_state.inner().clone(),
        peers_state.inner().clone(),
        block,
    )
    .await;
    Status::Created
}

#[get("/blocks/<hash>")]
async fn get_block(
    chain_state: &State<Arc<Mutex<BlockChain>>>,
    hash: String,
) -> Option<Json<Block>> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    match chain.get_block(&mut db, hash) {
        Some(block) => Some(Json(block)),
        None => None,
    }
}

#[get("/addresses/<address>/txnouts")]
async fn fetch_txnouts(
    chain_state: &State<Arc<Mutex<BlockChain>>>,
    address: String,
) -> Json<Vec<UTxnOut>> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let txnouts = chain.unspent_txnouts_by_address(&mut db, address.as_str());
    Json(txnouts)
}

#[get("/addresses/<address>/balance")]
async fn get_balance(
    chain_state: &State<Arc<Mutex<BlockChain>>>,
    address: String,
) -> Json<BalanceRespone> {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let balance = chain.balance_by_address(&mut db, address.as_str());
    Json(BalanceRespone { address, balance })
}

#[get("/mempool")]
async fn mempool(chain_state: &State<Arc<Mutex<BlockChain>>>) -> Json<Vec<Transaction>> {
    let chain = chain_state.lock().await;
    let mempool = chain.mempool.clone();
    Json(mempool)
}

#[post("/transactions", data = "<body>")]
async fn make_transaction(
    body: Json<MakeTransactionBody>,
    chain_state: &State<Arc<Mutex<BlockChain>>>,
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
    chain_state: &State<Arc<Mutex<BlockChain>>>,
    peers_state: &State<Arc<Mutex<Peers>>>,
    queue: &State<Sender<P2PMessage>>,
    shutdown: Shutdown,
    client_addr: IpAddr,
    rocket_config: &rocket::Config,
    app_id_state: &State<String>,
    openport: u16,
) -> EventStream![] {
    let peer_addr = format!("{}:{}", client_addr, openport);
    let peer_id = get_peer_id_from_address(peer_addr.as_str()).await;
    let peer = Peer::new(peer_id.as_str(), peer_addr.as_str());
    let mut rx = queue.subscribe();
    let app_id = app_id_state.inner().clone();

    {
        let mut db = get_db();
        let chain = chain_state.lock().await;
        add_peer_to_peers(
            app_id.clone(),
            &chain,
            &mut db,
            peers_state.inner().clone(),
            &peer,
            rocket_config.port,
        )
        .await;
    }

    let cloned_chain = chain_state.inner().clone();
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
            let app_id = app_id.clone();
            if msg.sender_id != peer.id {
                continue
            }
            let mut chain = cloned_chain.lock().await;
            let mut db = get_db();
            handle_message(app_id, &mut chain, &mut db, &peer, &msg).await;
            yield Event::json(&msg.event);
        }
    }
}

#[post("/sse", data = "<body>")]
async fn sse_post(queue: &State<Sender<P2PMessage>>, body: Json<P2PMessage>) {
    let msg = body.into_inner();
    let _ = queue.send(msg);
}

#[get("/peers")]
async fn peers(peers_state: &State<Arc<Mutex<Peers>>>) -> Json<Vec<Peer>> {
    let peers = peers_state.lock().await;
    Json(peers.all())
}

#[post("/peers", data = "<body>")]
async fn add_peer(
    chain_state: &State<Arc<Mutex<BlockChain>>>,
    peers_state: &State<Arc<Mutex<Peers>>>,
    app_id_state: &State<String>,
    rocket_config: &rocket::Config,
    body: Json<AddPeerBody>,
) {
    let chain = chain_state.lock().await;
    let mut db = get_db();
    let peer_address = body.address.as_str();
    let peer_id = get_peer_id_from_address(peer_address).await;
    let peer = Peer::new(peer_id.as_str(), peer_address);
    println!("Adding peer: {}", peer_address);
    add_peer_to_peers(
        app_id_state.inner().clone(),
        &chain,
        &mut db,
        peers_state.inner().clone(),
        &peer,
        rocket_config.port,
    )
    .await;
}

#[get("/app-id")]
async fn app_id(app_id_state: &State<String>) -> String {
    app_id_state.inner().clone()
}

async fn get_peer_id_from_address(address: &str) -> String {
    reqwest::get(format!("http://{}/app-id", address))
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}

#[launch]
fn rocket() -> _ {
    let mut db = get_db();
    let chain = Arc::new(Mutex::new(BlockChain::get(&mut db)));
    let queue = channel::<P2PMessage>(1024).0;
    let peers = Arc::new(Mutex::new(Peers::new()));
    let app_id = Uuid::new_v4().to_string();

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
                add_peer,
                app_id
            ],
        )
        .manage(chain)
        .manage(queue)
        .manage(peers)
        .manage(app_id)
}
