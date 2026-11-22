//! Fixed delays from the datasheet (table 4 and section 4), for a 2.4–5.5 V supply.

/// The datasheet's fixed timings, in microseconds unless noted.
pub(crate) struct Timing;

impl Timing {
    /// Minimum idle time between commands.
    pub(crate) const COMMAND_SPACING_US: u32 = 1_000;
    /// Longest time the sensor takes to come back after a soft reset, general call reset or nRESET pulse.
    pub(crate) const RESET_US: u32 = 1_500;
    /// Minimum low pulse on the nRESET pin, in nanoseconds.
    pub(crate) const RESET_PULSE_NS: u32 = 1_000;
}
