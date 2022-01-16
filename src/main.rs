#[macro_use]
extern crate rocket;
use nomadcoin::{transaction::TxnOut, Block, BlockChain};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rocket::{
    http::Status,
    serde::{json::Json, Serialize},
    State,
};
use std::sync::Mutex;

#[derive(Serialize)]
struct URLDescription {
    url: String,
    method: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<String>,
}

#[derive(Serialize)]
struct BalanceRespone {
    address: String,
    balance: u64,
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
            payload: None,
        },
        URLDescription {
            url: url("/blocks"),
            method: String::from("GET"),
            description: String::from("See All Blocks"),
            payload: None,
        },
        URLDescription {
            url: url("/blocks"),
            method: String::from("POST"),
            description: String::from("Add A Block"),
            payload: None,
        },
        URLDescription {
            url: url("/blocks/<id>"),
            method: String::from("GET"),
            description: String::from("See A Block"),
            payload: None,
        },
        URLDescription {
            url: url("/addresses/<address>/txnouts"),
            method: String::from("GET"),
            description: String::from("Get transaction outputs for an address"),
            payload: None,
        },
        URLDescription {
            url: url("/addresses/<address>/balance"),
            method: String::from("GET"),
            description: String::from("Get balance for an address"),
            payload: None,
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

#[post("/blocks")]
fn add_block(chain_state: &State<Mutex<BlockChain>>) -> Status {
    let mut chain = chain_state.lock().unwrap();
    let mut db = get_db();
    chain.add_block(&mut db);
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
fn fetch_txnouts(chain_state: &State<Mutex<BlockChain>>, address: String) -> Json<Vec<TxnOut>> {
    let chain = chain_state.lock().unwrap();
    let mut db = get_db();
    let txnouts = chain.txn_outs_by_address(&mut db, address.clone());
    Json(txnouts)
}

#[get("/addresses/<address>/balance")]
fn get_balance(chain_state: &State<Mutex<BlockChain>>, address: String) -> Json<BalanceRespone> {
    let chain = chain_state.lock().unwrap();
    let mut db = get_db();
    let balance = chain.balance_by_address(&mut db, address.clone());
    Json(BalanceRespone { address, balance })
}

#[launch]
fn rocket() -> _ {
    let mut db = get_db();
    let chain = BlockChain::get(&mut db);
    let chain_mutex = Mutex::new(chain);
    rocket::build()
        .mount(
            "/",
            routes![
                documentation,
                add_block,
                fetch_blocks,
                get_block,
                fetch_txnouts,
                get_balance
            ],
        )
        .manage(chain_mutex)
}
