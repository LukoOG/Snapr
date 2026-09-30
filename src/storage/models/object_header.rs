use crate::models::CompressionType;

#[allow(unused)]
pub struct ObjectHeader {
    pub version: u8,
    pub flags: u8,
    pub compression: CompressionType,
    pub original_size: u64,
}

impl ObjectHeader {
    pub fn extract(object: &[u8]) -> Option<Self> {
        if object.len() < 16 {
            return None
        }
        Some(Self {
            version: object[5],
            flags: object[6],
            compression: object[7].try_into().ok()?,
            original_size: u64::from_le_bytes(object[8..16].try_into().ok()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CompressionType;

    fn make_object(compression: u8, original_size: u64, payload: &[u8]) -> Vec<u8> {
        let mut obj = Vec::new();
        obj.extend_from_slice(b"SNAPR"); // magic
        obj.push(1);                     // version
        obj.push(0);                     // flags
        obj.push(compression);           // compression byte
        obj.extend_from_slice(&original_size.to_le_bytes());
        obj.extend_from_slice(payload);
        obj
    }

    // Valid Zstd object parses correctly.
    #[test]
    fn extract_valid_zstd_object() {
        let obj = make_object(1, 42, b"payload");
        let header = ObjectHeader::extract(&obj).unwrap();
        assert_eq!(header.version, 1);
        assert_eq!(header.flags, 0);
        assert_eq!(header.compression, CompressionType::Zstd);
        assert_eq!(header.original_size, 42);
    }

    // Object shorter than 16 bytes returns None.
    #[test]
    fn extract_too_short_returns_none() {
        let obj = b"SNAPR\x01\x00\x01".to_vec(); // only 8 bytes
        assert!(ObjectHeader::extract(&obj).is_none());
    }

    // Unknown compression byte (e.g. 99) causes extract to return None (TryFrom fails).
    #[test]
    fn extract_unknown_compression_returns_none() {
        let obj = make_object(99, 0, b"");
        assert!(
            ObjectHeader::extract(&obj).is_none(),
            "unknown compression byte should return None"
        );
    }

    // Valid CompressionType::None (value 0) parses.
    #[test]
    fn extract_compression_none_parses() {
        let obj = make_object(0, 10, b"data");
        let header = ObjectHeader::extract(&obj).unwrap();
        assert_eq!(header.compression, CompressionType::None);
    }

    // Exactly 16-byte object (header only, empty payload) parses.
    #[test]
    fn extract_exact_16_bytes_succeeds() {
        let obj = make_object(1, 0, b""); // 16 bytes total
        assert_eq!(obj.len(), 16);
        let header = ObjectHeader::extract(&obj).unwrap();
        assert_eq!(header.original_size, 0);
    }
}
