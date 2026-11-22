//! Clock stretching for single-shot measurements.

/// Whether the sensor holds SCL low until a single-shot measurement is ready (datasheet section 4.3).
///
/// With clock stretching the driver reads straight away and the sensor makes the bus wait; without it the driver waits the
/// measurement's maximum duration before reading. Some I2C controllers handle clock stretching badly (notably the Raspberry Pi's),
/// so the default is [`Disabled`](Self::Disabled).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockStretching {
    /// The sensor holds the bus until the measurement is ready.
    Enabled,
    /// The driver waits for the measurement, then reads.
    #[default]
    Disabled,
}
