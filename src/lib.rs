//! A platform-agnostic, `no_std` driver for the Sensirion SHT3x-DIS family of digital temperature and humidity sensors
//! (SHT30, SHT31 and SHT35), built on the [`embedded-hal`] 1.0 traits.
//!
//! Every command the sensor offers is supported:
//!
//! - single-shot measurements, with or without clock stretching, at three repeatability levels
//! - periodic measurements at 0.5–10 Hz, and accelerated response time (ART) mode
//! - the on-chip heater
//! - the status register, read and clear
//! - alert mode: reading and writing the four alert limits that drive the ALERT pin
//! - the serial number
//! - soft reset, nRESET pin (hard) reset and I2C general call reset
//!
//! Every word read from the sensor is CRC-checked, and every word written to it carries a CRC. The measurement mode is part of
//! the driver's type, so commands that are invalid while measuring periodically cannot be called then.
//!
//! # Example
//!
//! ```
//! # use embedded_hal_mock::eh1::{delay::NoopDelay, i2c::{Mock, Transaction}};
//! # let i2c = Mock::new(&[
//! #     Transaction::write(0x44, vec![0x24, 0x00]),
//! #     Transaction::read(0x44, vec![0x66, 0x66, 0x93, 0x80, 0x00, 0xA2]),
//! # ]);
//! # let mut done = i2c.clone();
//! use sht_sensor::{Address, Repeatability, Sht3x};
//!
//! let mut sensor = Sht3x::new(i2c, NoopDelay::new(), Address::Low);
//! let reading = sensor.measure(Repeatability::High)?;
//! println!("{:.1} °C, {:.1} %RH", reading.temperature.celsius(), reading.humidity.percent());
//! # assert!((reading.temperature.celsius() - 25.0).abs() < 0.01);
//! # done.done();
//! # Ok::<(), sht_sensor::Error<embedded_hal::i2c::ErrorKind>>(())
//! ```
//!
//! # Features
//!
//! - `defmt`: derive [`defmt::Format`](https://docs.rs/defmt) on the public data types.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/1

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(rust_2018_idioms, missing_debug_implementations)]

mod address;
mod alert;
mod bus;
mod clock_stretching;
mod command;
mod crc;
mod driver;
mod error;
mod humidity;
mod measurement;
mod mode;
mod periodic;
mod rate;
mod repeatability;
mod scale;
mod serial_number;
mod single_shot;
mod status;
mod temperature;
mod timing;
mod word;

pub use address::Address;
pub use alert::{AlertLimit, AlertThreshold};
pub use clock_stretching::ClockStretching;
pub use driver::Sht3x;
pub use error::{Error, ModeChangeError};
pub use humidity::Humidity;
pub use measurement::Measurement;
pub use mode::{Mode, Periodic, SingleShot};
pub use rate::Rate;
pub use repeatability::Repeatability;
pub use serial_number::SerialNumber;
pub use status::Status;
pub use temperature::Temperature;
