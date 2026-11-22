//! Measurement repeatability.

/// How many samples the sensor averages per measurement: higher repeatability means less noise but a longer measurement
/// (datasheet tables 2 and 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Repeatability {
    /// Lowest noise, longest measurement (up to 15.5 ms).
    #[default]
    High,
    /// Up to 6.5 ms.
    Medium,
    /// Noisiest, shortest measurement (up to 4.5 ms).
    Low,
}

impl Repeatability {
    /// The longest a single measurement can take at this repeatability, in microseconds (datasheet table 4, maximum column).
    #[must_use]
    pub const fn max_measurement_duration_us(self) -> u32 {
        match self {
            Self::High => 15_500,
            Self::Medium => 6_500,
            Self::Low => 4_500,
        }
    }
}
