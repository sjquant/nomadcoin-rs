use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{hashable::Hashable, Transaction};

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone)]
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
        let mut hash = String::from("");
        let target = std::iter::repeat("0")
            .take(difficulty.into())
            .collect::<String>();
        let txns = create_txns(address, mempool);
        let mut block = Block {
            prev_hash: prev_hash.to_string(),
            hash: "".to_string(),
            height: height,
            difficulty: difficulty,
            nonce: 0,
            timestamp: Utc::now().timestamp(),
            transactions: txns,
        };

        loop {
            if hash.starts_with(&target) {
                block.hash = hash;
                break;
            }
            hash = block.hash();
            block.nonce += 1;
        }
        block
    }
}

impl Hashable for Block {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.append(&mut self.prev_hash.to_string().into_bytes());
        bytes.append(&mut self.height.to_le_bytes().to_vec());
        bytes.append(&mut self.difficulty.to_le_bytes().to_vec());
        bytes.append(&mut self.nonce.to_le_bytes().to_vec());
        bytes.append(
            &mut self
                .transactions
                .iter()
                .flat_map(|txn| txn.bytes())
                .collect::<Vec<u8>>(),
        );
        bytes
    }
}

fn create_txns(address: &str, mempool: &mut Vec<Transaction>) -> Vec<Transaction> {
    let mut txns = vec![];
    let coinbase_txn = Transaction::from_coinbase(address);
    txns.push(coinbase_txn);
    txns.append(mempool);
    txns
}
