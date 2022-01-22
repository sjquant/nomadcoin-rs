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

impl TxnIn {
    pub fn new(owner: &str, amount: u64) -> Self {
        Self {
            owner: owner.to_string(),
            amount: amount,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TxnOut {
    pub owner: String,
    pub amount: u64,
}

impl TxnOut {
    pub fn new(owner: &str, amount: u64) -> Self {
        Self {
            owner: owner.to_string(),
            amount: amount,
        }
    }
}

impl Transaction {
    pub fn from_coinbase(address: &str) -> Self {
        let txn_ins = vec![TxnIn::new("COINBASE", MINER_REWARD)];
        let txn_outs = vec![TxnOut::new(address, MINER_REWARD)];
        Transaction::new(txn_ins, txn_outs)
    }

    pub fn new(txn_ins: Vec<TxnIn>, txn_outs: Vec<TxnOut>) -> Self {
        let timestamp = Utc::now().timestamp();
        let payload = format!("{:?}{:?}{}", txn_ins, txn_outs, timestamp);
        let id = format!("{:x}", Sha256::digest(payload.as_bytes()));

        Transaction {
            id,
            timestamp,
            txn_ins,
            txn_outs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coinbase_transaction() {
        let txn = Transaction::from_coinbase("my-address");
        assert_eq!(
            txn,
            Transaction {
                id: txn.id.clone(),
                timestamp: txn.timestamp,
                txn_ins: vec![TxnIn::new("COINBASE", MINER_REWARD)],
                txn_outs: vec![TxnOut::new("my-address", MINER_REWARD)]
            }
        );
    }
}
