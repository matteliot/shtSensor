//! The CRC-8 checksum that protects every 16-bit word on the bus (datasheet section 4.12).

/// Sensirion's CRC-8: polynomial `0x31` (x⁸ + x⁵ + x⁴ + 1), initial value `0xFF`, no reflection and no final XOR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Crc8;

impl Crc8 {
    const POLYNOMIAL: u8 = 0x31;
    const INITIAL: u8 = 0xFF;
    const TOP_BIT: u8 = 0x80;

    /// The checksum of one big-endian data word.
    pub(crate) const fn checksum(data: [u8; 2]) -> u8 {
        let mut crc = Self::INITIAL;
        let mut byte = 0;
        while byte < data.len() {
            crc ^= data[byte];
            let mut bit = 0;
            while bit < 8 {
                crc = if crc & Self::TOP_BIT == 0 {
                    crc << 1
                } else {
                    (crc << 1) ^ Self::POLYNOMIAL
                };
                bit += 1;
            }
            byte += 1;
        }
        crc
    }
}

#[cfg(test)]
mod tests {
    use super::Crc8;

    #[test]
    fn matches_the_datasheet_example() {
        // Datasheet table 19: CRC(0xBEEF) = 0x92.
        assert_eq!(Crc8::checksum([0xBE, 0xEF]), 0x92);
    }

    #[test]
    fn zero_word_has_the_known_checksum() {
        assert_eq!(Crc8::checksum([0x00, 0x00]), 0x81);
    }
}
