//! The I2C link to one sensor: sends commands, writes and reads CRC-protected words, keeps the datasheet's spacing between
//! commands, and performs the resets. Every driver operation is built from these few primitives.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;

use crate::command::Command;
use crate::timing::Timing;
use crate::word::Word;
use crate::{Address, Error};

/// The most words any command returns (a measurement or the serial number).
const MAX_WORDS: usize = 2;

/// The I2C general call address and the reset command sent to it (datasheet section 4.9).
const GENERAL_CALL_ADDRESS: u8 = 0x00;
const GENERAL_CALL_RESET: u8 = 0x06;

#[derive(Debug)]
pub(crate) struct Bus<I2C, D> {
    i2c: I2C,
    delay: D,
    address: Address,
}

impl<I2C, D> Bus<I2C, D> {
    pub(crate) const fn new(i2c: I2C, delay: D, address: Address) -> Self {
        Self {
            i2c,
            delay,
            address,
        }
    }

    pub(crate) const fn address(&self) -> Address {
        self.address
    }

    pub(crate) fn release(self) -> (I2C, D) {
        (self.i2c, self.delay)
    }
}

impl<I2C: I2c, D: DelayNs> Bus<I2C, D> {
    /// Sends `command`, then waits until it has finished (never less than the minimum command spacing).
    pub(crate) fn command(&mut self, command: Command) -> Result<(), Error<I2C::Error>> {
        self.write(&command.to_bytes(), command.execution_time_us())
    }

    /// Sends `command` followed by one CRC-protected data word.
    pub(crate) fn command_with_word(
        &mut self,
        command: Command,
        word: u16,
    ) -> Result<(), Error<I2C::Error>> {
        let [command_high, command_low] = command.to_bytes();
        let [data_high, data_low, crc] = Word::new(word).encode();
        self.write(
            &[command_high, command_low, data_high, data_low, crc],
            command.execution_time_us(),
        )
    }

    /// Sends `command`, waits until it has finished, then reads the `N` words it answers with.
    pub(crate) fn query<const N: usize>(
        &mut self,
        command: Command,
    ) -> Result<[u16; N], Error<I2C::Error>> {
        self.command(command)?;
        self.read_words()
    }

    /// Reads `N` CRC-protected words, rejecting the whole read if any checksum fails.
    pub(crate) fn read_words<const N: usize>(&mut self) -> Result<[u16; N], Error<I2C::Error>> {
        const { assert!(N <= MAX_WORDS, "no command returns this many words") };
        let mut buffer = [0; MAX_WORDS * Word::WIRE_LEN];
        let wire = &mut buffer[..N * Word::WIRE_LEN];
        self.i2c
            .read(self.address.as_u8(), wire)
            .map_err(Error::I2c)?;

        let mut words = [0; N];
        for (word, bytes) in words.iter_mut().zip(wire.chunks_exact(Word::WIRE_LEN)) {
            *word = Word::decode([bytes[0], bytes[1], bytes[2]])?.value();
        }
        Ok(words)
    }

    /// Resets every device on the bus that honours the I2C general call, not only this sensor.
    pub(crate) fn general_call_reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(GENERAL_CALL_ADDRESS, &[GENERAL_CALL_RESET])
            .map_err(Error::I2c)?;
        self.delay.delay_us(Timing::RESET_US);
        Ok(())
    }

    /// Pulses the sensor's nRESET pin low, then waits for it to come back.
    pub(crate) fn hard_reset<P: OutputPin>(&mut self, reset_pin: &mut P) -> Result<(), P::Error> {
        reset_pin.set_low()?;
        self.delay.delay_ns(Timing::RESET_PULSE_NS);
        reset_pin.set_high()?;
        self.delay.delay_us(Timing::RESET_US);
        Ok(())
    }

    fn write(&mut self, bytes: &[u8], wait_us: u32) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(self.address.as_u8(), bytes)
            .map_err(Error::I2c)?;
        self.delay.delay_us(wait_us.max(Timing::COMMAND_SPACING_US));
        Ok(())
    }
}
