use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone)]
pub struct Block {
    pub data: String,
    pub hash: String,
    pub prev_hash: String,
    pub height: u64,
    pub difficulty: u16,
    pub nonce: u64,
}

impl Block {
    pub fn mine(data: String, prev_hash: String, height: u64, difficulty: u16) -> Self {
        let mut nonce: u64 = 0;
        let mut hash_data;
        let mut hash = String::from("");
        let target = std::iter::repeat("0")
            .take(difficulty.into())
            .collect::<String>();

        loop {
            if hash.starts_with(&target) {
                break;
            }
            hash_data = format!("{}{}{}{}{}", data, prev_hash, height, difficulty, nonce);
            hash = format!("{:x}", Sha256::digest(hash_data.as_bytes()));
            nonce += 1;
        }

        Block {
            data: data,
            prev_hash: prev_hash,
            hash: hash,
            height: height,
            difficulty: difficulty,
            nonce: nonce,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_block() {
        let block = Block::mine(
            String::from("Hello, World"),
            String::from("a-prev-hash"),
            10,
            2,
        );
        assert_eq!(
            block,
            Block {
                data: String::from("Hello, World"),
                prev_hash: String::from("a-prev-hash"),
                hash: String::from(
                    "00f3645cc2dd8b1d2bbfaf29333ac4d31433dca73280cfd2f4f55a91f790947b"
                ),
                height: 10,
                difficulty: 2,
                nonce: 428
            }
        );
    }
}
