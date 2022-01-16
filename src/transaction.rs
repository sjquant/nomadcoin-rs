use std::vec;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MINER_REWARD: u64 = 50;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub id: String,
    pub timestamp: i64,
    pub txn_ins: Vec<TxnIn>,
    pub txn_outs: Vec<TxnOut>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TxnIn {
    pub owner: String,
    pub amount: u64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TxnOut {
    pub owner: String,
    pub amount: u64,
}

impl Transaction {
    pub fn from_coinbase(address: String) -> Self {
        let txn_ins = vec![TxnIn {
            owner: String::from("COINBASE"),
            amount: MINER_REWARD,
        }];
        let txn_outs = vec![TxnOut {
            owner: address,
            amount: MINER_REWARD,
        }];

        let timestamp = Utc::now().timestamp();
        let payload = format!("{:?}{:?}{}", txn_ins, txn_outs, timestamp);
        Transaction {
            id: format!("{:x}", Sha256::digest(payload.as_bytes())),
            timestamp: timestamp,
            txn_ins: txn_ins,
            txn_outs: txn_outs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coinbase_transaction() {
        let txn = Transaction::from_coinbase(String::from("my-address"));
        assert_eq!(
            txn,
            Transaction {
                id: txn.id.clone(),
                timestamp: txn.timestamp,
                txn_ins: vec![TxnIn {
                    owner: String::from("COINBASE"),
                    amount: MINER_REWARD,
                }],
                txn_outs: vec![TxnOut {
                    owner: String::from("my-address"),
                    amount: MINER_REWARD,
                }]
            }
        );
    }
}
