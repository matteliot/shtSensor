//! Periodic measurement rate.

/// How often the sensor measures in periodic mode (datasheet table 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Rate {
    /// One measurement every 2 seconds.
    HalfHz,
    /// One measurement per second.
    Hz1,
    /// Two measurements per second.
    Hz2,
    /// Four measurements per second.
    Hz4,
    /// Ten measurements per second. Measuring this often, especially with high repeatability, can warm the sensor slightly.
    Hz10,
}

impl Rate {
    /// The time between measurements, in milliseconds.
    #[must_use]
    pub const fn period_ms(self) -> u32 {
        match self {
            Self::HalfHz => 2_000,
            Self::Hz1 => 1_000,
            Self::Hz2 => 500,
            Self::Hz4 => 250,
            Self::Hz10 => 100,
        }
    }
}
