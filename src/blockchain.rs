use serde::{Deserialize, Serialize};

use crate::{block::Block, repo};

use nut::DB;

#[derive(Serialize, Deserialize)]
pub struct BlockChain {
    pub newest_hash: String,
    pub height: u64,
}

impl BlockChain {
    pub fn get(db: &mut DB) -> Self {
        match repo::checkpoint(db) {
            Some(data) => bincode::deserialize(&data).unwrap(),
            None => {
                let blockchain = BlockChain {
                    newest_hash: String::from(""),
                    height: 0,
                };
                blockchain.create_checkpoint(db);
                blockchain
            }
        }
    }

    pub fn add_block(&mut self, db: &mut DB, data: String) {
        let block = Block::mine(data, self.newest_hash.clone(), self.height + 1, 1);
        let data = bincode::serialize(&block).unwrap();
        repo::save_block(db, block.hash.as_bytes(), data);
        self.newest_hash = block.hash;
        self.height = block.height;
        self.create_checkpoint(db);
    }

    fn create_checkpoint(&self, db: &mut DB) {
        repo::save_blockchain(db, bincode::serialize(&self).unwrap());
    }

    pub fn all_blocks(&self, db: &mut DB) -> Vec<Block> {
        let mut hash_cursor = self.newest_hash.clone();
        let mut blocks: Vec<Block> = Vec::new();

        while hash_cursor.as_str() != "" {
            let block = self.get_block(db, hash_cursor).unwrap();
            blocks.push(block.clone());
            hash_cursor = block.prev_hash;
        }
        blocks
    }

    pub fn get_block(&self, db: &mut DB, hash: String) -> Option<Block> {
        repo::get_block(db, hash.as_bytes()).map(|data| bincode::deserialize(&data).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;

    #[test]
    fn get_new_blockchain() {
        let mut db = test_utils::test_db();
        let chain = BlockChain::get(&mut db);
        assert_eq!(chain.newest_hash, "");
        assert_eq!(chain.height, 0);
    }

    #[test]
    fn get_blockchain_from_db() {
        // Given
        let mut db = test_utils::test_db();

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
        let mut db = test_utils::test_db();

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
        let mut db = test_utils::test_db();
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
            },
        ];
        assert_eq!(actual, expected);
    }
}
