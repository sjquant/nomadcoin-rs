use sha2::{Digest, Sha256};

pub trait Hashable {
    fn hash(&self) -> String {
        format!("{:x}", Sha256::digest(&self.bytes()))
    }
    fn bytes(&self) -> Vec<u8>;
}
