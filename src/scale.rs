//! The linear conversion between the sensor's raw 16-bit readings and physical units (datasheet section 4.13), shared by
//! [`Temperature`](crate::Temperature) and [`Humidity`](crate::Humidity).

/// `value = offset + span × raw / 65535`, with `offset` and `span` in thousandths of the unit so the integer path is exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LinearScale {
    offset_milli: i32,
    span_milli: i32,
}

// The casts below are deliberate and bounded: offsets and spans are at most a few hundred thousand (exact in f32), raw values are
// 16-bit, and every narrowing cast is either range-checked first or relies on `as` saturating, as documented at each site.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
impl LinearScale {
    const RAW_MAX: i64 = u16::MAX as i64;

    pub(crate) const fn new(offset_milli: i32, span_milli: i32) -> Self {
        Self {
            offset_milli,
            span_milli,
        }
    }

    /// The value of `raw` in thousandths of the unit, using integer arithmetic only (suitable for MCUs without an FPU).
    pub(crate) const fn milli_from_raw(self, raw: u16) -> i32 {
        // span × raw can exceed i32 (175 000 × 65 535), so widen; the result always fits back into i32.
        (self.offset_milli as i64 + self.span_milli as i64 * raw as i64 / Self::RAW_MAX) as i32
    }

    /// The value of `raw` in whole units.
    pub(crate) fn float_from_raw(self, raw: u16) -> f32 {
        (self.offset_milli as f32 + self.span_milli as f32 * f32::from(raw) / Self::RAW_MAX as f32)
            / 1000.0
    }

    /// The raw reading nearest to `milli` thousandths of the unit, clamped to the sensor's range.
    pub(crate) const fn raw_from_milli(self, milli: i32) -> u16 {
        let numerator = (milli as i64 - self.offset_milli as i64) * Self::RAW_MAX;
        let span = self.span_milli as i64;
        let raw = (numerator + span / 2).div_euclid(span);
        if raw < 0 {
            0
        } else if raw > Self::RAW_MAX {
            u16::MAX
        } else {
            raw as u16
        }
    }

    /// The raw reading nearest to `value` whole units, clamped to the sensor's range (`NaN` maps to the bottom of the range).
    pub(crate) fn raw_from_float(self, value: f32) -> u16 {
        let raw = (value * 1000.0 - self.offset_milli as f32) / self.span_milli as f32
            * Self::RAW_MAX as f32;
        // `as` saturates at the integer's bounds and maps NaN to zero, which is exactly the clamping wanted here.
        (raw + 0.5) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::LinearScale;

    const CELSIUS: LinearScale = LinearScale::new(-45_000, 175_000);

    #[test]
    fn integer_path_covers_the_full_range() {
        assert_eq!(CELSIUS.milli_from_raw(0), -45_000);
        assert_eq!(CELSIUS.milli_from_raw(u16::MAX), 130_000);
    }

    #[test]
    fn float_path_agrees_with_the_integer_path() {
        for raw in [0, 1, 12_345, 32_768, 54_321, u16::MAX] {
            let exact = f64::from(CELSIUS.milli_from_raw(raw)) / 1000.0;
            assert!(
                (f64::from(CELSIUS.float_from_raw(raw)) - exact).abs() < 0.002,
                "raw {raw}"
            );
        }
    }

    #[test]
    fn inverse_round_trips() {
        for raw in [0, 1, 12_345, 32_768, 54_321, u16::MAX] {
            assert!(
                CELSIUS
                    .raw_from_milli(CELSIUS.milli_from_raw(raw))
                    .abs_diff(raw)
                    <= 1,
                "raw {raw}"
            );
            assert!(
                CELSIUS
                    .raw_from_float(CELSIUS.float_from_raw(raw))
                    .abs_diff(raw)
                    <= 1,
                "raw {raw}"
            );
        }
    }

    #[test]
    fn inverse_clamps_out_of_range_values() {
        assert_eq!(CELSIUS.raw_from_milli(-100_000), 0);
        assert_eq!(CELSIUS.raw_from_milli(500_000), u16::MAX);
        assert_eq!(CELSIUS.raw_from_float(-100.0), 0);
        assert_eq!(CELSIUS.raw_from_float(500.0), u16::MAX);
        assert_eq!(CELSIUS.raw_from_float(f32::NAN), 0);
    }
}
