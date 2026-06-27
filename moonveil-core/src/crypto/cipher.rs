use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use super::{Cipher, CryptoError};

/// AES-256-GCM cipher implementation.
///
/// Encrypts with a random 12-byte nonce prepended to the ciphertext.
/// Decrypts by extracting the first 12 bytes as the nonce.
pub struct AesGcmCipher {
    key: [u8; 32],
}

impl AesGcmCipher {
    /// Create a new cipher from a 32-byte key.
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Convert from a byte slice; returns an error if not 32 bytes.
    pub fn from_slice(key: &[u8]) -> Result<Self, CryptoError> {
        let arr: [u8; 32] = key.try_into().map_err(|_| {
            CryptoError::KeyGen(format!(
                "invalid key length: expected 32 bytes, got {}",
                key.len()
            ))
        })?;
        Ok(Self::new(arr))
    }
}

impl Cipher for AesGcmCipher {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| CryptoError::Encrypt(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| CryptoError::Encrypt(e.to_string()))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 12 {
            return Err(CryptoError::Decrypt(
                "data too short: missing nonce".into(),
            ));
        }

        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| CryptoError::Decrypt(e.to_string()))?;

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::Decrypt(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let cipher = AesGcmCipher::new(key);

        let plaintext = b"hello moonveil crypto";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypted_data_has_nonce_prepended() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let cipher = AesGcmCipher::new(key);

        let encrypted = cipher.encrypt(b"test").unwrap();
        // nonce (12) + ciphertext + tag
        assert!(encrypted.len() > 12);
    }

    #[test]
    fn decrypt_too_short_fails() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let cipher = AesGcmCipher::new(key);

        let result = cipher.decrypt(b"too short");
        assert!(result.is_err());
    }

    #[test]
    fn from_slice_validates_length() {
        let result = AesGcmCipher::from_slice(&[0u8; 16]);
        assert!(result.is_err());

        let result = AesGcmCipher::from_slice(&[0u8; 32]);
        assert!(result.is_ok());
    }

    #[test]
    fn different_keys_produce_different_ciphertexts() {
        let mut key1 = [0u8; 32];
        let mut key2 = [0u8; 32];
        OsRng.fill_bytes(&mut key1);
        OsRng.fill_bytes(&mut key2);

        let cipher1 = AesGcmCipher::new(key1);
        let cipher2 = AesGcmCipher::new(key2);

        let ct1 = cipher1.encrypt(b"same data").unwrap();
        let ct2 = cipher2.encrypt(b"same data").unwrap();

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let mut key1 = [0u8; 32];
        let mut key2 = [0u8; 32];
        OsRng.fill_bytes(&mut key1);
        OsRng.fill_bytes(&mut key2);

        let cipher1 = AesGcmCipher::new(key1);
        let cipher2 = AesGcmCipher::new(key2);

        let encrypted = cipher1.encrypt(b"secret").unwrap();
        let result = cipher2.decrypt(&encrypted);
        assert!(result.is_err());
    }
}
