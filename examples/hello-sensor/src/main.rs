//! Reads the temperature and humidity sensor once a second and prints a timestamp, the temperature and the humidity.

use std::{error::Error, thread, time::Duration};

use chrono::Local;
use linux_embedded_hal::{Delay, I2cdev};
use sht_sensor::{Address, Repeatability, Sht3x};

fn main() -> Result<(), Box<dyn Error>> {
    let i2c = I2cdev::new("/dev/i2c-1")?;
    let mut sensor = Sht3x::new(i2c, Delay, Address::Low);

    loop {
        let reading = sensor.measure(Repeatability::High)?;
        println!(
            "{}  {:6.2} °C  {:6.2} %RH",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            reading.temperature.celsius(),
            reading.humidity.percent(),
        );
        thread::sleep(Duration::from_secs(1));
    }
}
