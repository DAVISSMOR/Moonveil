use rand::RngCore;

use super::cipher::AesGcmCipher;
use super::{Cipher, CryptoError};
use crate::packet::Packet;
use crate::transport::Transport;

/// Simple hex decoder (no external dependency needed).
fn hex_decode(s: &str) -> Result<Vec<u8>, CryptoError> {
    if s.len() % 2 != 0 {
        return Err(CryptoError::KeyGen(
            "hex string must have even length".into(),
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| {
                CryptoError::KeyGen(format!("invalid hex at position {i}: {e}"))
            })
        })
        .collect()
}

/// Perform a mutual proof-of-possession handshake over the given transport
/// using a static pre-shared key read from the `MOONVEIL_PSK` environment
/// variable (hex-encoded, 32 bytes / 64 hex chars).
///
/// Protocol:
///   1. Generate 32 random bytes (challenge).
///   2. Encrypt the challenge with the PSK and send as a Packet.
///   3. Receive a Packet back, decrypt the payload.
///   4. Verify that the decrypted payload equals `challenge ^ 0xFF`.
///
/// On success, returns an `AesGcmCipher` wrapping the shared key.
pub async fn perform(transport: &dyn Transport) -> Result<Box<dyn Cipher + Send + Sync>, CryptoError>
{
    let psk_hex = std::env::var("MOONVEIL_PSK").map_err(|_| {
        CryptoError::Handshake("MOONVEIL_PSK environment variable not set".into())
    })?;

    let psk_bytes = hex_decode(&psk_hex)?;

    let key: [u8; 32] = psk_bytes.try_into().map_err(|_| {
        CryptoError::Handshake(
            "MOONVEIL_PSK must be exactly 32 bytes (64 hex characters)".into(),
        )
    })?;

    let cipher = AesGcmCipher::new(key);

    // Phase 1: send encrypted challenge
    let mut challenge = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut challenge);

    let encrypted_challenge = cipher
        .encrypt(&challenge)
        .map_err(|e| CryptoError::Handshake(format!("encrypt challenge failed: {e}")))?;

    transport
        .send(Packet::new(0, encrypted_challenge))
        .await
        .map_err(|e| CryptoError::Transport(format!("send challenge failed: {e}")))?;

    // Phase 2: receive encrypted response
    let response_packet = transport
        .recv()
        .await
        .map_err(|e| CryptoError::Transport(format!("recv response failed: {e}")))?;

    let decrypted = cipher.decrypt(&response_packet.payload).map_err(|_| {
        CryptoError::Handshake("handshake failed: unable to decrypt response".into())
    })?;

    // Verify: response should be challenge XOR 0xFF
    let expected: Vec<u8> = challenge.iter().map(|b| b ^ 0xFF).collect();
    if decrypted != expected {
        return Err(CryptoError::Handshake(
            "handshake failed: challenge response mismatch".into(),
        ));
    }

    Ok(Box::new(cipher))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_decode_valid() {
        let result = hex_decode("deadbeef").unwrap();
        assert_eq!(result, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hex_decode_empty() {
        let result = hex_decode("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn hex_decode_odd_length() {
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn hex_decode_invalid_chars() {
        assert!(hex_decode("xxyy").is_err());
    }
}
