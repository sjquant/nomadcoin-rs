use sha2::{Digest, Sha256};

pub struct BlockChain {
    blocks: Vec<Block>,
}

impl BlockChain {
    pub fn new() -> Self {
        BlockChain { blocks: Vec::new() }
    }

    pub fn add_block(&mut self, data: &str) {
        let prev_hash = self.get_prev_hash();
        let hash = Sha256::digest(data.as_bytes());
        self.blocks.push(Block {
            data: data.to_string(),
            prev_hash: prev_hash.to_string(),
            hash: format!("{:x}", hash),
        });
    }

    fn get_prev_hash(&self) -> String {
        match self.blocks.last() {
            None => "".to_string(),
            Some(block) => block.hash.clone(),
        }
    }

    pub fn print_blocks(&self) {
        for b in &self.blocks {
            println!("Data: {}", b.data);
            println!("Hash: {}", b.hash);
            println!("Prev hash: {}\n", b.prev_hash)
        }
    }
}

#[derive(PartialEq, Debug)]
struct Block {
    data: String,
    hash: String,
    prev_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_block() {
        let mut chain = BlockChain::new();
        assert_eq!(chain.blocks.len(), 0);

        chain.add_block("Hello, World");
        let block1 = Block {
            data: "Hello, World".to_string(),
            hash: "03675ac53ff9cd1535ccc7dfcdfa2c458c5218371f418dc136f2d19ac1fbe8a5".to_string(),
            prev_hash: "".to_string(),
        };
        assert_eq!(chain.blocks[0], block1);

        chain.add_block("Hello, Korea");
        let block2 = Block {
            data: "Hello, Korea".to_string(),
            hash: "be18266b56aabea65bf6cc3cc23d39996dd84f2893ee4ba4bb8abd24280d23ac".to_string(),
            prev_hash: block1.hash,
        };
        assert_eq!(chain.blocks[1], block2);
    }
}
