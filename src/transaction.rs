use std::vec;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{hashable::Hashable, Wallet};

const MINER_REWARD: u64 = 50;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub hash: String,
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
        let mut txn = Transaction::new(txn_ins, txn_outs);
        txn.hash = txn.hash();
        txn
    }

    pub fn new(txn_ins: Vec<TxnIn>, txn_outs: Vec<TxnOut>) -> Self {
        let timestamp = Utc::now().timestamp_nanos();
        let mut txn = Transaction {
            hash: String::from(""),
            timestamp,
            txn_ins,
            txn_outs,
        };
        txn.hash = txn.hash();
        txn
    }

    pub fn sign(&mut self, wallet: &Wallet) {
        let msg = hex::encode(&self.hash);
        let signature = wallet.sign(msg.as_str());
        for txn_in in &mut self.txn_ins {
            txn_in.set_signature(&signature);
        }
    }
}

impl Hashable for Transaction {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.append(&mut self.timestamp.to_le_bytes().to_vec());
        bytes.append(
            &mut self
                .txn_ins
                .iter()
                .flat_map(|txn_in| txn_in.bytes())
                .collect::<Vec<u8>>(),
        );
        bytes.append(
            &mut self
                .txn_outs
                .iter()
                .flat_map(|txn_out| txn_out.bytes())
                .collect::<Vec<u8>>(),
        );
        bytes
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct TxnIn {
    pub txn_hash: String,
    pub idx: i64,
    pub amount: u64,
    pub signature: String,
}

impl TxnIn {
    pub fn new(txn_hash: &str, idx: i64, amount: u64) -> Self {
        Self {
            txn_hash: txn_hash.to_string(),
            idx: idx,
            signature: String::from(""), // Unsignd yet
            amount: amount,
        }
    }

    pub fn set_signature(&mut self, signature: &str) {
        self.signature = signature.to_string();
    }
}

impl Hashable for TxnIn {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.append(&mut self.txn_hash.clone().into_bytes());
        bytes.append(&mut self.idx.to_le_bytes().to_vec());
        bytes.append(&mut self.amount.to_le_bytes().to_vec());
        bytes.append(&mut self.signature.clone().into_bytes());
        bytes
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
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

impl Hashable for TxnOut {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.append(&mut self.address.clone().into_bytes());
        bytes.append(&mut self.amount.to_le_bytes().to_vec());
        bytes
    }
}

// Unspent Transaction Out
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct UTxnOut {
    pub txn_hash: String,
    pub idx: i64,
    pub amount: u64,
}

impl UTxnOut {
    pub fn new(txn_hash: &str, idx: i64, amount: u64) -> Self {
        Self {
            txn_hash: txn_hash.to_string(),
            idx: idx,
            amount: amount,
        }
    }
}
