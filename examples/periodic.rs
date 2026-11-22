//! Periodic measurement at 1 Hz, with a brief heater pulse first to show its effect on the readings.
//!
//! ```text
//! cargo run --example periodic
//! ```

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use linux_embedded_hal::{Delay, I2cdev};
    use sht_sensor::{Address, Rate, Repeatability, Sht3x};
    use std::{thread, time::Duration};

    let mut sensor = Sht3x::new(I2cdev::new("/dev/i2c-1")?, Delay, Address::Low);

    let before = sensor.measure(Repeatability::High)?;
    sensor.enable_heater()?;
    thread::sleep(Duration::from_secs(5));
    let heated = sensor.measure(Repeatability::High)?;
    sensor.disable_heater()?;
    println!(
        "heater: {:.2} °C -> {:.2} °C, {:.2} %RH -> {:.2} %RH",
        before.temperature.celsius(),
        heated.temperature.celsius(),
        before.humidity.percent(),
        heated.humidity.percent()
    );

    let rate = Rate::Hz1;
    let mut sensor = sensor
        .start_periodic(rate, Repeatability::High)
        .map_err(|failure| failure.error)?;
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(rate.period_ms().into()));
        if let Some(reading) = sensor.fetch()? {
            println!(
                "{:6.2} °C  {:6.2} %RH",
                reading.temperature.celsius(),
                reading.humidity.percent()
            );
        }
    }
    sensor.stop().map_err(|failure| failure.error)?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("This example talks to /dev/i2c-1 and only runs on Linux.");
}
