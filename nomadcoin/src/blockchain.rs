use serde::{Deserialize, Serialize};

use crate::{block::Block, db};

#[derive(Serialize, Deserialize)]
pub struct BlockChain {
    pub newest_hash: String,
    pub height: usize,
}

impl BlockChain {
    pub fn get() -> Self {
        match db::checkpoint() {
            Some(hash) => bincode::deserialize(&hash).unwrap(),
            None => {
                let blockchain = BlockChain {
                    newest_hash: String::from(""),
                    height: 0,
                };
                blockchain.checkpoint();
                blockchain
            }
        }
    }

    pub fn add_block(&mut self, data: &str) {
        let block = Block::new(data.to_string(), self.newest_hash.clone(), self.height + 1);
        let data = bincode::serialize(&block).unwrap();
        db::save_block(block.hash.as_bytes(), data);
        self.newest_hash = block.hash;
        self.height = block.height;
        self.checkpoint();
    }

    fn checkpoint(&self) {
        db::save_blockchain(bincode::serialize(&self).unwrap());
    }

    pub fn all_blocks(&self) -> Vec<Block> {
        todo!()
    }

    pub fn get_block(&self, hash: usize) -> Option<Block> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_blockchain() {
        let chain = BlockChain::get();
        assert_eq!(chain.newest_hash, "");
        assert_eq!(chain.height, 0);
    }

    // #[test]
    // fn add_block() {
    //     let mut chain = BlockChain::get();

    //     chain.add_block("Hello, World");
    //     let block1 = Block::new(String::from("Hello, World"), String::from(""), 1);
    //     assert_eq!(chain.blocks[0], block1);

    //     chain.add_block("Hello, Korea");
    //     let block2 = Block::new(String::from("Hello, Korea"), block1.hash.to_string(), 2);
    //     assert_eq!(chain.blocks[1], block2);
    // }

    // #[test]
    // fn all_blocks() {
    //     let mut chain = BlockChain::get();
    //     chain.add_block("Hello, World");
    //     chain.add_block("Hello, Korea");

    //     let blocks = chain.all_blocks();
    //     assert_eq!(blocks, chain.blocks)
    // }

    // #[test]
    // fn get_block() {
    //     let mut chain = BlockChain::get();
    //     chain.add_block("Hello, World");

    //     let block = chain.get_block(1).unwrap();
    //     assert_eq!(
    //         block,
    //         Block::new(String::from("Hello, World"), String::from(""), 1)
    //     );

    //     let block_not_found = chain.get_block(2);
    //     assert_eq!(None, block_not_found);
    // }
}
