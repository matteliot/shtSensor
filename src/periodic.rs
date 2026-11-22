//! Periodic mode: the sensor measures on its own and the driver fetches the latest result.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::{Error as _, ErrorKind, I2c};

use crate::command::Command;
use crate::driver::ModeChange;
use crate::mode::{Periodic, SingleShot};
use crate::{Error, Measurement, Sht3x};

impl<I2C: I2c, D: DelayNs> Sht3x<I2C, D, Periodic> {
    /// Fetches the newest measurement, or `None` if there is no new one since the last fetch.
    ///
    /// The sensor signals "no new data" by not acknowledging the read. Some Linux I2C drivers (including the Raspberry Pi's) report that as
    /// a generic I/O error rather than as [`ErrorKind::NoAcknowledge`]; there it surfaces as [`Error::I2c`]. Fetching no faster than
    /// the configured [`Rate`](crate::Rate) avoids the question.
    ///
    /// # Errors
    /// [`Error::I2c`] on a bus failure, [`Error::Crc`] if the reading was corrupted.
    pub fn fetch(&mut self) -> Result<Option<Measurement>, Error<I2C::Error>> {
        self.bus.command(Command::FetchData)?;
        match self.bus.read_words() {
            Ok(words) => Ok(Some(Measurement::from_words(words))),
            Err(Error::I2c(error)) if matches!(error.kind(), ErrorKind::NoAcknowledge(_)) => {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    /// Stops periodic measurement and returns to single-shot mode.
    ///
    /// # Errors
    /// The driver, still in periodic mode, together with the [`Error`] if the command was not acknowledged.
    pub fn stop(self) -> ModeChange<I2C, D, Periodic, SingleShot, Error<I2C::Error>> {
        self.change_mode(Command::Break)
    }

    /// Soft-resets the sensor, which also ends periodic mode.
    ///
    /// # Errors
    /// The driver, still in periodic mode, together with the [`Error`] if the command was not acknowledged.
    pub fn soft_reset(self) -> ModeChange<I2C, D, Periodic, SingleShot, Error<I2C::Error>> {
        self.change_mode(Command::SoftReset)
    }

    /// Resets the sensor through its nRESET pin, which also ends periodic mode.
    ///
    /// # Errors
    /// The driver, still in periodic mode, together with the pin's error if it could not be driven.
    pub fn hard_reset<P: OutputPin>(
        self,
        reset_pin: &mut P,
    ) -> ModeChange<I2C, D, Periodic, SingleShot, P::Error> {
        self.change_mode_with(|bus| bus.hard_reset(reset_pin))
    }
}
