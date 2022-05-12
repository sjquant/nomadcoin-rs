use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::path::Path;
use std::{collections::HashMap, io::Error, iter, sync::Mutex};
use std::{env, fs};

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

pub struct DBResource {
    path: String,
}

impl Drop for DBResource {
    fn drop(&mut self) {
        let path = Path::new(&self.path);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }
}

pub fn test_pickle_db() -> (DBResource, Mutex<PickleDb>) {
    let temp_path = env::temp_dir().join(format!("{}.db", random_string(32)));
    let path_string = temp_path.clone().into_os_string().into_string().unwrap();
    let db = Mutex::new(PickleDb::new(
        temp_path,
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Bin,
    ));
    let db_resource = DBResource { path: path_string };

    // Order is important.
    // db first dropped, and then db_resource dropped
    (db_resource, db)
}
