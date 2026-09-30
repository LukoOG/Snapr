use std::fmt;

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompressionType {
    None = 0,
    Zstd = 1,
}

#[derive(Debug)]
pub struct InvalidCompressionType(pub u8);

impl fmt::Display for InvalidCompressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Unknown CompressionType: {}", self.0))
    }
}

impl std::error::Error for InvalidCompressionType {}
// impl From<u8> for CompressionType {
//     fn from(value: u8) -> Self {
//         match value {
//             0 => CompressionType::None,
//             1 => CompressionType::Zstd,
//             _ => CompressionType::None,
//         }
//     }
// }
impl TryFrom<u8> for CompressionType {
    type Error = InvalidCompressionType;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Zstd),
            _ => Err(InvalidCompressionType(value)),
        }
    }
}

impl From<CompressionType> for u8 {
    fn from(val: CompressionType) -> Self {
        match val {
            CompressionType::None => 0,
            CompressionType::Zstd => 1,
            // _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TryFrom: known values round-trip.
    #[test]
    fn try_from_0_gives_none() {
        assert_eq!(CompressionType::try_from(0).unwrap(), CompressionType::None);
    }

    #[test]
    fn try_from_1_gives_zstd() {
        assert_eq!(CompressionType::try_from(1).unwrap(), CompressionType::Zstd);
    }

    // TryFrom: unknown byte returns Err.
    #[test]
    fn try_from_unknown_is_err() {
        for bad in [2u8, 128, 255] {
            assert!(
                CompressionType::try_from(bad).is_err(),
                "expected Err for byte {bad}"
            );
        }
    }

    // Into<u8>: round-trips correctly.
    #[test]
    fn into_u8_roundtrip() {
        let n: u8 = CompressionType::None.into();
        let z: u8 = CompressionType::Zstd.into();
        assert_eq!(n, 0);
        assert_eq!(z, 1);
    }

    // InvalidCompressionType error carries the unknown byte.
    #[test]
    fn invalid_compression_type_carries_value() {
        let err = CompressionType::try_from(42u8).unwrap_err();
        assert_eq!(err.0, 42);
        assert!(err.to_string().contains("42"));
    }
}
