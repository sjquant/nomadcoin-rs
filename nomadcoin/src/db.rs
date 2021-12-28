use nut::DBBuilder;

pub fn save_block(hash: &[u8], data: Vec<u8>) {
    let mut db = DBBuilder::new("blockchain.db").build().unwrap();
    let mut tx = db.begin_rw_tx().unwrap();

    {
        let mut bucket = tx.create_bucket_if_not_exists(b"blocks").unwrap();
        bucket.put(hash, data).unwrap();
    }
}

pub fn save_blockchain(data: Vec<u8>) {
    let mut db = DBBuilder::new("blockchain.db").build().unwrap();
    let mut tx = db.begin_rw_tx().unwrap();

    {
        let mut bucket = tx.create_bucket_if_not_exists(b"data").unwrap();
        bucket.put(b"checkpoint", data).unwrap();
    }
}
