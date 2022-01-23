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

impl Transaction {
    pub fn from_coinbase(address: &str) -> Self {
        let txn_ins = vec![TxnIn::new("", -1, "COINBASE", MINER_REWARD)];
        let txn_outs = vec![TxnOut::new(address, MINER_REWARD)];
        Transaction::new(txn_ins, txn_outs)
    }

    pub fn new(txn_ins: Vec<TxnIn>, txn_outs: Vec<TxnOut>) -> Self {
        let timestamp = Utc::now().timestamp();
        let payload = format!("{:?}{:?}{}", txn_ins, txn_outs, timestamp);
        let hash = format!("{:x}", Sha256::digest(payload.as_bytes()));
        Transaction {
            id: hash,
            timestamp,
            txn_ins,
            txn_outs,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TxnIn {
    pub txn_id: String,
    pub idx: i64,
    pub owner: String,
    pub amount: u64,
}

impl TxnIn {
    pub fn new(txn_id: &str, idx: i64, owner: &str, amount: u64) -> Self {
        Self {
            txn_id: txn_id.to_string(),
            idx: idx,
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

// Unspent Transaction Out
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct UTxnOut {
    pub txn_id: String,
    pub idx: i64,
    pub amount: u64,
}

impl UTxnOut {
    pub fn new(txn_id: &str, idx: i64, amount: u64) -> Self {
        Self {
            txn_id: txn_id.to_string(),
            idx: idx,
            amount: amount,
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
                txn_ins: vec![TxnIn::new("", -1, "COINBASE", MINER_REWARD)],
                txn_outs: vec![TxnOut::new("my-address", MINER_REWARD)]
            }
        );
    }
}
