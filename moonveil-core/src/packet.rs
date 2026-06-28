use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packet {
    pub id: u64,
    pub timestamp: u128,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(id: u64, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0) as u128,
            payload: payload.into(),
        }
    }

    pub fn payload_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.payload).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_bincode_roundtrip_preserves_fields() {
        let pkt = Packet {
            id: 42,
            timestamp: 123456789,
            payload: b"hello".to_vec(),
        };

        let bytes = bincode::serialize(&pkt).unwrap();
        let decoded: Packet = bincode::deserialize(&bytes).unwrap();

        assert_eq!(decoded.id, pkt.id);
        assert_eq!(decoded.timestamp, pkt.timestamp);
        assert_eq!(decoded.payload, pkt.payload);
    }
}

