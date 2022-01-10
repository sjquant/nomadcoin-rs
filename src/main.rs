#[macro_use]
extern crate rocket;
use nomadcoin::{Block, BlockChain};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
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

#[derive(Deserialize)]
struct AddBlockBody {
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
            payload: Some(String::from("data:string")),
        },
        URLDescription {
            url: url("/blocks/<id>"),
            method: String::from("GET"),
            description: String::from("See A Block"),
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

#[post("/blocks", data = "<body>")]
fn add_block(body: Json<AddBlockBody>, chain_state: &State<Mutex<BlockChain>>) -> Status {
    let mut chain = chain_state.lock().unwrap();
    let mut db = get_db();
    chain.add_block(&mut db, body.message.clone());
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

#[launch]
fn rocket() -> _ {
    let mut db = get_db();
    let chain = BlockChain::get(&mut db);
    let chain_mutex = Mutex::new(chain);
    rocket::build()
        .mount(
            "/",
            routes![documentation, add_block, fetch_blocks, get_block],
        )
        .manage(chain_mutex)
}
