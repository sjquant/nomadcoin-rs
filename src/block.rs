use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Transaction;

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
        prev_hash: String,
        height: u64,
        difficulty: u16,
        mempool: &mut Vec<Transaction>,
    ) -> Self {
        let mut nonce: u64 = 0;
        let mut payload;
        let mut hash = String::from("");
        let target = std::iter::repeat("0")
            .take(difficulty.into())
            .collect::<String>();
        let mut txns = vec![];
        let coinbase_txn = Transaction::from_coinbase("todo-address");
        txns.push(coinbase_txn);
        txns.append(mempool);
        loop {
            if hash.starts_with(&target) {
                break;
            }
            payload = format!("{}{}{}{}{:?}", prev_hash, height, difficulty, nonce, txns);
            hash = format!("{:x}", Sha256::digest(payload.as_bytes()));
            nonce += 1;
        }

        Block {
            prev_hash: prev_hash,
            hash: hash,
            height: height,
            difficulty: difficulty,
            nonce: nonce,
            timestamp: Utc::now().timestamp(),
            transactions: txns,
        }
    }
}
