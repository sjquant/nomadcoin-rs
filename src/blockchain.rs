use p256::ecdsa::{
    signature::{Signature, Verifier},
    VerifyingKey,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    block::Block,
    error::Error,
    repo::BaseRepository,
    transaction::{Transaction, TxnIn, TxnOut, UTxnOut},
    Wallet,
};

const DIFFICULTY_INTERVAL: u64 = 5;
const TIME_THRESHOLD: i64 = 36000;
const ALLOWED_BUFFER: i64 = 7200;

macro_rules! unwrap_or_return_false {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return false,
        }
    };
}

fn verify_msg(public_key_str: &str, msg: &str, signature_str: &str) -> bool {
    let public_key_as_bytes = unwrap_or_return_false!(hex::decode(public_key_str));
    let public_key = unwrap_or_return_false!(VerifyingKey::from_sec1_bytes(&public_key_as_bytes));
    let signature_as_bytes = unwrap_or_return_false!(hex::decode(signature_str));
    let signature = unwrap_or_return_false!(Signature::from_bytes(&signature_as_bytes));
    let msg_as_bytes = unwrap_or_return_false!(hex::decode(msg));
    public_key.verify(&msg_as_bytes, &signature).is_ok()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockChainSnapshot {
    pub newest_hash: String,
    pub height: u64,
    pub difficulty: u16,
    pub mempool: Vec<Transaction>,
}

impl BlockChainSnapshot {
    pub fn new() -> Self {
        Self {
            newest_hash: String::from(""),
            height: 0,
            difficulty: 1,
            mempool: vec![],
        }
    }
}

pub struct BlockChain {
    repo: Box<dyn BaseRepository>,
    snapshot: BlockChainSnapshot,
}

impl BlockChain {
    pub fn load(repo: Box<dyn BaseRepository>) -> Self {
        match repo.load_snapshot() {
            Some(snapshot) => Self { repo, snapshot },
            None => {
                let snapshot = BlockChainSnapshot::new();
                repo.save_snapshot(&snapshot).unwrap();
                let blockchain = BlockChain { repo, snapshot };
                blockchain
            }
        }
    }

    pub fn mine_block(&mut self, address: &str) -> Block {
        let difficulty = self.calc_difficulty();
        let block = Block::mine(
            address,
            self.snapshot.newest_hash.as_str(),
            self.snapshot.height + 1,
            difficulty,
            &mut self.snapshot.mempool,
        );
        self.update_snapshot(&block);
        self.clear_mempool();
        self.repo.save_block(&block).unwrap();
        self.repo.save_snapshot(&self.snapshot).unwrap();
        block
    }

    fn clear_mempool(&mut self) {
        self.snapshot.mempool.clear();
    }

    fn update_snapshot(&mut self, block: &Block) {
        self.snapshot.newest_hash = block.hash.clone();
        self.snapshot.height = block.height;
        self.snapshot.difficulty = block.difficulty;
    }

    pub fn add_block(&mut self, block: Block) {
        self.update_snapshot(&block);
        self.repo.save_block(&block).unwrap();
        self.repo.save_snapshot(&self.snapshot).unwrap();
    }

    pub fn all_blocks(&self) -> Vec<Block> {
        let mut hash_cursor = self.snapshot.newest_hash.clone();
        let mut blocks: Vec<Block> = Vec::new();

        while hash_cursor.as_str() != "" {
            let block = self.repo.get_block(hash_cursor).unwrap();
            blocks.push(block.clone());
            hash_cursor = block.prev_hash;
        }
        blocks
    }

    pub fn newest_block(&self) -> Option<Block> {
        self.repo.get_block(self.snapshot.newest_hash.clone())
    }
    pub fn get_block(&self, hash: String) -> Option<Block> {
        self.repo.get_block(hash)
    }

    fn calc_difficulty(&mut self) -> u16 {
        if self.snapshot.height != 0 && self.snapshot.height % DIFFICULTY_INTERVAL == 0 {
            let all_blocks = self.all_blocks();
            let newest_timestamp = all_blocks[0].timestamp;
            let base_timestamp = all_blocks[(DIFFICULTY_INTERVAL - 1) as usize].timestamp;
            let time_taken = newest_timestamp - base_timestamp;
            if time_taken < TIME_THRESHOLD - ALLOWED_BUFFER {
                return self.snapshot.difficulty + 1;
            } else if time_taken > TIME_THRESHOLD + ALLOWED_BUFFER {
                return self.snapshot.difficulty - 1;
            }
        }
        return self.snapshot.difficulty;
    }

    pub fn all_txn_outs(&self) -> Vec<TxnOut> {
        let blocks = self.all_blocks();
        let mut txn_outs: Vec<TxnOut> = vec![];

        for block in blocks.into_iter() {
            for mut txn in block.transactions.into_iter() {
                txn_outs.append(&mut txn.txn_outs)
            }
        }
        txn_outs
    }

    pub fn balance_by_address(&self, address: &str) -> u64 {
        self.unspent_txnouts_by_address(address)
            .iter()
            .map(|txn| txn.amount)
            .sum()
    }

    fn is_on_mempool(&self, utxnout: &UTxnOut) -> bool {
        self.snapshot.mempool.iter().any(|txn| {
            txn.txn_ins
                .iter()
                .any(|txn_in| txn_in.txn_hash == utxnout.txn_hash && txn_in.idx == utxnout.idx)
        })
    }

    pub fn unspent_txnouts_by_address(&self, address: &str) -> Vec<UTxnOut> {
        let mut utxnouts = vec![];
        let mut existing_txn_hashes: HashSet<&str> = HashSet::new();
        for block in self.all_blocks().iter() {
            for txn in block.transactions.iter() {
                for txn_in in txn.txn_ins.iter() {
                    if txn_in.signature.as_str() == "COINBASE" {
                        break;
                    }
                    if txn.txn_outs[txn_in.idx as usize].address == address {
                        existing_txn_hashes.insert(txn_in.txn_hash.as_str());
                    }
                }
                for (idx, txn_out) in txn.txn_outs.clone().into_iter().enumerate() {
                    if txn_out.address.as_str() == address
                        && !existing_txn_hashes.contains(txn.hash.as_str())
                    {
                        let utxnout =
                            UTxnOut::new(&txn.hash, idx.try_into().unwrap(), txn_out.amount);
                        if !self.is_on_mempool(&utxnout) {
                            utxnouts.push(utxnout);
                        }
                    }
                }
            }
        }
        utxnouts
    }

    fn get_transaction(&self, id: &str) -> Option<Transaction> {
        let blocks = self.all_blocks();
        for block in blocks.iter() {
            for txn in block.transactions.iter() {
                if txn.hash == id {
                    return Some(txn.clone());
                }
            }
        }
        None
    }

    fn validate_transaction(&self, txn: &Transaction) -> bool {
        for txn_in in txn.txn_ins.iter() {
            match self.get_transaction(txn_in.txn_hash.as_str()) {
                Some(prev_txn) => {
                    let address = prev_txn.txn_outs[txn_in.idx as usize].address.as_str();
                    let signature = txn_in.signature.as_str();
                    let msg = hex::encode(&txn.hash);
                    if !verify_msg(address, msg.as_str(), signature) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    pub fn make_transaction(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
        wallet: &Wallet,
    ) -> Result<Transaction, Error> {
        if self.balance_by_address(from) < amount {
            Err(Error::new("Not enough balance"))
        } else {
            let utxn_outs = self.unspent_txnouts_by_address(from);
            let mut txn_ins: Vec<TxnIn> = vec![];
            let mut txn_outs: Vec<TxnOut> = vec![];
            let mut total = 0;
            for utxnout in utxn_outs.into_iter() {
                if total >= amount {
                    break;
                }
                txn_ins.push(TxnIn::new(&utxnout.txn_hash, utxnout.idx, utxnout.amount));
                total += utxnout.amount;
            }
            // Bring changes back to transaction sender
            if total > amount {
                txn_outs.push(TxnOut::new(from, total - amount));
            }
            txn_outs.push(TxnOut::new(to, amount));
            let mut transaction = Transaction::new(txn_ins, txn_outs);
            transaction.sign(wallet);
            if !self.validate_transaction(&transaction) {
                return Err(Error::new("Invalid transaction"));
            };
            self.add_txn_to_mempool(transaction.clone());
            Ok(transaction)
        }
    }

    pub fn add_txn_to_mempool(&mut self, txn: Transaction) {
        self.snapshot.mempool.push(txn);
    }

    pub fn replace(&mut self, new_blocks: Vec<Block>) {
        self.update_snapshot(&new_blocks[0]);
        self.repo.save_snapshot(&self.snapshot).unwrap();
        let _ = self.repo.remove_all_blocks();
        for block in new_blocks.iter() {
            self.repo.save_block(block).unwrap();
        }
    }

    pub fn mempool(&self) -> Vec<Transaction> {
        self.snapshot.mempool.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::repo::TestRepository;

    use super::*;

    #[test]
    fn load_new_blockchain_when_repository_is_empty() {
        let test_repo = Box::new(TestRepository::new());
        let chain = BlockChain::load(test_repo);
        assert_eq!(chain.snapshot.newest_hash, "");
        assert_eq!(chain.snapshot.height, 0);
    }

    #[test]
    fn load_old_blockchain_when_repository_is_not_empty() {
        // Given
        let test_repo = TestRepository::new();
        let mut test_snapshot = BlockChainSnapshot::new();
        let block1 = Block::mine("some_address", "", 1, 1, &mut vec![]);
        let block2 = Block::mine("some_address", block1.hash.as_str(), 2, 1, &mut vec![]);
        test_snapshot.height = 2;
        test_snapshot.newest_hash = block2.hash.clone();

        test_repo.save_snapshot(&test_snapshot).unwrap();
        test_repo.save_block(&block1).unwrap();
        test_repo.save_block(&block2).unwrap();

        // When
        let chain = BlockChain::load(Box::new(test_repo));

        // Then
        assert_eq!(chain.snapshot.height, 2);
        assert_eq!(chain.all_blocks(), vec![block2, block1]);
    }

    #[test]
    fn mining_block_records_on_blockchain() {
        // Given
        let test_repo = Box::new(TestRepository::new());

        // When
        let mut chain = BlockChain::load(test_repo);
        let block1 = chain.mine_block("some-address");

        // Then
        assert_eq!(chain.snapshot.height, 1);
        assert_eq!(chain.snapshot.newest_hash, block1.hash);

        // When
        let block2 = chain.mine_block("some-address");

        // Then
        assert_eq!(chain.snapshot.height, 2);
        assert_eq!(chain.snapshot.newest_hash, block2.hash);

        let blocks = chain.all_blocks();
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn mining_block_should_increase_miner_balance_of_address() {
        // Given
        let test_repo = Box::new(TestRepository::new());

        // When
        let mut chain = BlockChain::load(test_repo);
        let block = chain.mine_block("some-address");

        // Then
        let balance = chain.balance_by_address("some-address");
        assert_eq!(balance, block.transactions[0].txn_outs[0].amount);
    }

    #[test]
    fn making_transaction_changes_balance_and_mempool() {
        // Given
        let test_repo = Box::new(TestRepository::new());
        let mut chain = BlockChain::load(test_repo);
        let wallet = Wallet::get("nico.wallet");
        let address = wallet.address.as_str();
        chain.mine_block(address); // Earn 50 by mining block
        chain.mine_block(address); // Earn 50 by mining block
        chain.mine_block(address); // Earn 50 by mining bloc

        // When
        chain
            .make_transaction(address, "to-address", 20, &wallet)
            .unwrap();
        chain
            .make_transaction(address, "to-address", 10, &wallet)
            .unwrap();

        // Then
        let mempool = chain.mempool();
        let mem_txn = mempool[0].clone();
        assert_eq!(mempool.len(), 2);
        assert_eq!(chain.balance_by_address(address), 50); // 100 were spent for making transaction
        assert_eq!(mem_txn.txn_ins[0].amount, 50);
        assert_eq!(mem_txn.txn_outs[0].address, String::from(address));
        assert_eq!(mem_txn.txn_outs[0].amount, 30);
        assert_eq!(mem_txn.txn_outs[1].address, String::from("to-address"));
        assert_eq!(mem_txn.txn_outs[1].amount, 20);
    }

    #[test]
    fn mining_block_confirms_transaction() {
        // Given
        let test_repo = Box::new(TestRepository::new());
        let mut chain = BlockChain::load(test_repo);
        let wallet = Wallet::get("nico.wallet");
        let address = wallet.address.as_str();
        chain.mine_block(address); // Earn 50 by mining block
        chain
            .make_transaction(address, "to-address", 20, &wallet)
            .unwrap();

        // When
        chain.mine_block(address); // Earn another 50 by mining block

        // Then
        assert_eq!(chain.balance_by_address(address), 80);
        assert_eq!(chain.balance_by_address("to-address"), 20);
        assert_eq!(chain.mempool().len(), 0);
    }

    #[test]
    fn cannot_make_transaction_when_verification_failed() {
        // Given
        let test_repo = Box::new(TestRepository::new());
        let mut chain = BlockChain::load(test_repo);
        let wallet = Wallet::get("nico.wallet");
        let wrong_address = "04C72F87E9176F814714F5EF9DE2414863937D1391B02EF8BA576C89A2F69130E6032A56D01750F2638146BC898FA59695813462A49BA24B85003304DFF2BF76D4";
        chain.mine_block(wrong_address); // Earn 50 by mining block

        // When
        let err = chain
            .make_transaction(wrong_address, "to-address", 20, &wallet)
            .unwrap_err();

        // Then
        assert_eq!(err.msg, String::from("Invalid transaction"));
    }

    #[test]
    fn cannot_make_transaction_when_balance_is_not_enough() {
        // Given
        let test_repo = Box::new(TestRepository::new());
        let mut chain = BlockChain::load(test_repo);
        let wallet = Wallet::get("nico.wallet");
        let address = wallet.address.as_str();
        chain.mine_block(address); // Earn 50 by mining block

        // When
        let err = chain
            .make_transaction(address, "to-address", 60, &wallet)
            .unwrap_err();

        // Then
        assert_eq!(err.msg, String::from("Not enough balance"));
    }
}
