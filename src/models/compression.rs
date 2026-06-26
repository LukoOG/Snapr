#[repr(u8)]
pub enum CompressionType {
    None = 0,
    Zstd = 1,
}