#[macro_use]
extern crate rocket;
use nomadcoin::BlockChain;
use rocket::serde::{json::Json, Serialize};
use std::sync::Mutex;

#[derive(Serialize)]
struct URLDescription {
    url: String,
    method: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<String>,
}

#[get("/")]
fn documentation() -> Json<Vec<URLDescription>> {
    let data = vec![
        URLDescription {
            url: "/".to_string(),
            method: "Get".to_string(),
            description: "See Documentation".to_string(),
            payload: None,
        },
        URLDescription {
            url: "/blocks".to_string(),
            method: "POST".to_string(),
            description: "Add A Block".to_string(),
            payload: Some("data:string".to_string()),
        },
    ];
    Json(data)
}

#[launch]
fn rocket() -> _ {
    let chain = BlockChain::new();
    let chain_mutex = Mutex::new(chain);
    rocket::build()
        .mount("/", routes![documentation])
        .manage(chain_mutex)
}
