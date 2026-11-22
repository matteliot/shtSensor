# sht-sensor

A platform-agnostic, `no_std` Rust driver for the Sensirion **SHT3x-DIS** digital temperature and humidity sensors — **SHT30**, **SHT31** and **SHT35** — built on [`embedded-hal`] 1.0.

It covers every command the sensor offers, checks the CRC of every word it reads, tracks the measurement mode in the type system, and has no dependencies beyond `embedded-hal` (plus optional `defmt`).

This crate started as a fork of [`sht3x`](https://github.com/miek/sht3x-rs) by Mike Walters and contributors; its git history is preserved.

## Contents

- [Supported sensors](#supported-sensors)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Examples](#examples)
- [Everything the sensor can do](#everything-the-sensor-can-do)
- [Units and conversions](#units-and-conversions)
- [Errors](#errors)
- [Platform notes](#platform-notes)
- [Design](#design)
- [Development](#development)
- [License](#license)

## Supported sensors

SHT30-DIS, SHT31-DIS and SHT35-DIS. They differ only in accuracy, and share one command set and one datasheet. The analog variants (SHT3x-ARP) are not I2C devices and are not supported.

## Installation

```toml
[dependencies]
sht-sensor = { git = "https://github.com/matteliot/shtSensor" }
```

Requires Rust 1.81 or newer. The crate is `no_std` and never allocates.

### Features

| Feature | Effect |
| --- | --- |
| `defmt` | Derives `defmt::Format` on the public data types for logging on microcontrollers. |

## Quick start

```rust,ignore
use linux_embedded_hal::{Delay, I2cdev};
use sht_sensor::{Address, Repeatability, Sht3x};

let i2c = I2cdev::new("/dev/i2c-1")?;
let mut sensor = Sht3x::new(i2c, Delay, Address::Low);

let reading = sensor.measure(Repeatability::High)?;
println!("{:.2} °C, {:.2} %RH", reading.temperature.celsius(), reading.humidity.percent());
```

The driver takes ownership of the I2C bus and a delay provider (`release()` gives them back). To share a bus with other devices, pass a sharing wrapper such as those in [`embedded-hal-bus`](https://docs.rs/embedded-hal-bus).

## Examples

Everything in [`examples/`](examples) runs on a Linux board such as a Raspberry Pi with the sensor on `/dev/i2c-1`.

### Hello, sensor

[`examples/hello-sensor`](examples/hello-sensor) is the smallest useful program: a standalone Cargo project that reads the sensor once a second and prints a timestamp, the temperature and the humidity. Copy the folder to start your own project (change the `sht-sensor` path dependency to the git URL from [Installation](#installation)).

```rust,ignore
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
```

Run it on the Pi:

```text
cd examples/hello-sensor
cargo run --release
```

The output looks like this:

```text
2026-11-22 13:10:01   21.48 °C   44.12 %RH
2026-11-22 13:10:02   21.47 °C   44.15 %RH
2026-11-22 13:10:03   21.47 °C   44.13 %RH
```

It stops with an error if a read fails (for example if the sensor is unplugged); see [Platform notes](#platform-notes) for handling that.

### More examples

```text
cargo run --example raspberry_pi   # single-shot readings, serial number, status
cargo run --example periodic       # heater pulse, then 1 Hz periodic readings
```

## Everything the sensor can do

### I2C address

The ADDR pin picks the address: `Address::Low` (`0x44`, ADDR to GND — most breakout boards) or `Address::High` (`0x45`, ADDR to VDD). Two sensors can share a bus, one at each address.

### Single-shot measurement

```rust,ignore
let reading = sensor.measure(Repeatability::High)?;
```

Repeatability trades noise against time:

| Repeatability | Max. duration | Use when |
| --- | --- | --- |
| `High` (default) | 15.5 ms | You want the steadiest readings. |
| `Medium` | 6.5 ms | |
| `Low` | 4.5 ms | Speed or power matter more than noise. |

### Clock stretching

By default the driver sends the command, waits the measurement's maximum duration, then reads. With clock stretching the sensor instead holds the I2C clock low until the result is ready, so the read finishes as soon as possible:

```rust,ignore
use sht_sensor::ClockStretching;
let reading = sensor.measure_with(Repeatability::High, ClockStretching::Enabled)?;
```

Only enable it if your I2C controller handles clock stretching correctly (see [Platform notes](#platform-notes)).

### Periodic measurement

The sensor can measure by itself at a fixed rate; the driver fetches the newest result. Starting periodic mode consumes the single-shot driver and returns a periodic one, so single-shot-only commands cannot be sent by mistake:

```rust,ignore
use sht_sensor::Rate;

let mut sensor = sensor.start_periodic(Rate::Hz1, Repeatability::High).map_err(|failure| failure.error)?;
if let Some(reading) = sensor.fetch()? {
    // a new reading since the last fetch
}
let sensor = sensor.stop().map_err(|failure| failure.error)?; // back to single-shot mode
```

Rates: `HalfHz` (every 2 s), `Hz1`, `Hz2`, `Hz4`, `Hz10`. `Rate::period_ms()` gives the interval to poll at. Measuring fast with high repeatability can warm the sensor slightly.

`fetch()` returns `Ok(None)` when there is no new reading since the last fetch.

### Accelerated response time (ART)

ART is a periodic mode at 4 Hz in which the sensor's humidity reading reacts faster to changes:

```rust,ignore
let mut sensor = sensor.start_art().map_err(|failure| failure.error)?;
```

Use `fetch()` and `stop()` exactly as for periodic mode.

### Heater

The sensor has a small on-chip heater:

```rust,ignore
sensor.enable_heater()?;
// ...
sensor.disable_heater()?;
let on = sensor.status()?.heater_on();
```

While it is on, temperature reads high and humidity reads low, by a few degrees depending on supply voltage and surroundings. It is meant for:

- **Plausibility checks**: a working sensor's temperature rises and humidity falls when the heater is on.
- **Drying out** after condensation or long exposure to very high humidity.

It is not regulated and has no timer, so switch it off yourself and let the sensor cool before trusting readings again.

### Status register

```rust,ignore
let status = sensor.status()?;
status.alert_pending();             // bit 15: an alert is active
status.heater_on();                 // bit 13
status.humidity_alert();            // bit 11: humidity tracking alert
status.temperature_alert();         // bit 10: temperature tracking alert
status.reset_detected();            // bit 4: reset since the last clear
status.last_command_failed();       // bit 1: last command invalid or corrupted
status.last_write_checksum_failed(); // bit 0: last data write had a bad checksum
sensor.clear_status()?;             // clears the alert and reset flags
```

`reset_detected()` is a cheap way to spot brown-outs: clear the register at startup and check it later.

### Alert mode

In periodic mode the sensor compares each measurement against four programmable limits and drives its ALERT pin (and the status flags) when humidity or temperature goes out of range. Each limit holds a humidity and a temperature:

| Threshold | Meaning |
| --- | --- |
| `HighSet` | Raise the alert above this. |
| `HighClear` | Release a high alert below this. |
| `LowClear` | Release a low alert above this. |
| `LowSet` | Raise the alert below this. |

The gap between each set/clear pair is the hysteresis.

```rust,ignore
use sht_sensor::{AlertLimit, AlertThreshold, Humidity, Temperature};

let high = AlertLimit::new(Humidity::from_percent(70.0), Temperature::from_celsius(35.0));
sensor.set_alert_limit(AlertThreshold::HighSet, high)?;
let current = sensor.alert_limit(AlertThreshold::HighClear)?;
```

Limits are stored at reduced precision (the top 7 bits of humidity and 9 bits of temperature, about 0.8 %RH and 0.34 °C steps), so a limit read back is rounded down. They are volatile: any reset restores the sensor's power-up defaults. If the sensor rejects a write's checksum it ignores the write and sets `last_write_checksum_failed()`.

### Serial number

```rust,ignore
let serial = sensor.serial_number()?; // displays as e.g. 0x0b7c3a2e
```

This uses command `0x3780`, which is not in the datasheet's command tables but is the one Sensirion's own driver uses for the SHT3x.

### Resets

| Method | What it does |
| --- | --- |
| `soft_reset()` | Command `0x30A2`: reloads calibration and returns every setting (mode, heater, alert limits) to its default. |
| `hard_reset(&mut pin)` | Pulses the nRESET pin low. Works even when the sensor no longer answers on the bus. |
| `general_call_reset()` | I2C general call reset. **Resets every device on the bus that honours the general call**, not only this sensor. |

In periodic mode `soft_reset` and `hard_reset` consume the driver and return it in single-shot mode, as the sensor itself returns to single-shot mode.

### Not supported

- The SCL-toggling **interface reset** (datasheet section 4.9): it needs direct control of the bus lines, which `embedded-hal`'s I2C trait does not give.
- **Low-supply timings**: the waits assume a 2.4–5.5 V supply. Below that the datasheet allows longer measurement times.
- **Async**: the driver is blocking. `embedded-hal-async` support would be a natural addition.

## Units and conversions

`Temperature` and `Humidity` keep the sensor's raw 16-bit value, so no precision is lost until you pick a unit:

| Method | Returns |
| --- | --- |
| `Temperature::celsius()` / `fahrenheit()` | `f32` degrees |
| `Temperature::millidegrees_celsius()` / `millidegrees_fahrenheit()` | `i32`, integer maths only |
| `Humidity::percent()` | `f32` %RH |
| `Humidity::millipercent()` | `u32`, integer maths only |
| `Temperature::from_celsius()`, `Humidity::from_percent()` and the integer variants | the nearest raw value, clamped to the sensor's range |

The integer methods are exact and suit microcontrollers without an FPU. Ranges are −45 °C to 130 °C and 0 % to 100 %RH.

## Errors

Every fallible method returns `Result<_, sht_sensor::Error<E>>`, where `E` is your I2C bus's error type:

- `Error::I2c(e)`: the transfer failed. A sensor that is unplugged or unpowered shows up here as a NACK.
- `Error::Crc`: data arrived with a bad checksum and was discarded.

`Error` implements `core::error::Error` and `Display`.

Mode changes (`start_periodic`, `start_art`, `stop`, and the resets in periodic mode) return a `ModeChangeError { driver, error }` on failure. It hands the driver back in its original mode, so you can retry, reset or release it rather than lose it. Use `.map_err(|failure| failure.error)` when you just want the error.

## Platform notes

**Raspberry Pi.** Enable I2C with `raspi-config` (Interface Options > I2C); the bus is `/dev/i2c-1` on GPIO2 (SDA) and GPIO3 (SCL). The Pi's I2C controller has a long-standing problem with clock stretching, so leave `ClockStretching::Disabled` (the default) there.

**"No new data" on Linux.** The sensor answers an early `fetch()` with a NACK. `fetch()` turns that into `Ok(None)` when the bus reports it as `ErrorKind::NoAcknowledge`. Some Linux I2C drivers, including the Pi's, report a NACK as a generic I/O error, so there it appears as `Err(Error::I2c(_))` instead. Polling no faster than `Rate::period_ms()` avoids it.

**Hot-plugging.** A sensor that is missing simply makes every command fail with `Error::I2c`, and one that appears later starts answering, so retrying on error is all that is needed to detect plugging and unplugging. Connecting a sensor to a powered bus can still glitch other devices on it.

**Power-up.** The sensor needs up to 1.5 ms after power-up before it accepts commands.

## Design

Each concept lives in its own module and type:

| Module | Type | Responsibility |
| --- | --- | --- |
| `crc` | `Crc8` | Sensirion CRC-8 (polynomial `0x31`, init `0xFF`). |
| `word` | `Word` | One CRC-protected 16-bit word: encode for writes, decode and verify for reads. |
| `scale` | `LinearScale` | Raw ↔ physical conversion, shared by temperature and humidity. |
| `temperature`, `humidity` | `Temperature`, `Humidity` | Units on top of `LinearScale`. |
| `measurement` | `Measurement` | A temperature/humidity pair. |
| `status` | `Status` | Status register flags. |
| `alert` | `AlertThreshold`, `AlertLimit` | Alert limits and their packed format. |
| `serial_number` | `SerialNumber` | The 32-bit serial. |
| `address`, `repeatability`, `clock_stretching`, `rate` | enums | Configuration. |
| `command` | `Command` | Every command code, and how long each keeps the sensor busy. |
| `timing` | `Timing` | Datasheet delays. |
| `bus` | `Bus` | The I2C link: send commands, read and write words, command spacing, resets. |
| `mode` | `SingleShot`, `Periodic` | Typestate markers. |
| `driver` | `Sht3x` | The driver, shared behaviour and mode changes. |
| `single_shot`, `periodic` | — | The commands available in each mode. |

All bus traffic goes through `Bus`, and all data goes through `Word`, so the CRC check and the inter-command delay cannot be skipped by any operation.

## Development

```text
cargo test                                     # unit, integration and doc tests (mocked I2C, no hardware needed)
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo build --target thumbv7em-none-eabihf     # proves no_std
cargo check --examples --target aarch64-unknown-linux-gnu
(cd examples/hello-sensor && cargo check --target aarch64-unknown-linux-gnu)
```

The integration tests in [`tests/`](tests) drive the full driver against [`embedded-hal-mock`](https://docs.rs/embedded-hal-mock), checking the exact bytes on the bus and every delay. They use their own CRC implementation, so a CRC bug in the driver cannot hide itself.

[`ai.md`](ai.md) is a prompt that describes this crate completely enough to reimplement it from scratch.

## References

- Sensirion, *Datasheet SHT3x-DIS*: <https://sensirion.com/media/documents/213E6A3B/61641DC3/Sensirion_Humidity_Sensors_SHT3x_Datasheet_digital.pdf>
- Sensirion, *Application Note: SHT3x-DIS Alert Mode*
- Sensirion's reference C driver, `embedded-sht`: <https://github.com/Sensirion/embedded-sht>

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://docs.rs/embedded-hal/1
