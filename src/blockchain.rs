use serde::{Deserialize, Serialize};

use crate::{block::Block, repo};

use nut::DB;

#[derive(Serialize, Deserialize)]
pub struct BlockChain {
    pub newest_hash: String,
    pub height: usize,
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
        let block = Block::new(data, self.newest_hash.clone(), self.height + 1);
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
            String::from("03675ac53ff9cd1535ccc7dfcdfa2c458c5218371f418dc136f2d19ac1fbe8a5")
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
        let expected = Block::new(String::from("Hello, World"), String::from(""), 1);
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
                    "03675ac53ff9cd1535ccc7dfcdfa2c458c5218371f418dc136f2d19ac1fbe8a5",
                ),
                hash: String::from(
                    "be18266b56aabea65bf6cc3cc23d39996dd84f2893ee4ba4bb8abd24280d23ac",
                ),
                height: 2,
            },
            Block {
                data: String::from("Hello, World"),
                prev_hash: String::from(""),
                hash: String::from(
                    "03675ac53ff9cd1535ccc7dfcdfa2c458c5218371f418dc136f2d19ac1fbe8a5",
                ),
                height: 1,
            },
        ];
        assert_eq!(actual, expected);
    }
}
