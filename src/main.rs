use p256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey,
};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
// #[macro_use]
// extern crate rocket;
// use nomadcoin::{transaction::UTxnOut, Block, BlockChain, Transaction};
// use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
// use rocket::{
//     http::Status,
//     serde::{json::Json, Deserialize, Serialize},
//     State,
// };
// use std::sync::Mutex;

// #[derive(Serialize)]
// struct URLDescription {
//     url: String,
//     method: String,
//     description: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     payload: Option<String>,
// }

// #[derive(Serialize)]
// struct BalanceRespone {
//     address: String,
//     balance: u64,
// }

// #[derive(Deserialize)]
// struct MakeTransactionBody {
//     from: String,
//     to: String,
//     amount: u64,
// }

// fn url(path: &str) -> String {
//     format!("http://localhost:8000{}", path)
// }

// fn get_db() -> PickleDb {
//     match PickleDb::load(
//         "blockchain.db",
//         PickleDbDumpPolicy::AutoDump,
//         SerializationMethod::Json,
//     ) {
//         Ok(load) => load,
//         Err(_) => PickleDb::new(
//             "blockchain.db",
//             PickleDbDumpPolicy::AutoDump,
//             SerializationMethod::Json,
//         ),
//     }
// }

// #[get("/")]
// fn documentation() -> Json<Vec<URLDescription>> {
//     let data = vec![
//         URLDescription {
//             url: url("/"),
//             method: String::from("GET"),
//             description: String::from("See Documentation"),
//             payload: None,
//         },
//         URLDescription {
//             url: url("/blocks"),
//             method: String::from("GET"),
//             description: String::from("See All Blocks"),
//             payload: None,
//         },
//         URLDescription {
//             url: url("/blocks"),
//             method: String::from("POST"),
//             description: String::from("Add A Block"),
//             payload: None,
//         },
//         URLDescription {
//             url: url("/blocks/<id>"),
//             method: String::from("GET"),
//             description: String::from("See A Block"),
//             payload: None,
//         },
//         URLDescription {
//             url: url("/addresses/<address>/txnouts"),
//             method: String::from("GET"),
//             description: String::from("Get transaction outputs for an address"),
//             payload: None,
//         },
//         URLDescription {
//             url: url("/addresses/<address>/balance"),
//             method: String::from("GET"),
//             description: String::from("Get balance for an address"),
//             payload: None,
//         },
//         URLDescription {
//             url: url("/mempool"),
//             method: String::from("GET"),
//             description: String::from("Get transactions inside blockchain memory pool"),
//             payload: None,
//         },
//     ];
//     Json(data)
// }

// #[get("/blocks")]
// fn fetch_blocks(chain_state: &State<Mutex<BlockChain>>) -> Json<Vec<Block>> {
//     let chain = chain_state.lock().unwrap();
//     let mut db = get_db();
//     let blocks = chain.all_blocks(&mut db);
//     Json(blocks)
// }

// #[post("/blocks")]
// fn add_block(chain_state: &State<Mutex<BlockChain>>) -> Status {
//     let mut chain = chain_state.lock().unwrap();
//     let mut db = get_db();
//     chain.add_block(&mut db);
//     Status::Created
// }

// #[get("/blocks/<hash>")]
// fn get_block(chain_state: &State<Mutex<BlockChain>>, hash: String) -> Option<Json<Block>> {
//     let chain = chain_state.lock().unwrap();
//     let mut db = get_db();
//     match chain.get_block(&mut db, hash) {
//         Some(block) => Some(Json(block)),
//         None => None,
//     }
// }

// #[get("/addresses/<address>/txnouts")]
// fn fetch_txnouts(chain_state: &State<Mutex<BlockChain>>, address: String) -> Json<Vec<UTxnOut>> {
//     let chain = chain_state.lock().unwrap();
//     let mut db = get_db();
//     let txnouts = chain.unspent_txnouts_by_address(&mut db, address.as_str());
//     Json(txnouts)
// }

// #[get("/addresses/<address>/balance")]
// fn get_balance(chain_state: &State<Mutex<BlockChain>>, address: String) -> Json<BalanceRespone> {
//     let chain = chain_state.lock().unwrap();
//     let mut db = get_db();
//     let balance = chain.balance_by_address(&mut db, address.as_str());
//     Json(BalanceRespone { address, balance })
// }

// #[get("/mempool")]
// fn mempool(chain_state: &State<Mutex<BlockChain>>) -> Json<Vec<Transaction>> {
//     let chain = chain_state.lock().unwrap();
//     let mempool = chain.mempool.clone();
//     Json(mempool)
// }

// #[post("/transactions", data = "<body>")]
// fn make_transaction(
//     body: Json<MakeTransactionBody>,
//     chain_state: &State<Mutex<BlockChain>>,
// ) -> Status {
//     let mut chain = chain_state.lock().unwrap();
//     let mut db = get_db();
//     match chain.make_transaction(&mut db, body.from.as_str(), body.to.as_str(), body.amount) {
//         Ok(()) => Status::Created,
//         Err(_) => Status::BadRequest,
//     }
// }

// #[launch]
// fn rocket() -> _ {
//     let mut db = get_db();
//     let chain = BlockChain::get(&mut db);
//     let chain_mutex = Mutex::new(chain);
//     rocket::build()
//         .mount(
//             "/",
//             routes![
//                 documentation,
//                 add_block,
//                 fetch_blocks,
//                 get_block,
//                 fetch_txnouts,
//                 get_balance,
//                 mempool,
//                 make_transaction
//             ],
//         )
//         .manage(chain_mutex)
// }

fn main() {
    let SIGNATURE ="3F1383D3E8C23FC3683B5F88A84006741A43DDF19B8925BADFEF79C7FEBA86C3CA6BAD1EFC2873CD2F24245A79906257E419806861137F22ED521216243A3DF2";
    let PRIVATE_KEY = "341561634f901ac1226bb7b69a9d771590138ebf41ae2479820774e8b4363835";
    let HASHED_MESSAGE = "c33084feaa65adbbbebd0c9bf292a26ffc6dea97b170d88e501ab4865591aafd";

    let private_key_as_bytes = &hex::decode(PRIVATE_KEY).unwrap();
    let private_key = SigningKey::from_bytes(private_key_as_bytes).unwrap();
    // let private_key = SigningKey::random(&mut OsRng);
    let hashed_msg = Sha256::digest(b"I love you");
    let signature: Signature = private_key.sign(&hashed_msg);
    let public_key = private_key.verifying_key();
    println!("PrivateKey: {}", hex::encode(private_key.to_bytes()));
    println!("HashedMsg: {}", hex::encode(&hashed_msg));
    println!("Signature: {}", signature.to_string());
    println!("R: {}, S: {}", signature.r(), signature.s());
    match public_key.verify(&Sha256::digest(b"I love you"), &signature) {
        Ok(_) => {
            println!("Verified!")
        }
        Err(_) => {
            println!("Not verified!")
        }
    }
}
