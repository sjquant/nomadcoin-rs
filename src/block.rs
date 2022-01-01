use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone)]
pub struct Block {
    pub data: String,
    pub hash: String,
    pub prev_hash: String,
    pub height: usize,
}

impl Block {
    pub fn new(data: String, prev_hash: String, height: usize) -> Self {
        let hash = Sha256::digest(data.as_bytes());
        Block {
            data: data,
            prev_hash: prev_hash,
            hash: format!("{:x}", hash),
            height: height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_block() {
        let block = Block::new(String::from("Hello, World"), String::from("a-prev-hash"), 2);
        assert_eq!(
            block,
            Block {
                data: String::from("Hello, World"),
                prev_hash: String::from("a-prev-hash"),
                hash: String::from(
                    "03675ac53ff9cd1535ccc7dfcdfa2c458c5218371f418dc136f2d19ac1fbe8a5"
                ),
                height: 2
            }
        );
    }
}
