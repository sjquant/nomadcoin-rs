use std::io::Error;

use crate::{Block, BlockChainSnapshot};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

pub trait BaseRepository: Send + Sync {
    fn load_snapshot(&self) -> Option<BlockChainSnapshot>;
    fn get_block(&self, hash: String) -> Option<Block>;
    fn save_snapshot(&self, snapshot: &BlockChainSnapshot) -> Result<(), Error>;
    fn save_block(&self, block: &Block) -> Result<(), Error>;
    fn remove_all_blocks(&self) -> Result<(), Error>;
}

pub struct PickleDBRepository {
    db_path: String,
}

impl PickleDBRepository {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }

    fn get_conn(&self) -> PickleDb {
        match PickleDb::load(
            self.db_path.as_str(),
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        ) {
            Ok(load) => load,
            Err(_) => PickleDb::new(
                self.db_path.as_str(),
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Json,
            ),
        }
    }
}

impl BaseRepository for PickleDBRepository {
    fn load_snapshot(&self) -> Option<BlockChainSnapshot> {
        let conn = self.get_conn();
        conn.get::<BlockChainSnapshot>("snapshot")
    }
    fn get_block(&self, hash: String) -> Option<Block> {
        let conn = self.get_conn();
        conn.get::<Block>(format!("block:{}", hash).as_str())
    }
    fn save_snapshot(&self, snapshot: &BlockChainSnapshot) -> Result<(), Error> {
        let mut conn = self.get_conn();
        let _ = conn.set("snapshot", snapshot);
        Ok(())
    }
    fn save_block(&self, block: &Block) -> Result<(), Error> {
        let mut conn = self.get_conn();
        let _ = conn.set(format!("block:{}", block.hash).as_str(), block);
        Ok(())
    }
    fn remove_all_blocks(&self) -> Result<(), Error> {
        let mut conn = self.get_conn();
        for key in conn.get_all().into_iter() {
            let _ = conn.rem(key.as_str());
        }
        Ok(())
    }
}
