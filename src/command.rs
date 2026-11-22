//! Every command the sensor understands, and its 16-bit code (datasheet section 4, application notes for alert mode and the serial number).

use crate::timing::Timing;
use crate::{AlertThreshold, ClockStretching, Rate, Repeatability};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Command {
    SingleShot(Repeatability, ClockStretching),
    Periodic(Rate, Repeatability),
    /// Periodic measurement at 4 Hz with accelerated response time.
    PeriodicArt,
    FetchData,
    Break,
    SoftReset,
    HeaterEnable,
    HeaterDisable,
    ReadStatus,
    ClearStatus,
    ReadSerialNumber,
    ReadAlertLimit(AlertThreshold),
    WriteAlertLimit(AlertThreshold),
}

impl Command {
    pub(crate) const fn code(self) -> u16 {
        use AlertThreshold::{HighClear, HighSet, LowClear, LowSet};
        use ClockStretching::{Disabled, Enabled};
        use Rate::{HalfHz, Hz1, Hz10, Hz2, Hz4};
        use Repeatability::{High, Low, Medium};

        match self {
            // Table 8: single shot.
            Self::SingleShot(High, Enabled) => 0x2C06,
            Self::SingleShot(Medium, Enabled) => 0x2C0D,
            Self::SingleShot(Low, Enabled) => 0x2C10,
            Self::SingleShot(High, Disabled) => 0x2400,
            Self::SingleShot(Medium, Disabled) => 0x240B,
            Self::SingleShot(Low, Disabled) => 0x2416,
            // Table 9: periodic.
            Self::Periodic(HalfHz, High) => 0x2032,
            Self::Periodic(HalfHz, Medium) => 0x2024,
            Self::Periodic(HalfHz, Low) => 0x202F,
            Self::Periodic(Hz1, High) => 0x2130,
            Self::Periodic(Hz1, Medium) => 0x2126,
            Self::Periodic(Hz1, Low) => 0x212D,
            Self::Periodic(Hz2, High) => 0x2236,
            Self::Periodic(Hz2, Medium) => 0x2220,
            Self::Periodic(Hz2, Low) => 0x222B,
            Self::Periodic(Hz4, High) => 0x2334,
            Self::Periodic(Hz4, Medium) => 0x2322,
            Self::Periodic(Hz4, Low) => 0x2329,
            Self::Periodic(Hz10, High) => 0x2737,
            Self::Periodic(Hz10, Medium) => 0x2721,
            Self::Periodic(Hz10, Low) => 0x272A,
            // Tables 10 to 13, 15, 16 and 18.
            Self::FetchData => 0xE000,
            Self::PeriodicArt => 0x2B32,
            Self::Break => 0x3093,
            Self::SoftReset => 0x30A2,
            Self::HeaterEnable => 0x306D,
            Self::HeaterDisable => 0x3066,
            Self::ReadStatus => 0xF32D,
            Self::ClearStatus => 0x3041,
            // Not in the datasheet's command tables; the code used by Sensirion's own SHT3x driver.
            Self::ReadSerialNumber => 0x3780,
            // Alert mode application note.
            Self::ReadAlertLimit(HighSet) => 0xE11F,
            Self::ReadAlertLimit(HighClear) => 0xE114,
            Self::ReadAlertLimit(LowClear) => 0xE109,
            Self::ReadAlertLimit(LowSet) => 0xE102,
            Self::WriteAlertLimit(HighSet) => 0x611D,
            Self::WriteAlertLimit(HighClear) => 0x6116,
            Self::WriteAlertLimit(LowClear) => 0x610B,
            Self::WriteAlertLimit(LowSet) => 0x6100,
        }
    }

    /// How long the sensor is busy after this command before it can be read or sent the next one, in microseconds. The bus never
    /// waits less than the minimum command spacing, so commands that finish at once report zero.
    pub(crate) const fn execution_time_us(self) -> u32 {
        match self {
            Self::SingleShot(repeatability, ClockStretching::Disabled) => {
                repeatability.max_measurement_duration_us()
            }
            Self::SoftReset => Timing::RESET_US,
            // With clock stretching the sensor holds the bus during the read until the result is ready.
            _ => 0,
        }
    }

    pub(crate) const fn to_bytes(self) -> [u8; 2] {
        self.code().to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::Command;
    use crate::{ClockStretching, Rate, Repeatability};

    #[test]
    fn only_slow_commands_report_an_execution_time() {
        assert_eq!(
            Command::SingleShot(Repeatability::High, ClockStretching::Disabled).execution_time_us(),
            15_500
        );
        assert_eq!(
            Command::SingleShot(Repeatability::High, ClockStretching::Enabled).execution_time_us(),
            0
        );
        assert_eq!(Command::SoftReset.execution_time_us(), 1_500);
        assert_eq!(Command::ReadStatus.execution_time_us(), 0);
    }

    #[test]
    fn encodes_big_endian() {
        assert_eq!(Command::SoftReset.to_bytes(), [0x30, 0xA2]);
    }

    #[test]
    fn single_shot_codes_follow_table_8() {
        assert_eq!(
            Command::SingleShot(Repeatability::High, ClockStretching::Disabled).code(),
            0x2400
        );
        assert_eq!(
            Command::SingleShot(Repeatability::Low, ClockStretching::Enabled).code(),
            0x2C10
        );
    }

    #[test]
    fn periodic_codes_follow_table_9() {
        assert_eq!(
            Command::Periodic(Rate::HalfHz, Repeatability::High).code(),
            0x2032
        );
        assert_eq!(
            Command::Periodic(Rate::Hz10, Repeatability::Low).code(),
            0x272A
        );
    }
}
