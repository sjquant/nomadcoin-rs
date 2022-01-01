use nut::DB;

pub fn save_block(db: &mut DB, hash: &[u8], data: Vec<u8>) {
    let mut tx = db.begin_rw_tx().unwrap();

    {
        let mut bucket = tx.create_bucket_if_not_exists(b"data").unwrap();
        bucket.put(hash, data).unwrap();
    }
}

pub fn get_block(db: &mut DB, hash: &[u8]) -> Option<Vec<u8>> {
    let mut tx = db.begin_rw_tx().unwrap();
    let bucket = tx.create_bucket_if_not_exists(b"data").unwrap();
    bucket.get(hash).map(|data| data.to_vec())
}

pub fn checkpoint(db: &mut DB) -> Option<Vec<u8>> {
    let mut tx = db.begin_rw_tx().unwrap();
    let bucket = tx.create_bucket_if_not_exists(b"data").unwrap();
    bucket.get(b"checkpoint").map(|data| data.to_vec())
}

pub fn save_blockchain(db: &mut DB, data: Vec<u8>) {
    let mut tx = db.begin_rw_tx().unwrap();

    {
        let mut bucket = tx.create_bucket_if_not_exists(b"data").unwrap();
        bucket.put(b"checkpoint", data).unwrap();
    }
}
