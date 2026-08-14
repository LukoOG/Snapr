#[repr(u8)]
#[allow(unused)]

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum CompressionType {
    None = 0,
    Zstd = 1,
}

impl From<u8> for CompressionType {
    fn from(value: u8) -> Self {
        match value {
            0 => CompressionType::None,
            1 => CompressionType::Zstd,
            _ => CompressionType::None,
        }
    }
}

impl Into<u8> for CompressionType {
    fn into(self) -> u8 {
        match self {
            CompressionType::None => 0,
            CompressionType::Zstd => 1,
            // _ => 0,
        }
    }
}