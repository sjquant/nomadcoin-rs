use serde::{Deserialize, Serialize};

use crate::{block::Block, repo};
use pickledb::PickleDb;

const DIFFICULTY_INTERVAL: u64 = 5;
const TIME_THRESHOLD: i64 = 36000;
const ALLOWED_BUFFER: i64 = 7200;

#[derive(Serialize, Deserialize)]
pub struct BlockChain {
    pub newest_hash: String,
    pub height: u64,
    pub difficulty: u16,
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
                };
                blockchain.create_checkpoint(db);
                blockchain
            }
        }
    }

    pub fn add_block(&mut self, db: &mut PickleDb, data: String) {
        let block = Block::mine(
            data,
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
            chain.add_block(&mut db, String::from("Hello, Korea"));
            chain.add_block(&mut db, String::from("Hello, World"));
        }

        // When
        let chain = BlockChain::get(&mut db);

        // Then
        assert_eq!(
            chain.newest_hash,
            String::from("0bcf2215a416a39b22b37a159168147f039c2c53028d9a193c9c9fe92dc54043")
        );
        assert_eq!(chain.height, 2);
    }

    #[test]
    fn add_block() {
        let (_r, mut db) = test_utils::test_db();

        // When
        let mut chain = BlockChain::get(&mut db);
        chain.add_block(&mut db, String::from("Hello, World"));

        // Then
        let expected = Block::mine(String::from("Hello, World"), String::from(""), 1, 1);
        let actual = chain.get_block(&mut db, expected.hash.clone()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_blocks() {
        // Given
        let (_r, mut db) = test_utils::test_db();
        let mut chain = BlockChain::get(&mut db);
        chain.add_block(&mut db, String::from("Hello, World"));
        chain.add_block(&mut db, String::from("Hello, Korea"));

        // Then
        let actual = chain.all_blocks(&mut db);
        let expected = vec![
            Block {
                data: "Hello, Korea".to_string(),
                prev_hash: String::from(
                    "093da9f4424f0ba24e620a723061a5f1350ae9d67347820d4a7e0852ea7a1d3c",
                ),
                hash: String::from(
                    "06ae2f7cfaa5faa17c7bbd9897cf1d15da5a63cdaf24fcbc62e40421e5becb6a",
                ),
                height: 2,
                difficulty: 1,
                nonce: 32,
                timestamp: actual[0].timestamp,
            },
            Block {
                data: String::from("Hello, World"),
                prev_hash: String::from(""),
                hash: String::from(
                    "093da9f4424f0ba24e620a723061a5f1350ae9d67347820d4a7e0852ea7a1d3c",
                ),
                height: 1,
                difficulty: 1,
                nonce: 13,
                timestamp: actual[1].timestamp,
            },
        ];
        assert_eq!(actual, expected);
    }
}
