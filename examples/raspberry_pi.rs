//! Single-shot measurements on a Raspberry Pi (or any Linux board), plus the diagnostic commands.
//!
//! Wire the sensor to the Pi's I2C1 pins (SDA = GPIO2, SCL = GPIO3), enable I2C (`raspi-config` > Interface Options > I2C), then:
//!
//! ```text
//! cargo run --example raspberry_pi
//! ```

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use linux_embedded_hal::{Delay, I2cdev};
    use sht_sensor::{Address, Repeatability, Sht3x};
    use std::{thread, time::Duration};

    let mut sensor = Sht3x::new(I2cdev::new("/dev/i2c-1")?, Delay, Address::Low);

    println!("serial number: {}", sensor.serial_number()?);
    println!("status: {:?}", sensor.status()?);

    loop {
        let reading = sensor.measure(Repeatability::High)?;
        println!(
            "{:6.2} °C  {:6.2} %RH",
            reading.temperature.celsius(),
            reading.humidity.percent()
        );
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("This example talks to /dev/i2c-1 and only runs on Linux.");
}
