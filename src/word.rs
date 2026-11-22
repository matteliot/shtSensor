//! A 16-bit data word as it travels on the bus: two big-endian bytes followed by their CRC-8 (datasheet section 4.4).

use crate::crc::Crc8;

/// The checksum that arrived with a word did not match its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CrcMismatch;

/// One CRC-protected data word. Every read (measurements, status, serial number, alert limits) and every write that carries data
/// (alert limits) goes through this type, so the checksum is never skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Word(u16);

impl Word {
    /// Bytes on the wire per word: two data bytes and the checksum.
    pub(crate) const WIRE_LEN: usize = 3;

    pub(crate) const fn new(value: u16) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u16 {
        self.0
    }

    /// Decodes a word from its three wire bytes, rejecting it if the checksum does not match.
    pub(crate) const fn decode(wire: [u8; Self::WIRE_LEN]) -> Result<Self, CrcMismatch> {
        let data = [wire[0], wire[1]];
        if Crc8::checksum(data) == wire[2] {
            Ok(Self(u16::from_be_bytes(data)))
        } else {
            Err(CrcMismatch)
        }
    }

    /// The three wire bytes for this word, checksum included.
    pub(crate) const fn encode(self) -> [u8; Self::WIRE_LEN] {
        let [high, low] = self.0.to_be_bytes();
        [high, low, Crc8::checksum([high, low])]
    }
}

#[cfg(test)]
mod tests {
    use super::{CrcMismatch, Word};

    #[test]
    fn round_trips_through_the_wire_format() {
        let word = Word::new(0xBEEF);
        assert_eq!(word.encode(), [0xBE, 0xEF, 0x92]);
        assert_eq!(Word::decode(word.encode()), Ok(word));
    }

    #[test]
    fn rejects_a_corrupted_checksum() {
        assert_eq!(Word::decode([0xBE, 0xEF, 0x93]), Err(CrcMismatch));
    }

    #[test]
    fn rejects_corrupted_data() {
        assert_eq!(Word::decode([0xBE, 0xEE, 0x92]), Err(CrcMismatch));
    }
}
