use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use anyhow::Result;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct ServerCrypto {
    cipher: Aes256Gcm
}

impl ServerCrypto {
    pub fn new(key: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let derived_key = hasher.finalize();
        
        let cipher = Aes256Gcm::new_from_slice(&derived_key).expect("Invalid key size");
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let bytes: [u8;12] = rand::random();
        let nonce = Nonce::from_slice(&bytes);

        let ciphertext = self.cipher.encrypt(nonce, plaintext).map_err(|e| anyhow::anyhow!("Encryption faild, {:?}", e))?;

        let mut result = bytes.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            anyhow::bail!("Ciphertext too short");
        }

        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let plaintext = self.cipher.decrypt(nonce, &ciphertext[12..]).map_err(|e| anyhow::anyhow!("Decryption faild: {:?}", e))?;

        Ok(plaintext)
    }
}