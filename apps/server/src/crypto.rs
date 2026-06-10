use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, Nonce},
};
use hmac::{KeyInit, digest::common::Generate};
use secrecy::{ExposeSecret, SecretSlice, SecretString};

use crate::error::AppError;

pub struct CryptoEngine {
    key: Key<Aes256Gcm>,
}

impl CryptoEngine {
    pub fn new(encryption_key: SecretSlice<u8>) -> crate::Result<Self> {
        Ok(Self {
            key: Key::<Aes256Gcm>::try_from(encryption_key.expose_secret())
                .map_err(|e| AppError::InternalError(e.into()))?,
        })
    }

    #[instrument(skip(self, payload), err(Debug))]
    pub fn encrypt(&self, payload: &str) -> crate::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(&self.key);

        let nonce = Nonce::<Aes256Gcm>::generate();
        let ciphertext = cipher
            .encrypt(&nonce, payload.as_bytes())
            .map_err(|e| AppError::InternalError(e.into()))?;

        let mut packed = nonce.to_vec();
        packed.extend_from_slice(&ciphertext);

        Ok(packed)
    }

    #[instrument(skip(self, packed), err(Debug))]
    pub fn decrypt(&self, packed: &[u8]) -> crate::Result<SecretString> {
        if packed.len() < 12 {
            return Err(AppError::InternalError(anyhow::anyhow!(
                "too short iphertext payload"
            )));
        }

        let (nonce_bytes, ciphertext) = packed.split_at(12);
        let nonce = Nonce::<Aes256Gcm>::try_from(nonce_bytes)
            .map_err(|e| AppError::InternalError(e.into()))?;

        let cipher = Aes256Gcm::new(&self.key);
        let decrypted_bytes = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| AppError::InternalError(e.into()))?;

        let plaintext =
            String::from_utf8(decrypted_bytes).map_err(|e| AppError::InternalError(e.into()))?;

        Ok(plaintext.into())
    }
}
