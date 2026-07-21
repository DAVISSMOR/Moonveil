use crate::error::{FrameError, Result};

/// Moonveil v0 protocol version.
pub const FRAME_VERSION: u8 = 0;

/// Fixed header size in bytes (version through payload_length, exclusive of payload/checksum).
pub const HEADER_SIZE: usize = 34;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoonveilFrame {
    pub version: u8,
    pub flags: u8,
    pub stream_id: u32,
    pub packet_id: u64,
    pub timestamp: u128,
    pub payload_length: u32,
    pub payload: Vec<u8>,
    pub checksum: u32,
}

impl MoonveilFrame {
    pub fn new(stream_id: u32, packet_id: u64, payload: impl Into<Vec<u8>>) -> Self {
        let payload = payload.into();
        let payload_length = u32::try_from(payload.len()).unwrap_or(u32::MAX);
        Self {
            version: FRAME_VERSION,
            flags: 0,
            stream_id,
            packet_id,
            timestamp: unix_timestamp_ms(),
            payload_length,
            payload,
            checksum: 0,
        }
    }

    pub fn payload_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.payload).ok()
    }
}

pub fn encode_frame(frame: &MoonveilFrame) -> Result<Vec<u8>> {
    let payload_len = frame.payload.len();
    if payload_len != frame.payload_length as usize {
        return Err(FrameError::FrameDecodeError(format!(
            "payload length mismatch: field={} actual={payload_len}",
            frame.payload_length
        ))
        .into());
    }

    let mut buf = Vec::with_capacity(HEADER_SIZE + payload_len + 4);
    write_header(&mut buf, frame);
    buf.extend_from_slice(&frame.payload);

    let checksum = crc32(&buf);
    buf.extend_from_slice(&checksum.to_le_bytes());
    Ok(buf)
}

pub fn decode_frame(bytes: &[u8]) -> Result<MoonveilFrame> {
    if bytes.len() < HEADER_SIZE + 4 {
        return Err(FrameError::FrameDecodeError(format!(
            "buffer too short: {} bytes (minimum {})",
            bytes.len(),
            HEADER_SIZE + 4
        ))
        .into());
    }

    let version = bytes[0];
    if version != FRAME_VERSION {
        return Err(FrameError::InvalidVersion(version).into());
    }

    let flags = bytes[1];
    let stream_id = u32::from_le_bytes(
        bytes[2..6].try_into()
            .map_err(|_| FrameError::FrameDecodeError("invalid stream_id bytes".into()))?
    );
    let packet_id = u64::from_le_bytes(
        bytes[6..14].try_into()
            .map_err(|_| FrameError::FrameDecodeError("invalid packet_id bytes".into()))?
    );
    let timestamp = u128::from_le_bytes(
        bytes[14..30].try_into()
            .map_err(|_| FrameError::FrameDecodeError("invalid timestamp bytes".into()))?
    );
    let payload_length = u32::from_le_bytes(
        bytes[30..34].try_into()
            .map_err(|_| FrameError::FrameDecodeError("invalid payload_length bytes".into()))?
    );

    let payload_end = HEADER_SIZE
        .checked_add(payload_length as usize)
        .ok_or_else(|| {
            FrameError::FrameDecodeError("payload length overflow".into())
        })?;
    let checksum_offset = payload_end;
    let expected_len = checksum_offset + 4;

    if bytes.len() != expected_len {
        return Err(FrameError::FrameDecodeError(format!(
            "frame size mismatch: got {actual} bytes, expected {expected_len} (payload_length={payload_length})", actual = bytes.len()
        ))
        .into());
    }

    let payload = bytes[HEADER_SIZE..payload_end].to_vec();
    let checksum = u32::from_le_bytes(
        bytes[checksum_offset..expected_len].try_into()
            .map_err(|_| FrameError::FrameDecodeError("invalid checksum bytes".into()))?
    );

    let computed = crc32(&bytes[..checksum_offset]);
    if computed != checksum {
        return Err(FrameError::ChecksumMismatch {
            expected: computed,
            actual: checksum,
        }
        .into());
    }

    Ok(MoonveilFrame {
        version,
        flags,
        stream_id,
        packet_id,
        timestamp,
        payload_length,
        payload,
        checksum,
    })
}

fn write_header(buf: &mut Vec<u8>, frame: &MoonveilFrame) {
    buf.push(frame.version);
    buf.push(frame.flags);
    buf.extend_from_slice(&frame.stream_id.to_le_bytes());
    buf.extend_from_slice(&frame.packet_id.to_le_bytes());
    buf.extend_from_slice(&frame.timestamp.to_le_bytes());
    buf.extend_from_slice(&frame.payload_length.to_le_bytes());
}

fn unix_timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

const fn make_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

const CRC32_TABLE: [u32; 256] = make_crc32_table();

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        let idx = ((crc ^ u32::from(byte)) & 0xFF) as usize;
        crc = CRC32_TABLE[idx] ^ (crc >> 8);
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encode_decode() {
        let frame = MoonveilFrame::new(1, 42, b"hello moonveil");
        let bytes = encode_frame(&frame).unwrap();
        let decoded = decode_frame(&bytes).unwrap();
        assert_eq!(decoded.version, FRAME_VERSION);
        assert_eq!(decoded.stream_id, 1);
        assert_eq!(decoded.packet_id, 42);
        assert_eq!(decoded.payload, b"hello moonveil");
        assert_eq!(decoded.checksum, crc32(&bytes[..bytes.len() - 4]));
    }

    #[test]
    fn rejects_invalid_version() {
        let mut frame = MoonveilFrame::new(0, 1, b"test");
        frame.version = 99;
        let bytes = encode_frame(&frame).unwrap();
        let err = decode_frame(&bytes).unwrap_err();
        assert!(matches!(
            err,
            crate::error::Error::Frame(FrameError::InvalidVersion(99))
        ));
    }

    #[test]
    fn rejects_checksum_mismatch() {
        let frame = MoonveilFrame::new(0, 1, b"test");
        let mut bytes = encode_frame(&frame).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        let err = decode_frame(&bytes).unwrap_err();
        assert!(matches!(
            err,
            crate::error::Error::Frame(FrameError::ChecksumMismatch { .. })
        ));
    }
}
