use super::{ObfuscationError, Obfuscator};

pub struct XorObfuscator {
    pub key: Vec<u8>,
}

impl XorObfuscator {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

impl Obfuscator for XorObfuscator {
    fn obfuscate(&self, data: &[u8]) -> Vec<u8> {
        if self.key.is_empty() {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, b)| b ^ self.key[i % self.key.len()])
            .collect()
    }

    fn deobfuscate(&self, data: &[u8]) -> Result<Vec<u8>, ObfuscationError> {
        // XOR is symmetric.
        Ok(self.obfuscate(data))
    }
}
