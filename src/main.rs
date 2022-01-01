#[macro_use]
extern crate rocket;
use nomadcoin::{Block, BlockChain};
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
    let blocks = chain.all_blocks();
    Json(blocks)
}

#[post("/blocks", data = "<body>")]
fn add_block(body: Json<AddBlockBody>, chain_state: &State<Mutex<BlockChain>>) -> Status {
    let mut chain = chain_state.lock().unwrap();
    chain.add_block(&body.message);
    Status::Created
}

#[get("/blocks/<height>")]
fn get_block(chain_state: &State<Mutex<BlockChain>>, height: usize) -> Option<Json<Block>> {
    let chain = chain_state.lock().unwrap();
    match chain.get_block(height) {
        Some(block) => Some(Json(block)),
        None => None,
    }
}

#[launch]
fn rocket() -> _ {
    let chain = BlockChain::get();
    let chain_mutex = Mutex::new(chain);
    rocket::build()
        .mount(
            "/",
            routes![documentation, add_block, fetch_blocks, get_block],
        )
        .manage(chain_mutex)
}
