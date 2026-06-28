use super::{ObfuscationError, Obfuscator};
use rand::RngCore;

pub struct PaddingObfuscator {
    pub min_padding: usize,
    pub max_padding: usize,
}

impl PaddingObfuscator {
    pub fn new(min_padding: usize, max_padding: usize) -> Self {
        Self {
            min_padding,
            max_padding,
        }
    }
}

impl Obfuscator for PaddingObfuscator {
    fn obfuscate(&self, data: &[u8]) -> Vec<u8> {
        let mut rng = rand::thread_rng();

        let padding_range = self.max_padding.saturating_sub(self.min_padding);
        let padding = self.min_padding + (rng.next_u32() as usize % (padding_range + 1));

        let original_len: u16 = data.len().min(u16::MAX as usize) as u16;

        let mut out = Vec::with_capacity(2 + data.len() + padding);
        out.extend_from_slice(&original_len.to_le_bytes());
        out.extend_from_slice(data);

        let mut pad = vec![0u8; padding];
        rng.fill_bytes(&mut pad);
        out.extend_from_slice(&pad);
        out
    }

    fn deobfuscate(&self, data: &[u8]) -> Result<Vec<u8>, ObfuscationError> {
        if data.len() < 2 {
            return Err(ObfuscationError::TooShort);
        }

        let mut len_bytes = [0u8; 2];
        len_bytes.copy_from_slice(&data[..2]);
        let original_len = u16::from_le_bytes(len_bytes) as usize;

        if data.len() < 2 + original_len {
            return Err(ObfuscationError::InvalidData(format!(
                "invalid original_len: {original_len}"
            )));
        }

        Ok(data[2..2 + original_len].to_vec())
    }
}
