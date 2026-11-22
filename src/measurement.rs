//! One temperature and humidity reading.

use crate::{Humidity, Temperature};

/// A temperature and humidity pair taken in the same measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Measurement {
    /// The temperature.
    pub temperature: Temperature,
    /// The relative humidity.
    pub humidity: Humidity,
}

impl Measurement {
    /// Builds a measurement from the two words the sensor sends, temperature first (datasheet section 4.4).
    pub(crate) const fn from_words([temperature, humidity]: [u16; 2]) -> Self {
        Self {
            temperature: Temperature::from_raw(temperature),
            humidity: Humidity::from_raw(humidity),
        }
    }
}
