//! Errors.

use core::fmt;

use crate::word::CrcMismatch;

/// An error talking to the sensor. `E` is the I2C bus's own error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// The I2C transfer failed. A sensor that is missing or unpowered shows up here as a NACK.
    I2c(E),
    /// Data arrived with a checksum that did not match, so it was discarded.
    Crc,
}

impl<E> From<CrcMismatch> for Error<E> {
    fn from(_: CrcMismatch) -> Self {
        Self::Crc
    }
}

impl<E: fmt::Debug> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I2c(error) => write!(f, "I2C transfer failed: {error:?}"),
            Self::Crc => f.write_str("checksum mismatch in data from the sensor"),
        }
    }
}

impl<E: fmt::Debug> core::error::Error for Error<E> {}

/// A mode change (for example starting or stopping periodic measurement) failed. The driver is handed back in its original
/// mode so the caller can retry, reset or release it.
pub struct ModeChangeError<T, E> {
    /// The driver, still in the mode it was in before the attempt.
    pub driver: T,
    /// Why the change failed.
    pub error: E,
}

impl<T, E: fmt::Debug> fmt::Debug for ModeChangeError<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The driver is deliberately left out: it is usually not Debug-friendly and adds nothing to the failure.
        f.debug_struct("ModeChangeError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl<T, E: fmt::Debug> fmt::Display for ModeChangeError<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sensor mode change failed: {:?}", self.error)
    }
}

impl<T, E: fmt::Debug> core::error::Error for ModeChangeError<T, E> {}
