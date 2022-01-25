use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    block::Block,
    error::Error,
    repo,
    transaction::{Transaction, TxnIn, TxnOut, UTxnOut},
};
use pickledb::PickleDb;

const DIFFICULTY_INTERVAL: u64 = 5;
const TIME_THRESHOLD: i64 = 36000;
const ALLOWED_BUFFER: i64 = 7200;

#[derive(Serialize, Deserialize)]
pub struct BlockChain {
    pub newest_hash: String,
    pub height: u64,
    pub difficulty: u16,
    pub mempool: Vec<Transaction>,
}

impl BlockChain {
    pub fn get(db: &mut PickleDb) -> Self {
        match repo::checkpoint(db) {
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

    pub fn add_block(&mut self, db: &mut PickleDb) {
        let block = Block::mine(
            self.newest_hash.clone(),
            self.height + 1,
            self.calc_difficulty(db),
            &mut self.mempool,
        );
        repo::save_block(db, block.hash.clone(), &block);
        self.newest_hash = block.hash;
        self.height = block.height;
        self.mempool = vec![];
        self.create_checkpoint(db);
    }

    fn create_checkpoint(&self, db: &mut PickleDb) {
        repo::save_blockchain(db, self);
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
        repo::get_block(db, hash)
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
                    if txn_in.owner.as_str() == address {
                        existing_txn_ids.insert(txn_in.txn_id.as_str());
                    }
                }
                for (idx, txn_out) in txn.txn_outs.clone().into_iter().enumerate() {
                    if txn_out.owner.as_str() == address
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
                txn_ins.push(TxnIn::new(
                    &utxnout.txn_id,
                    utxnout.idx,
                    from,
                    utxnout.amount,
                ));
                total += utxnout.amount;
            }
            // Bring changes back to transaction sender
            if total > amount {
                txn_outs.push(TxnOut::new(from, total - amount));
            }
            txn_outs.push(TxnOut::new(to, amount));
            let transaction = Transaction::new(txn_ins, txn_outs);
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
            chain.add_block(&mut db);
            chain.add_block(&mut db);
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
        chain.add_block(&mut db);
        chain.add_block(&mut db);

        // Then
        let blocks = chain.all_blocks(&mut db);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn make_transaction() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        chain.add_block(&mut db); // Earn 50 by mining block

        // When
        chain
            .make_transaction(&mut db, "todo-address", "john", 20)
            .unwrap();

        // Then
        let mempool = chain.mempool[0].clone();
        assert_eq!(chain.balance_by_address(&mut db, "todo-address"), 0); // balance not yet changed
        assert_eq!(mempool.txn_ins[0].owner, String::from("todo-address"));
        assert_eq!(mempool.txn_ins[0].amount, 50);
        assert_eq!(mempool.txn_outs[0].owner, String::from("todo-address"));
        assert_eq!(mempool.txn_outs[0].amount, 30);
        assert_eq!(mempool.txn_outs[1].owner, String::from("john"));
        assert_eq!(mempool.txn_outs[1].amount, 20);
    }

    #[test]
    fn add_block_confirms_transaction() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        chain.add_block(&mut db); // Earn 50 by mining block
        chain
            .make_transaction(&mut db, "todo-address", "john", 20)
            .unwrap();

        // When
        chain.add_block(&mut db); // Earn another 50 by mining block

        // Then
        assert_eq!(chain.balance_by_address(&mut db, "todo-address"), 80);
        assert_eq!(chain.balance_by_address(&mut db, "john"), 20);
        assert_eq!(chain.mempool.len(), 0);
    }

    #[test]
    fn cannot_make_transaction_when_balance_is_not_enough() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        // Earn 50 by mining block
        chain.add_block(&mut db);

        // When
        let err = chain
            .make_transaction(&mut db, "todo-address", "TO", 60)
            .unwrap_err();

        // Then
        assert_eq!(err.msg, String::from("Not enough balance"));
    }
}
