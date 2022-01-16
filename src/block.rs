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
    pub fn mine(prev_hash: String, height: u64, difficulty: u16) -> Self {
        let mut nonce: u64 = 0;
        let mut payload;
        let mut hash = String::from("");
        let target = std::iter::repeat("0")
            .take(difficulty.into())
            .collect::<String>();
        let txn = Transaction::from_coinbase(String::from("todo-address"));
        loop {
            if hash.starts_with(&target) {
                break;
            }
            payload = format!("{}{}{}{}{}", txn.id, prev_hash, height, difficulty, nonce);
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
            transactions: vec![txn],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_block() {
        let block = Block::mine(String::from("a-prev-hash"), 10, 2);
        assert_eq!(
            block,
            Block {
                prev_hash: String::from("a-prev-hash"),
                hash: block.hash.clone(),
                height: 10,
                difficulty: 2,
                nonce: block.nonce,
                timestamp: block.timestamp,
                transactions: block.transactions.clone()
            }
        );
    }
}
