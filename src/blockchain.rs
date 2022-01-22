use serde::{Deserialize, Serialize};

use crate::{
    block::Block,
    error::Error,
    repo,
    transaction::{Transaction, TxnIn, TxnOut},
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
    mempool: Vec<Transaction>,
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
        );
        repo::save_block(db, block.hash.clone(), &block);
        self.newest_hash = block.hash;
        self.height = block.height;
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
    pub fn txn_outs_by_address(&self, db: &mut PickleDb, address: &str) -> Vec<TxnOut> {
        self.all_txn_outs(db)
            .into_iter()
            .filter(|txn_out| txn_out.owner == address.to_string())
            .collect()
    }

    pub fn balance_by_address(&self, db: &mut PickleDb, address: &str) -> u64 {
        self.txn_outs_by_address(db, address)
            .iter()
            .map(|txn| txn.amount)
            .sum()
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
            let old_txn_outs = self.txn_outs_by_address(db, from);
            let mut txn_ins: Vec<TxnIn> = vec![];
            let mut txn_outs: Vec<TxnOut> = vec![];
            let mut total = 0;
            for each in old_txn_outs.into_iter() {
                if total >= amount {
                    break;
                }
                txn_ins.push(TxnIn::new(from, each.amount));
                txn_outs.push(TxnOut::new(to, each.amount));
                total += each.amount;
            }
            if total > amount {
                txn_outs.push(TxnOut::new(from, total - amount));
            }
            let transaction = Transaction::new(txn_ins, txn_outs);
            self.mempool.push(transaction);
            Ok(())
        }
    }

    pub fn mempool(&self) -> Vec<Transaction> {
        self.mempool.clone()
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
        let (_r, mut db) = test_utils::test_db();

        // When
        let mut chain = BlockChain::get(&mut db);
        chain.add_block(&mut db);

        // Then
        let expected = Block::mine(String::from(""), 1, 1);
        let actual = chain.get_block(&mut db, expected.hash.clone()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_blocks() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        chain.add_block(&mut db);
        chain.add_block(&mut db);

        // Then
        let actual = chain.all_blocks(&mut db);
        assert_eq!(actual.len(), 2);
    }
}
