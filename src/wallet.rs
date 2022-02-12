use p256::ecdsa::{signature::Signer, SigningKey};
use rand_core::OsRng;
use std::{
    io::{Read, Write},
    path::Path,
};

pub struct Wallet {
    pub private_key: SigningKey,
    pub address: String,
}

fn read_key_from_file(filename: &str) -> SigningKey {
    let mut file = std::fs::File::open(filename).unwrap();
    let mut key_as_bytes = Vec::new();
    file.read_to_end(&mut key_as_bytes).unwrap();
    SigningKey::from_bytes(&key_as_bytes).unwrap()
}

fn store_key_to_file(filename: &str, key: &SigningKey) -> std::io::Result<()> {
    let mut file = std::fs::File::create(filename)?;
    file.write_all(&key.to_bytes()).unwrap();
    Ok(())
}

impl Wallet {
    pub fn get(filename: &str) -> Self {
        let private_key: SigningKey;
        if Path::new(filename).exists() {
            private_key = read_key_from_file(filename)
        } else {
            private_key = SigningKey::random(&mut OsRng);
            store_key_to_file(filename, &private_key).unwrap();
        }
        let address = private_key
            .verifying_key()
            .to_encoded_point(false)
            .to_string();
        Wallet {
            private_key,
            address,
        }
    }

    pub fn sign(&self, payload: &[u8]) -> String {
        self.private_key.sign(payload).to_string()
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::signature::{Signature, Verifier};

    use super::*;
    use crate::test_utils;

    #[test]
    fn get_new_wallet_creates_file() {
        let filename = test_utils::random_string(16);
        // When
        let wallet = Wallet::get(&filename);

        // Then
        assert!(Path::new(&filename).exists());
        let key = read_key_from_file(&filename);
        assert_eq!(wallet.private_key, key);

        // Teardown
        std::fs::remove_file(&filename).unwrap();
    }

    #[test]
    fn get_existing_wallet_when_file_exists() {
        let filename = test_utils::random_string(16);
        // Given
        let a_key = SigningKey::random(&mut OsRng);
        store_key_to_file(&filename, &a_key).unwrap();

        // When
        let wallet = Wallet::get(&filename);

        // Then
        assert_eq!(wallet.private_key, a_key);

        // Teardown
        std::fs::remove_file(&filename).unwrap();
    }

    #[test]
    fn sign_data() {
        let filename = test_utils::random_string(16);
        // Given
        let wallet = Wallet::get(&filename);
        let hashed_string = hex::encode("hello, world");
        let data = &hex::decode(hashed_string).unwrap();
        // When
        let signature_as_string = wallet.sign(data);

        // Then
        let signature_as_bytes = &hex::decode(signature_as_string).unwrap();
        let signature = Signature::from_bytes(&signature_as_bytes).unwrap();
        assert!(wallet
            .private_key
            .verifying_key()
            .verify(data, &signature)
            .is_ok());

        // Teardown
        std::fs::remove_file(&filename).unwrap();
    }
}
