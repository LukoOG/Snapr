use crate::models::CompressionType;

pub struct ObjectHeader {
    pub version: u8,
    pub flags: u8,
    pub compression: CompressionType,
    pub original_size: u64,
}