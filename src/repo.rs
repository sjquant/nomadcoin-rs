use std::{io::Error, sync::Mutex};

use crate::{Block, BlockChainSnapshot};
use pickledb::PickleDb;

pub trait BaseRepository: Send + Sync {
    fn load_snapshot(&self) -> Option<BlockChainSnapshot>;
    fn get_block(&self, hash: String) -> Option<Block>;
    fn save_snapshot(&self, snapshot: &BlockChainSnapshot) -> Result<(), Error>;
    fn save_block(&self, block: &Block) -> Result<(), Error>;
    fn remove_all_blocks(&self) -> Result<(), Error>;
}

pub struct PickleDBRepository {
    conn: Mutex<PickleDb>,
}

impl PickleDBRepository {
    pub fn new(conn: Mutex<PickleDb>) -> Self {
        Self { conn }
    }
}

impl BaseRepository for PickleDBRepository {
    fn load_snapshot(&self) -> Option<BlockChainSnapshot> {
        let conn = self.conn.lock().unwrap();
        conn.get::<BlockChainSnapshot>("snapshot")
    }
    fn get_block(&self, hash: String) -> Option<Block> {
        let conn = self.conn.lock().unwrap();
        conn.get::<Block>(format!("block:{}", hash).as_str())
    }
    fn save_snapshot(&self, snapshot: &BlockChainSnapshot) -> Result<(), Error> {
        let mut conn = self.conn.lock().unwrap();
        let _ = conn.set("snapshot", snapshot);
        Ok(())
    }
    fn save_block(&self, block: &Block) -> Result<(), Error> {
        let mut conn = self.conn.lock().unwrap();
        let _ = conn.set(format!("block:{}", block.hash).as_str(), block);
        Ok(())
    }
    fn remove_all_blocks(&self) -> Result<(), Error> {
        let mut conn = self.conn.lock().unwrap();
        for key in conn.get_all().into_iter() {
            let _ = conn.rem(key.as_str());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::testutils;

    use super::*;

    #[test]
    fn test_pickle_repo_saving_snapshot_works_correctly() {
        // Given
        let (_, conn) = testutils::test_pickle_db();
        let repo = PickleDBRepository::new(conn);
        let mut snapshot = BlockChainSnapshot::new();
        snapshot.newest_hash = "newest_hash".to_string();
        snapshot.height = 2;
        snapshot.difficulty = 1;

        // When
        repo.save_snapshot(&snapshot).unwrap();

        // Then
        let actual = repo.load_snapshot().unwrap();
        assert_eq!(actual.newest_hash, "newest_hash");
        assert_eq!(actual.height, 2);
        assert_eq!(actual.difficulty, 1);
    }

    #[test]
    fn test_pickle_repo_save_block_works_properly() {
        // Given
        let (_, conn) = testutils::test_pickle_db();
        let repo = PickleDBRepository::new(conn);
        let block = Block::mine("address", "prev_hash", 1, 1, &mut vec![]);

        // When
        repo.save_block(&block).unwrap();

        // Then
        let actual = repo.get_block(block.hash.clone()).unwrap();
        assert_eq!(actual.hash, block.hash);
    }

    #[test]
    fn test_pickle_repo_remove_all_blocks_works_properly() {
        // Given
        let (_, conn) = testutils::test_pickle_db();
        let repo = PickleDBRepository::new(conn);
        let block = Block::mine("address", "prev_hash", 1, 1, &mut vec![]);
        repo.save_block(&block).unwrap();

        // When
        repo.remove_all_blocks().unwrap();

        // Then
        let actual = repo.get_block(block.hash.clone());
        assert!(actual.is_none());
    }
}
