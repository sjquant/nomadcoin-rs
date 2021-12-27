use crate::block::Block;

pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl BlockChain {
    pub fn new() -> Self {
        BlockChain { blocks: Vec::new() }
    }

    pub fn add_block(&mut self, data: &str) {
        let prev_hash = self.get_prev_hash();
        self.blocks.push(Block::new(
            data.to_string(),
            prev_hash,
            self.blocks.len() + 1,
        ));
    }

    fn get_prev_hash(&self) -> String {
        match self.blocks.last() {
            None => "".to_string(),
            Some(block) => block.hash.clone(),
        }
    }

    pub fn all_blocks(&self) -> Vec<Block> {
        self.blocks.clone()
    }

    pub fn get_block(&self, height: usize) -> Option<Block> {
        let blocks = self.blocks.clone();
        match blocks.get(height - 1) {
            Some(block) => Some(block.clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_block() {
        let mut chain = BlockChain::new();

        chain.add_block("Hello, World");
        let block1 = Block::new(String::from("Hello, World"), String::from(""), 1);
        assert_eq!(chain.blocks[0], block1);

        chain.add_block("Hello, Korea");
        let block2 = Block::new(String::from("Hello, Korea"), block1.hash.to_string(), 2);
        assert_eq!(chain.blocks[1], block2);
    }

    #[test]
    fn all_blocks() {
        let mut chain = BlockChain::new();
        chain.add_block("Hello, World");
        chain.add_block("Hello, Korea");

        let blocks = chain.all_blocks();
        assert_eq!(blocks, chain.blocks)
    }

    #[test]
    fn get_block() {
        let mut chain = BlockChain::new();
        chain.add_block("Hello, World");

        let block = chain.get_block(1).unwrap();
        assert_eq!(
            block,
            Block::new(String::from("Hello, World"), String::from(""), 1)
        );

        let block_not_found = chain.get_block(2);
        assert_eq!(None, block_not_found);
    }
}
