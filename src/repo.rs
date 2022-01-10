use pickledb::PickleDb;

use crate::{Block, BlockChain};

pub fn save_block(db: &mut PickleDb, hash: String, block: &Block) {
    db.set(format!("block:{}", hash).as_str(), block).unwrap();
}

pub fn get_block(db: &mut PickleDb, hash: String) -> Option<Block> {
    db.get::<Block>(format!("block:{}", hash).as_str())
}

pub fn checkpoint(db: &mut PickleDb) -> Option<BlockChain> {
    db.get::<BlockChain>("checkpoint")
}

pub fn save_blockchain(db: &mut PickleDb, blockchain: &BlockChain) {
    db.set("checkpoint", blockchain).unwrap();
}
