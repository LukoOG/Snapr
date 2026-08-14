use crate::models::CompressionType;

pub struct ObjectHeader {
    pub version: u8,
    pub flags: u8,
    pub compression: CompressionType,
    pub original_size: u64,
}

impl ObjectHeader {
    pub fn extract(object: &[u8]) -> Option<Self> {
        Some(Self {
            version: object[5],
            flags: object[6],
            compression: object[7].into(),
            original_size: u64::from_le_bytes(object[8..16].try_into().ok()?),
        })
    }
}
