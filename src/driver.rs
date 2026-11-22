//! The driver type and what every mode shares.

use core::marker::PhantomData;

use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

use crate::bus::Bus;
use crate::command::Command;
use crate::mode::{Mode, SingleShot};
use crate::{Address, Error, ModeChangeError};

/// The outcome of a mode change from `From` to `To`: the driver in its new mode, or the driver back in its old mode with the error `E`.
pub(crate) type ModeChange<I2C, D, From, To, E> =
    Result<Sht3x<I2C, D, To>, ModeChangeError<Sht3x<I2C, D, From>, E>>;

/// A driver for one SHT3x-DIS sensor, in measurement mode `M` ([`SingleShot`] or [`Periodic`](crate::Periodic)).
///
/// The driver owns the I2C bus and a delay provider; use [`release`](Self::release) to get them back. To share one bus between
/// several devices, pass a bus-sharing wrapper such as those in the `embedded-hal-bus` crate.
#[derive(Debug)]
pub struct Sht3x<I2C, D, M: Mode = SingleShot> {
    pub(crate) bus: Bus<I2C, D>,
    mode: PhantomData<M>,
}

impl<I2C, D, M: Mode> Sht3x<I2C, D, M> {
    pub(crate) const fn from_bus(bus: Bus<I2C, D>) -> Self {
        Self {
            bus,
            mode: PhantomData,
        }
    }

    /// The sensor's I2C address.
    #[must_use]
    pub const fn address(&self) -> Address {
        self.bus.address()
    }

    /// Gives back the I2C bus and the delay provider. The sensor is left in whatever state it was in.
    pub fn release(self) -> (I2C, D) {
        self.bus.release()
    }

    /// Runs `step`, which moves the sensor to mode `N`; on failure the driver comes back unchanged, with the step's error.
    pub(crate) fn change_mode_with<N: Mode, E>(
        mut self,
        step: impl FnOnce(&mut Bus<I2C, D>) -> Result<(), E>,
    ) -> ModeChange<I2C, D, M, N, E> {
        match step(&mut self.bus) {
            Ok(()) => Ok(Sht3x::from_bus(self.bus)),
            Err(error) => Err(ModeChangeError {
                driver: self,
                error,
            }),
        }
    }
}

impl<I2C: I2c, D: DelayNs, M: Mode> Sht3x<I2C, D, M> {
    /// Sends a command that moves the sensor to mode `N`; on failure the driver comes back unchanged.
    pub(crate) fn change_mode<N: Mode>(
        self,
        command: Command,
    ) -> ModeChange<I2C, D, M, N, Error<I2C::Error>> {
        self.change_mode_with(|bus| bus.command(command))
    }
}
