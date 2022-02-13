use p256::ecdsa::{
    signature::{Signature, Verifier},
    VerifyingKey,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    block::Block,
    error::Error,
    transaction::{Transaction, TxnIn, TxnOut, UTxnOut},
    Wallet,
};
use pickledb::PickleDb;

const DIFFICULTY_INTERVAL: u64 = 5;
const TIME_THRESHOLD: i64 = 36000;
const ALLOWED_BUFFER: i64 = 7200;

fn verify_msg(public_key_str: &str, msg: &str, signature_str: &str) -> bool {
    let public_key_as_bytes = &hex::decode(public_key_str).unwrap();
    let public_key = VerifyingKey::from_sec1_bytes(public_key_as_bytes).unwrap();
    let signature_as_bytes = &hex::decode(signature_str).unwrap();
    let signature = Signature::from_bytes(signature_as_bytes).unwrap();
    let msg_as_bytes = &hex::decode(msg).unwrap();
    public_key.verify(msg_as_bytes, &signature).is_ok()
}

#[derive(Serialize, Deserialize)]
pub struct BlockChain {
    pub newest_hash: String,
    pub height: u64,
    pub difficulty: u16,
    pub mempool: Vec<Transaction>,
}

impl BlockChain {
    pub fn get(db: &mut PickleDb) -> Self {
        match db.get::<BlockChain>("checkpoint") {
            Some(blockchain) => blockchain,
            None => {
                let blockchain = BlockChain {
                    newest_hash: String::from(""),
                    height: 0,
                    difficulty: 1,
                    mempool: vec![],
                };
                blockchain.create_checkpoint(db);
                blockchain
            }
        }
    }

    pub fn add_block(&mut self, db: &mut PickleDb, address: &str) {
        let block = Block::mine(
            address,
            self.newest_hash.clone(),
            self.height + 1,
            self.calc_difficulty(db),
            &mut self.mempool,
        );
        db.set(format!("block:{}", block.hash).as_str(), &block)
            .unwrap();
        self.newest_hash = block.hash;
        self.height = block.height;
        self.mempool = vec![];
        self.create_checkpoint(db);
    }

    fn create_checkpoint(&self, db: &mut PickleDb) {
        db.set("checkpoint", self).unwrap();
    }

    pub fn all_blocks(&self, db: &mut PickleDb) -> Vec<Block> {
        let mut hash_cursor = self.newest_hash.clone();
        let mut blocks: Vec<Block> = Vec::new();

        while hash_cursor.as_str() != "" {
            let block = self.get_block(db, hash_cursor).unwrap();
            blocks.push(block.clone());
            hash_cursor = block.prev_hash;
        }
        blocks
    }

    pub fn get_block(&self, db: &mut PickleDb, hash: String) -> Option<Block> {
        db.get::<Block>(format!("block:{}", hash).as_str())
    }

    fn calc_difficulty(&mut self, db: &mut PickleDb) -> u16 {
        if self.height != 0 && self.height % DIFFICULTY_INTERVAL == 0 {
            let all_blocks = self.all_blocks(db);
            let newest_timestamp = all_blocks[0].timestamp;
            let base_timestamp = all_blocks[(DIFFICULTY_INTERVAL - 1) as usize].timestamp;
            let time_taken = newest_timestamp - base_timestamp;
            if time_taken < TIME_THRESHOLD - ALLOWED_BUFFER {
                self.difficulty += 1;
            } else if time_taken > TIME_THRESHOLD + ALLOWED_BUFFER {
                self.difficulty -= 1;
            }
        }
        self.difficulty
    }

    pub fn all_txn_outs(&self, db: &mut PickleDb) -> Vec<TxnOut> {
        let blocks = self.all_blocks(db);
        let mut txn_outs: Vec<TxnOut> = vec![];

        for block in blocks.into_iter() {
            for mut txn in block.transactions.into_iter() {
                txn_outs.append(&mut txn.txn_outs)
            }
        }
        txn_outs
    }

    pub fn balance_by_address(&self, db: &mut PickleDb, address: &str) -> u64 {
        self.unspent_txnouts_by_address(db, address)
            .iter()
            .map(|txn| txn.amount)
            .sum()
    }

    fn is_on_mempool(&self, utxnout: &UTxnOut) -> bool {
        self.mempool.iter().any(|txn| {
            txn.txn_ins
                .iter()
                .any(|txn_in| txn_in.txn_id == utxnout.txn_id && txn_in.idx == utxnout.idx)
        })
    }

    pub fn unspent_txnouts_by_address(&self, db: &mut PickleDb, address: &str) -> Vec<UTxnOut> {
        let mut utxnouts = vec![];
        let mut existing_txn_ids: HashSet<&str> = HashSet::new();
        for block in self.all_blocks(db).iter() {
            for txn in block.transactions.iter() {
                for txn_in in txn.txn_ins.iter() {
                    if txn_in.signature.as_str() == "COINBASE" {
                        break;
                    }

                    if txn.txn_outs[txn_in.idx as usize].address == address {
                        existing_txn_ids.insert(txn_in.txn_id.as_str());
                    }
                }
                for (idx, txn_out) in txn.txn_outs.clone().into_iter().enumerate() {
                    if txn_out.address.as_str() == address
                        && !existing_txn_ids.contains(txn.id.as_str())
                    {
                        let utxnout =
                            UTxnOut::new(&txn.id, idx.try_into().unwrap(), txn_out.amount);
                        if !self.is_on_mempool(&utxnout) {
                            utxnouts.push(utxnout);
                        }
                    }
                }
            }
        }
        utxnouts
    }

    fn get_transaction(&self, db: &mut PickleDb, id: &str) -> Option<Transaction> {
        let blocks = self.all_blocks(db);
        for block in blocks.iter() {
            for txn in block.transactions.iter() {
                if txn.id == id {
                    return Some(txn.clone());
                }
            }
        }
        None
    }

    fn validate_transaction(&self, db: &mut PickleDb, txn: &Transaction) -> bool {
        for txn_in in txn.txn_ins.iter() {
            match self.get_transaction(db, txn_in.txn_id.as_str()) {
                Some(prev_txn) => {
                    let address = prev_txn.txn_outs[txn_in.idx as usize].address.as_str();
                    let signature = txn_in.signature.as_str();
                    let msg = hex::encode(&txn.id);
                    if !verify_msg(address, msg.as_str(), signature) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    pub fn make_transaction(
        &mut self,
        db: &mut PickleDb,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<(), Error> {
        if self.balance_by_address(db, from) < amount {
            Err(Error::new("Not enough balance"))
        } else {
            let utxn_outs = self.unspent_txnouts_by_address(db, from);
            let mut txn_ins: Vec<TxnIn> = vec![];
            let mut txn_outs: Vec<TxnOut> = vec![];
            let mut total = 0;
            for utxnout in utxn_outs.into_iter() {
                if total >= amount {
                    break;
                }
                txn_ins.push(TxnIn::new(&utxnout.txn_id, utxnout.idx, utxnout.amount));
                total += utxnout.amount;
            }
            // Bring changes back to transaction sender
            if total > amount {
                txn_outs.push(TxnOut::new(from, total - amount));
            }
            txn_outs.push(TxnOut::new(to, amount));
            let mut transaction = Transaction::new(txn_ins, txn_outs);
            transaction.sign(Wallet::get("nico.wallet"));
            if !self.validate_transaction(db, &transaction) {
                return Err(Error::new("Invalid transaction"));
            };
            self.mempool.push(transaction);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;

    #[test]
    fn get_new_blockchain() {
        let (_r, mut db) = test_utils::test_db();
        let chain = BlockChain::get(&mut db);
        assert_eq!(chain.newest_hash, "");
        assert_eq!(chain.height, 0);
    }

    #[test]
    fn get_blockchain_from_db() {
        // Given
        let (_r, mut db) = test_utils::test_db();

        {
            let mut chain = BlockChain::get(&mut db);
            chain.add_block(&mut db, "some-address");
            chain.add_block(&mut db, "some-address");
        }

        // When
        let chain = BlockChain::get(&mut db);

        // Then
        assert_eq!(chain.height, 2);
    }

    #[test]
    fn add_block() {
        // Given
        let (_r, mut db) = test_utils::test_db();

        // When
        let mut chain = BlockChain::get(&mut db);
        chain.add_block(&mut db, "some-address");
        chain.add_block(&mut db, "some-address");

        // Then
        let blocks = chain.all_blocks(&mut db);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn make_transaction() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        let wallet = Wallet::get("nico.wallet");
        let address = wallet.address.as_str();
        chain.add_block(&mut db, address); // Earn 50 by mining block

        // When
        chain
            .make_transaction(&mut db, address, "to-address", 20)
            .unwrap();

        // Then
        let mem_txn = chain.mempool[0].clone();
        assert_eq!(chain.balance_by_address(&mut db, address), 0); // balance not yet changed
        assert_eq!(mem_txn.txn_ins[0].amount, 50);
        assert_eq!(mem_txn.txn_outs[0].address, String::from(address));
        assert_eq!(mem_txn.txn_outs[0].amount, 30);
        assert_eq!(mem_txn.txn_outs[1].address, String::from("to-address"));
        assert_eq!(mem_txn.txn_outs[1].amount, 20);
    }

    #[test]
    fn add_block_confirms_transaction() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        let wallet = Wallet::get("nico.wallet");
        let address = wallet.address.as_str();
        chain.add_block(&mut db, address); // Earn 50 by mining block
        chain
            .make_transaction(&mut db, address, "john", 20)
            .unwrap();

        // When
        chain.add_block(&mut db, address); // Earn another 50 by mining block

        // Then
        assert_eq!(chain.balance_by_address(&mut db, address), 80);
        assert_eq!(chain.balance_by_address(&mut db, "john"), 20);
        assert_eq!(chain.mempool.len(), 0);
    }

    #[test]
    fn cannot_make_transaction_when_balance_is_not_enough() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        let wallet = Wallet::get("nico.wallet");
        let address = wallet.address.as_str();

        // Earn 50 by mining block
        chain.add_block(&mut db, address);

        // When
        let err = chain
            .make_transaction(&mut db, address, "to-address", 60)
            .unwrap_err();

        // Then
        assert_eq!(err.msg, String::from("Not enough balance"));
    }
}
