//! Single-shot mode: measurements on demand, plus every configuration and diagnostic command.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;

use crate::bus::Bus;
use crate::command::Command;
use crate::driver::ModeChange;
use crate::mode::{Periodic, SingleShot};
use crate::{
    Address, AlertLimit, AlertThreshold, ClockStretching, Error, Measurement, Rate, Repeatability,
    SerialNumber, Sht3x, Status,
};

impl<I2C: I2c, D: DelayNs> Sht3x<I2C, D, SingleShot> {
    /// A driver for the sensor at `address`. No bus traffic happens until the first command.
    ///
    /// After power-up the sensor needs up to 1.5 ms before it accepts commands.
    pub const fn new(i2c: I2C, delay: D, address: Address) -> Self {
        Self::from_bus(Bus::new(i2c, delay, address))
    }

    /// Takes one measurement, waiting for it without clock stretching.
    ///
    /// # Errors
    /// [`Error::I2c`] if the sensor does not answer (for example when it is not connected), [`Error::Crc`] if the reading was corrupted.
    pub fn measure(
        &mut self,
        repeatability: Repeatability,
    ) -> Result<Measurement, Error<I2C::Error>> {
        self.measure_with(repeatability, ClockStretching::Disabled)
    }

    /// Takes one measurement with the given clock stretching setting.
    ///
    /// # Errors
    /// As for [`measure`](Self::measure).
    pub fn measure_with(
        &mut self,
        repeatability: Repeatability,
        clock_stretching: ClockStretching,
    ) -> Result<Measurement, Error<I2C::Error>> {
        self.bus
            .query(Command::SingleShot(repeatability, clock_stretching))
            .map(Measurement::from_words)
    }

    /// Switches the on-chip heater on. While it is on, readings run warm and dry; it is meant for plausibility checks and for
    /// driving off condensation, not for normal measurement. Check its state with [`Status::heater_on`].
    ///
    /// # Errors
    /// [`Error::I2c`] if the command was not acknowledged.
    pub fn enable_heater(&mut self) -> Result<(), Error<I2C::Error>> {
        self.bus.command(Command::HeaterEnable)
    }

    /// Switches the on-chip heater off.
    ///
    /// # Errors
    /// [`Error::I2c`] if the command was not acknowledged.
    pub fn disable_heater(&mut self) -> Result<(), Error<I2C::Error>> {
        self.bus.command(Command::HeaterDisable)
    }

    /// Reads the status register.
    ///
    /// # Errors
    /// [`Error::I2c`] on a bus failure, [`Error::Crc`] if the register value was corrupted.
    pub fn status(&mut self) -> Result<Status, Error<I2C::Error>> {
        self.bus
            .query(Command::ReadStatus)
            .map(|[bits]| Status::from_bits(bits))
    }

    /// Clears the alert and reset-detected flags in the status register.
    ///
    /// # Errors
    /// [`Error::I2c`] if the command was not acknowledged.
    pub fn clear_status(&mut self) -> Result<(), Error<I2C::Error>> {
        self.bus.command(Command::ClearStatus)
    }

    /// Reads the sensor's unique serial number.
    ///
    /// # Errors
    /// [`Error::I2c`] on a bus failure, [`Error::Crc`] if the value was corrupted.
    pub fn serial_number(&mut self) -> Result<SerialNumber, Error<I2C::Error>> {
        self.bus
            .query(Command::ReadSerialNumber)
            .map(SerialNumber::from_words)
    }

    /// Reads one alert limit.
    ///
    /// # Errors
    /// [`Error::I2c`] on a bus failure, [`Error::Crc`] if the value was corrupted.
    pub fn alert_limit(
        &mut self,
        threshold: AlertThreshold,
    ) -> Result<AlertLimit, Error<I2C::Error>> {
        self.bus
            .query(Command::ReadAlertLimit(threshold))
            .map(|[word]| AlertLimit::from_word(word))
    }

    /// Writes one alert limit. The limits take effect in periodic mode, where the ALERT pin and the status register's alert flags
    /// follow them. Limits are volatile and return to their defaults on any reset. If the sensor rejects the data's checksum it
    /// ignores the write and sets [`Status::last_write_checksum_failed`].
    ///
    /// # Errors
    /// [`Error::I2c`] if the write was not acknowledged.
    pub fn set_alert_limit(
        &mut self,
        threshold: AlertThreshold,
        limit: AlertLimit,
    ) -> Result<(), Error<I2C::Error>> {
        self.bus
            .command_with_word(Command::WriteAlertLimit(threshold), limit.to_word())
    }

    /// Soft-resets the sensor: it reloads its calibration and returns every setting to its default.
    ///
    /// # Errors
    /// [`Error::I2c`] if the command was not acknowledged.
    pub fn soft_reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.bus.command(Command::SoftReset)
    }

    /// Resets the sensor through its nRESET pin, which works even when the sensor no longer answers on the bus.
    ///
    /// # Errors
    /// The pin's error if it could not be driven.
    pub fn hard_reset<P: OutputPin>(&mut self, reset_pin: &mut P) -> Result<(), P::Error> {
        self.bus.hard_reset(reset_pin)
    }

    /// Sends an I2C general call reset. This resets **every** device on the bus that honours the general call, not only this sensor.
    ///
    /// # Errors
    /// [`Error::I2c`] if no device acknowledged the general call.
    pub fn general_call_reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.bus.general_call_reset()
    }

    /// Starts measuring continuously at `rate`. Read the results with [`Sht3x::fetch`].
    ///
    /// # Errors
    /// The driver, unchanged, together with the [`Error`] if the command was not acknowledged.
    pub fn start_periodic(
        self,
        rate: Rate,
        repeatability: Repeatability,
    ) -> ModeChange<I2C, D, SingleShot, Periodic, Error<I2C::Error>> {
        self.change_mode(Command::Periodic(rate, repeatability))
    }

    /// Starts measuring continuously at 4 Hz with accelerated response time (ART). Read the results with [`Sht3x::fetch`].
    ///
    /// # Errors
    /// The driver, unchanged, together with the [`Error`] if the command was not acknowledged.
    pub fn start_art(self) -> ModeChange<I2C, D, SingleShot, Periodic, Error<I2C::Error>> {
        self.change_mode(Command::PeriodicArt)
    }
}
