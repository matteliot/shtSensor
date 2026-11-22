//! Alert mode: the ALERT pin and status flags fire when a periodic measurement crosses a programmable limit
//! (Sensirion application note "SHT3x-DIS Alert Mode").

use crate::{Humidity, Temperature};

/// One of the four programmable alert limits.
///
/// An alert is raised when a periodic measurement goes above [`HighSet`](Self::HighSet) or below [`LowSet`](Self::LowSet),
/// and released once it comes back below [`HighClear`](Self::HighClear) or above [`LowClear`](Self::LowClear). The gap between
/// each set/clear pair is the hysteresis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlertThreshold {
    /// Raise the alert above this limit.
    HighSet,
    /// Release a high alert below this limit.
    HighClear,
    /// Release a low alert above this limit.
    LowClear,
    /// Raise the alert below this limit.
    LowSet,
}

/// The value of one alert limit. The sensor compares humidity and temperature together, at reduced precision: the limit keeps
/// the top 7 bits of the raw humidity and the top 9 bits of the raw temperature, so values read back are rounded down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AlertLimit {
    /// The humidity limit.
    pub humidity: Humidity,
    /// The temperature limit.
    pub temperature: Temperature,
}

impl AlertLimit {
    const HUMIDITY_MASK: u16 = 0xFE00;
    const TEMPERATURE_SHIFT: u32 = 7;
    const TEMPERATURE_FIELD: u16 = 0x01FF;

    /// A limit at `humidity` and `temperature`.
    #[must_use]
    pub const fn new(humidity: Humidity, temperature: Temperature) -> Self {
        Self {
            humidity,
            temperature,
        }
    }

    /// Packs the limit into the sensor's 16-bit format: humidity in bits 15–9, temperature in bits 8–0.
    pub(crate) const fn to_word(self) -> u16 {
        (self.humidity.raw() & Self::HUMIDITY_MASK)
            | (self.temperature.raw() >> Self::TEMPERATURE_SHIFT)
    }

    /// Unpacks a limit read from the sensor; the bits the format drops come back as zero.
    pub(crate) const fn from_word(word: u16) -> Self {
        Self {
            humidity: Humidity::from_raw(word & Self::HUMIDITY_MASK),
            temperature: Temperature::from_raw(
                (word & Self::TEMPERATURE_FIELD) << Self::TEMPERATURE_SHIFT,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AlertLimit;
    use crate::{Humidity, Temperature};

    #[test]
    fn packs_humidity_high_and_temperature_low() {
        let limit = AlertLimit::new(Humidity::from_raw(0xFFFF), Temperature::from_raw(0x0000));
        assert_eq!(limit.to_word(), 0xFE00);
        let limit = AlertLimit::new(Humidity::from_raw(0x0000), Temperature::from_raw(0xFFFF));
        assert_eq!(limit.to_word(), 0x01FF);
    }

    #[test]
    fn round_trips_with_reduced_precision() {
        let limit = AlertLimit::new(
            Humidity::from_percent(80.0),
            Temperature::from_celsius(60.0),
        );
        let back = AlertLimit::from_word(limit.to_word());
        assert_eq!(back.to_word(), limit.to_word());
        assert!((back.humidity.percent() - 80.0).abs() < 100.0 / 128.0);
        assert!((back.temperature.celsius() - 60.0).abs() < 175.0 / 512.0);
    }

    #[test]
    fn decodes_a_packed_limit() {
        // 0xCD33: humidity bits 0xCC00 (about 79.7 %RH), temperature field 0x133 (about 59.9 °C).
        let limit = AlertLimit::from_word(0xCD33);
        assert!((limit.humidity.percent() - 80.0).abs() < 0.5);
        assert!((limit.temperature.celsius() - 60.0).abs() < 0.5);
    }
}
