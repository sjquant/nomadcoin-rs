use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{collections::HashMap, io::Error, iter, sync::Mutex};

use crate::Wallet;
use crate::{repo::BaseRepository, Block, BlockChainSnapshot};

pub fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect::<String>()
}

pub struct TestRepository {
    snapshot: Mutex<Option<BlockChainSnapshot>>,
    blocks: Mutex<HashMap<String, Block>>,
}

impl TestRepository {
    pub fn new() -> Self {
        Self {
            snapshot: Mutex::new(None),
            blocks: Mutex::new(HashMap::new()),
        }
    }
}

impl BaseRepository for TestRepository {
    fn load_snapshot(&self) -> Option<BlockChainSnapshot> {
        self.snapshot.lock().unwrap().clone()
    }
    fn get_block(&self, hash: String) -> Option<Block> {
        self.blocks.lock().unwrap().get(&hash).cloned()
    }
    fn save_snapshot(&self, snapshot: &BlockChainSnapshot) -> Result<(), Error> {
        *self.snapshot.lock().unwrap() = Some(snapshot.clone());
        Ok(())
    }
    fn save_block(&self, block: &Block) -> Result<(), Error> {
        self.blocks
            .lock()
            .unwrap()
            .insert(block.hash.clone(), block.clone());
        Ok(())
    }
    fn remove_all_blocks(&self) -> Result<(), Error> {
        self.blocks.lock().unwrap().clear();
        Ok(())
    }
}

pub fn test_pickle_db() -> Mutex<PickleDb> {
    let temp_path = std::env::temp_dir().join(format!("{}.db", random_string(32)));
    let db = Mutex::new(PickleDb::new(
        temp_path,
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Bin,
    ));
    db
}

pub fn test_wallet() -> Wallet {
    let temp_path = std::env::temp_dir().join(format!("{}.wallet", random_string(32)));
    let wallet = Wallet::get(temp_path.to_str().unwrap());
    wallet
}
