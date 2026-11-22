//! The sensor's unique serial number.

use core::fmt;

/// The 32-bit serial number burned into each sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SerialNumber(u32);

impl SerialNumber {
    /// Builds the serial number from the two words the sensor sends, most significant first.
    pub(crate) const fn from_words([high, low]: [u16; 2]) -> Self {
        Self(((high as u32) << 16) | low as u32)
    }

    /// The serial number as an integer.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for SerialNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#010x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::SerialNumber;
    use std::string::ToString;

    #[test]
    fn joins_the_words_most_significant_first() {
        assert_eq!(
            SerialNumber::from_words([0x1234, 0xABCD]).value(),
            0x1234_ABCD
        );
    }

    #[test]
    fn displays_as_hex() {
        assert_eq!(
            SerialNumber::from_words([0x0001, 0x0203]).to_string(),
            "0x00010203"
        );
    }
}
