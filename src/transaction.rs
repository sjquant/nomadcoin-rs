use std::vec;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::Wallet;

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
        let mut coinbase_txn_in = TxnIn::new("", -1, MINER_REWARD);
        coinbase_txn_in.set_signature("COINBASE");
        let txn_ins = vec![coinbase_txn_in];
        let txn_outs = vec![TxnOut::new(address, MINER_REWARD)];
        Transaction::new(txn_ins, txn_outs)
    }

    pub fn new(txn_ins: Vec<TxnIn>, txn_outs: Vec<TxnOut>) -> Self {
        let timestamp = Utc::now().timestamp();
        Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp,
            txn_ins,
            txn_outs,
        }
    }

    pub fn sign(&mut self, wallet: Wallet) {
        let msg = hex::encode(&self.id);
        let signature = wallet.sign(msg.as_str());
        for txn_in in &mut self.txn_ins {
            txn_in.set_signature(&signature);
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TxnIn {
    pub txn_id: String,
    pub idx: i64,
    pub amount: u64,
    pub signature: String,
}

impl TxnIn {
    pub fn new(txn_id: &str, idx: i64, amount: u64) -> Self {
        Self {
            txn_id: txn_id.to_string(),
            idx: idx,
            signature: String::from(""), // Unsignd yet
            amount: amount,
        }
    }

    pub fn set_signature(&mut self, signature: &str) {
        self.signature = signature.to_string();
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TxnOut {
    pub address: String,
    pub amount: u64,
}

impl TxnOut {
    pub fn new(address: &str, amount: u64) -> Self {
        Self {
            address: address.to_string(),
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
