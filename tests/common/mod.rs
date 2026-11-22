//! Helpers shared by the integration tests: an independent CRC implementation (so the tests don't trust the code under test) and
//! builders for the mocked bus. Each test binary uses only some of them.
#![allow(dead_code)]

use embedded_hal_mock::eh1::delay::{CheckedDelay, NoopDelay, Transaction as DelayTransaction};
use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use sht_sensor::{Address, Sht3x};

pub const ADDRESS: u8 = 0x44;
pub const COMMAND_SPACING_US: u32 = 1_000;

/// Sensirion CRC-8, written out bit by bit from the datasheet's definition.
pub fn crc8(data: [u8; 2]) -> u8 {
    data.iter().fold(0xFF_u8, |crc, &byte| {
        (0..8).fold(crc ^ byte, |crc, _| {
            if crc & 0x80 != 0 {
                (crc << 1) ^ 0x31
            } else {
                crc << 1
            }
        })
    })
}

/// The wire bytes for `words`, each followed by its CRC.
pub fn frame(words: &[u16]) -> Vec<u8> {
    words
        .iter()
        .flat_map(|word| {
            let [high, low] = word.to_be_bytes();
            [high, low, crc8([high, low])]
        })
        .collect()
}

pub fn write(command: u16) -> I2cTransaction {
    I2cTransaction::write(ADDRESS, command.to_be_bytes().to_vec())
}

pub fn read(words: &[u16]) -> I2cTransaction {
    I2cTransaction::read(ADDRESS, frame(words))
}

/// A driver over a mock bus that expects exactly `transactions`, with delays ignored. Returns a handle to check the bus afterwards.
pub fn sensor(transactions: &[I2cTransaction]) -> (Sht3x<I2cMock, NoopDelay>, I2cMock) {
    let i2c = I2cMock::new(transactions);
    (Sht3x::new(i2c.clone(), NoopDelay::new(), Address::Low), i2c)
}

/// A driver over a mock bus and a mock delay that also checks every wait, in microseconds.
pub fn timed_sensor(
    transactions: &[I2cTransaction],
    waits_us: &[u32],
) -> (Sht3x<I2cMock, CheckedDelay>, I2cMock, CheckedDelay) {
    let i2c = I2cMock::new(transactions);
    let delay = CheckedDelay::new(
        &waits_us
            .iter()
            .map(|&us| DelayTransaction::delay_us(us))
            .collect::<Vec<_>>(),
    );
    (
        Sht3x::new(i2c.clone(), delay.clone(), Address::Low),
        i2c,
        delay,
    )
}
