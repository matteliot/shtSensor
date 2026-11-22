//! The sensor's status register (datasheet section 4.11, table 17).

/// A snapshot of the status register.
///
/// Read it with [`Sht3x::status`](crate::Sht3x::status); the alert and reset flags stay set until
/// [`Sht3x::clear_status`](crate::Sht3x::clear_status) is called.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    bits: u16,
}

impl Status {
    const ALERT_PENDING: u16 = 1 << 15;
    const HEATER_ON: u16 = 1 << 13;
    const HUMIDITY_ALERT: u16 = 1 << 11;
    const TEMPERATURE_ALERT: u16 = 1 << 10;
    const RESET_DETECTED: u16 = 1 << 4;
    const COMMAND_FAILED: u16 = 1 << 1;
    const WRITE_CHECKSUM_FAILED: u16 = 1 << 0;

    /// Wraps a raw register value.
    #[must_use]
    pub const fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    /// The raw register value.
    #[must_use]
    pub const fn bits(self) -> u16 {
        self.bits
    }

    /// At least one alert is pending (bit 15).
    #[must_use]
    pub const fn alert_pending(self) -> bool {
        self.is_set(Self::ALERT_PENDING)
    }

    /// The on-chip heater is on (bit 13).
    #[must_use]
    pub const fn heater_on(self) -> bool {
        self.is_set(Self::HEATER_ON)
    }

    /// A humidity tracking alert has fired (bit 11).
    #[must_use]
    pub const fn humidity_alert(self) -> bool {
        self.is_set(Self::HUMIDITY_ALERT)
    }

    /// A temperature tracking alert has fired (bit 10).
    #[must_use]
    pub const fn temperature_alert(self) -> bool {
        self.is_set(Self::TEMPERATURE_ALERT)
    }

    /// The sensor has reset (power-up, soft reset, general call reset or the nRESET pin) since the register was last cleared (bit 4).
    #[must_use]
    pub const fn reset_detected(self) -> bool {
        self.is_set(Self::RESET_DETECTED)
    }

    /// The last command was not processed: it was invalid or failed its internal checksum (bit 1).
    #[must_use]
    pub const fn last_command_failed(self) -> bool {
        self.is_set(Self::COMMAND_FAILED)
    }

    /// The checksum of the last data write did not match, so the sensor ignored the data (bit 0).
    #[must_use]
    pub const fn last_write_checksum_failed(self) -> bool {
        self.is_set(Self::WRITE_CHECKSUM_FAILED)
    }

    const fn is_set(self, mask: u16) -> bool {
        self.bits & mask != 0
    }
}

#[cfg(test)]
mod tests {
    use super::Status;

    #[test]
    fn decodes_each_flag() {
        type Flag = fn(Status) -> bool;
        let cases: [(u16, Flag); 7] = [
            (1 << 15, Status::alert_pending),
            (1 << 13, Status::heater_on),
            (1 << 11, Status::humidity_alert),
            (1 << 10, Status::temperature_alert),
            (1 << 4, Status::reset_detected),
            (1 << 1, Status::last_command_failed),
            (1 << 0, Status::last_write_checksum_failed),
        ];
        for (bit, flag) in cases {
            assert!(flag(Status::from_bits(bit)), "bit {bit:#06x} not reported");
            assert!(
                !flag(Status::from_bits(!bit)),
                "bit {bit:#06x} reported while clear"
            );
        }
    }

    #[test]
    fn keeps_the_raw_bits() {
        assert_eq!(Status::from_bits(0x8010).bits(), 0x8010);
    }
}
