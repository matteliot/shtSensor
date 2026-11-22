//! The sensor's I2C address.

/// The 7-bit I2C address, chosen by the level of the ADDR pin (datasheet table 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Address {
    /// ADDR pin tied to VSS: `0x44`. This is how most breakout boards are wired.
    #[default]
    Low,
    /// ADDR pin tied to VDD: `0x45`.
    High,
}

impl Address {
    /// The 7-bit address.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Low => 0x44,
            Self::High => 0x45,
        }
    }
}

impl From<Address> for u8 {
    fn from(address: Address) -> Self {
        address.as_u8()
    }
}
