//! A temperature reading.

use crate::scale::LinearScale;

/// A temperature, stored as the sensor's raw 16-bit value so no precision is lost until a unit is chosen.
///
/// The sensor's range is −45 °C to 130 °C (−49 °F to 266 °F).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Temperature {
    raw: u16,
}

impl Temperature {
    // Datasheet section 4.13: T[°C] = −45 + 175 × raw / 65535, T[°F] = −49 + 315 × raw / 65535.
    const CELSIUS: LinearScale = LinearScale::new(-45_000, 175_000);
    const FAHRENHEIT: LinearScale = LinearScale::new(-49_000, 315_000);

    /// Wraps a raw sensor value.
    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self { raw }
    }

    /// The nearest representable temperature to `celsius`, clamped to the sensor's range.
    #[must_use]
    pub fn from_celsius(celsius: f32) -> Self {
        Self::from_raw(Self::CELSIUS.raw_from_float(celsius))
    }

    /// The nearest representable temperature to `millidegrees` thousandths of a degree Celsius, clamped to the sensor's range.
    #[must_use]
    pub const fn from_millidegrees_celsius(millidegrees: i32) -> Self {
        Self::from_raw(Self::CELSIUS.raw_from_milli(millidegrees))
    }

    /// The raw 16-bit sensor value.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.raw
    }

    /// Degrees Celsius.
    #[must_use]
    pub fn celsius(self) -> f32 {
        Self::CELSIUS.float_from_raw(self.raw)
    }

    /// Degrees Fahrenheit.
    #[must_use]
    pub fn fahrenheit(self) -> f32 {
        Self::FAHRENHEIT.float_from_raw(self.raw)
    }

    /// Thousandths of a degree Celsius, computed with integer arithmetic only.
    #[must_use]
    pub const fn millidegrees_celsius(self) -> i32 {
        Self::CELSIUS.milli_from_raw(self.raw)
    }

    /// Thousandths of a degree Fahrenheit, computed with integer arithmetic only.
    #[must_use]
    pub const fn millidegrees_fahrenheit(self) -> i32 {
        Self::FAHRENHEIT.milli_from_raw(self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::Temperature;

    #[test]
    fn converts_the_range_ends() {
        assert_eq!(Temperature::from_raw(0).millidegrees_celsius(), -45_000);
        assert_eq!(
            Temperature::from_raw(u16::MAX).millidegrees_celsius(),
            130_000
        );
        assert_eq!(Temperature::from_raw(0).millidegrees_fahrenheit(), -49_000);
        assert_eq!(
            Temperature::from_raw(u16::MAX).millidegrees_fahrenheit(),
            266_000
        );
    }

    #[test]
    fn celsius_and_fahrenheit_agree() {
        let t = Temperature::from_celsius(21.5);
        assert!((t.fahrenheit() - (t.celsius() * 9.0 / 5.0 + 32.0)).abs() < 0.01);
    }

    #[test]
    fn from_celsius_lands_within_one_step() {
        assert!((Temperature::from_celsius(25.0).celsius() - 25.0).abs() < 0.003);
        assert_eq!(
            Temperature::from_millidegrees_celsius(25_000),
            Temperature::from_celsius(25.0)
        );
    }
}
