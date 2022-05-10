use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{utils, Transaction};

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone)]
pub struct Block {
    pub hash: String,
    pub prev_hash: String,
    pub height: u64,
    pub difficulty: u16,
    pub nonce: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn mine(
        address: &str,
        prev_hash: &str,
        height: u64,
        difficulty: u16,
        mempool: &mut Vec<Transaction>,
    ) -> Self {
        let mut nonce: u64 = 0;
        let mut hash = String::from("");
        let target = std::iter::repeat("0")
            .take(difficulty.into())
            .collect::<String>();
        let txns = create_txns(address, mempool);
        loop {
            if hash.starts_with(&target) {
                break;
            }
            hash = utils::hash(&block_bytes(
                address, prev_hash, height, difficulty, nonce, &txns,
            ));
            nonce += 1;
        }

        Block {
            prev_hash: prev_hash.to_string(),
            hash: hash,
            height: height,
            difficulty: difficulty,
            nonce: nonce,
            timestamp: Utc::now().timestamp(),
            transactions: txns,
        }
    }
}

fn block_bytes(
    address: &str,
    prev_hash: &str,
    height: u64,
    difficulty: u16,
    nonce: u64,
    txns: &Vec<Transaction>,
) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.append(&mut address.to_string().into_bytes());
    bytes.append(&mut prev_hash.to_string().into_bytes());
    bytes.append(&mut height.to_le_bytes().to_vec());
    bytes.append(&mut difficulty.to_le_bytes().to_vec());
    bytes.append(&mut nonce.to_le_bytes().to_vec());
    bytes.append(&mut txns.iter().flat_map(|txn| txn.bytes()).collect::<Vec<u8>>());
    bytes
}

fn create_txns(address: &str, mempool: &mut Vec<Transaction>) -> Vec<Transaction> {
    let mut txns = vec![];
    let coinbase_txn = Transaction::from_coinbase(address);
    txns.push(coinbase_txn);
    txns.append(mempool);
    txns
}
