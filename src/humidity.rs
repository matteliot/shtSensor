//! A relative humidity reading.

use crate::scale::LinearScale;

/// A relative humidity, stored as the sensor's raw 16-bit value so no precision is lost until a unit is chosen.
///
/// The sensor's range is 0 %RH to 100 %RH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Humidity {
    raw: u16,
}

impl Humidity {
    // Datasheet section 4.13: RH = 100 × raw / 65535.
    const PERCENT: LinearScale = LinearScale::new(0, 100_000);

    /// Wraps a raw sensor value.
    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self { raw }
    }

    /// The nearest representable humidity to `percent` %RH, clamped to 0–100 %RH.
    #[must_use]
    pub fn from_percent(percent: f32) -> Self {
        Self::from_raw(Self::PERCENT.raw_from_float(percent))
    }

    /// The nearest representable humidity to `millipercent` thousandths of a percent, clamped to 0–100 %RH.
    #[must_use]
    pub const fn from_millipercent(millipercent: u32) -> Self {
        // Anything above i32::MAX is far above 100 %RH and clamps to the top of the range either way, so the cast below cannot wrap.
        #[allow(clippy::cast_possible_wrap)]
        let millipercent = if millipercent > i32::MAX as u32 {
            i32::MAX
        } else {
            millipercent as i32
        };
        Self::from_raw(Self::PERCENT.raw_from_milli(millipercent))
    }

    /// The raw 16-bit sensor value.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.raw
    }

    /// Percent relative humidity.
    #[must_use]
    pub fn percent(self) -> f32 {
        Self::PERCENT.float_from_raw(self.raw)
    }

    /// Thousandths of a percent relative humidity, computed with integer arithmetic only.
    #[must_use]
    pub const fn millipercent(self) -> u32 {
        // The scale's offset is zero, so the result is never negative.
        Self::PERCENT.milli_from_raw(self.raw).unsigned_abs()
    }
}

#[cfg(test)]
mod tests {
    use super::Humidity;

    #[test]
    fn converts_the_range_ends() {
        assert_eq!(Humidity::from_raw(0).millipercent(), 0);
        assert_eq!(Humidity::from_raw(u16::MAX).millipercent(), 100_000);
    }

    #[test]
    fn from_percent_lands_within_one_step() {
        assert!((Humidity::from_percent(42.0).percent() - 42.0).abs() < 0.002);
        assert_eq!(
            Humidity::from_millipercent(42_000),
            Humidity::from_percent(42.0)
        );
    }

    #[test]
    fn clamps_out_of_range_input() {
        assert_eq!(Humidity::from_percent(150.0).raw(), u16::MAX);
        assert_eq!(Humidity::from_percent(-5.0).raw(), 0);
        assert_eq!(Humidity::from_millipercent(u32::MAX).raw(), u16::MAX);
    }
}
