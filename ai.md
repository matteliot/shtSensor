# Rebuilding `sht-sensor` from scratch

This file is a prompt. Give everything below the line to an AI coding assistant (or read it as a spec) to reimplement this crate from an empty directory. It describes what to build, the exact protocol details, the structure, and how to prove it works. The result should match this repository's behaviour and public API.

---

## Task

Write a Rust library crate named `sht-sensor` (library name `sht_sensor`): a platform-agnostic, `no_std`, allocation-free driver for the Sensirion SHT3x-DIS temperature and humidity sensors (SHT30, SHT31, SHT35) over I2C, built on `embedded-hal` 1.0 (`embedded_hal::i2c::I2c`, `embedded_hal::delay::DelayNs`, `embedded_hal::digital::OutputPin`).

The code must be show-quality: idiomatic, small single-purpose types, no duplicated logic, full rustdoc on every public item, and clean under `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`. It must be dual licensed `MIT OR Apache-2.0`.

### Crate setup

- Edition 2021, `rust-version = "1.81"` (needed for `core::error::Error`).
- Dependencies: `embedded-hal = "1.0.0"`; optional `defmt = "1.0"` behind a `defmt` feature that adds `#[cfg_attr(feature = "defmt", derive(defmt::Format))]` to every public data type.
- Dev-dependencies: `embedded-hal-mock = { version = "0.11", default-features = false, features = ["eh1"] }`. The example projects carry their own `linux-embedded-hal` dependency, so the library has none.
- `lib.rs` has `#![no_std]`, `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, `#![warn(rust_2018_idioms, missing_debug_implementations)]`.

### Protocol facts (from the SHT3x-DIS datasheet and Sensirion application notes)

**Addresses.** 7-bit `0x44` when the ADDR pin is low (default) and `0x45` when high.

**Commands.** Every command is a 16-bit code sent big-endian:

| Command | Code |
| --- | --- |
| Single shot, clock stretching on: high / medium / low repeatability | `0x2C06` / `0x2C0D` / `0x2C10` |
| Single shot, clock stretching off: high / medium / low | `0x2400` / `0x240B` / `0x2416` |
| Periodic 0.5 Hz: high / medium / low | `0x2032` / `0x2024` / `0x202F` |
| Periodic 1 Hz | `0x2130` / `0x2126` / `0x212D` |
| Periodic 2 Hz | `0x2236` / `0x2220` / `0x222B` |
| Periodic 4 Hz | `0x2334` / `0x2322` / `0x2329` |
| Periodic 10 Hz | `0x2737` / `0x2721` / `0x272A` |
| Periodic with accelerated response time (ART, 4 Hz) | `0x2B32` |
| Fetch data (periodic mode) | `0xE000` |
| Break (stop periodic mode) | `0x3093` |
| Soft reset | `0x30A2` |
| Heater enable / disable | `0x306D` / `0x3066` |
| Read status register | `0xF32D` |
| Clear status register | `0x3041` |
| Read serial number (from Sensirion's reference driver, not the datasheet tables) | `0x3780` |
| Read alert limit: high set / high clear / low clear / low set | `0xE11F` / `0xE114` / `0xE109` / `0xE102` |
| Write alert limit: high set / high clear / low clear / low set | `0x611D` / `0x6116` / `0x610B` / `0x6100` |

**General call reset.** Write the single byte `0x06` to I2C address `0x00`. It resets every device on the bus that supports it.

**Data words.** Every 16-bit word the sensor sends, and every data word written to it, is two big-endian bytes followed by a CRC-8 byte. CRC-8: polynomial `0x31`, initial value `0xFF`, no reflection, no final XOR. Test vector: CRC(`0xBE 0xEF`) = `0x92`; CRC(`0x00 0x00`) = `0x81`.

**Responses.**
- Measurement (single shot or fetch): 6 bytes = temperature word, then humidity word.
- Status: 1 word. Serial number: 2 words, most significant first, forming a `u32`.
- Alert limit: 1 word.
- An alert-limit write is the 2 command bytes followed by one data word with its CRC (5 bytes in one write).

**Conversions** (raw is the 16-bit word):
- °C = −45 + 175 × raw / 65535; °F = −49 + 315 × raw / 65535; %RH = 100 × raw / 65535.

**Status register bits.** 15 alert pending; 13 heater on; 11 humidity tracking alert; 10 temperature tracking alert; 4 system reset detected since last clear; 1 last command not processed; 0 last write's checksum failed.

**Alert limit packing.** One word holds both values: bits 15–9 are the top 7 bits of the raw humidity, bits 8–0 are the top 9 bits of the raw temperature. Encode: `(rh_raw & 0xFE00) | (t_raw >> 7)`. Decode: `rh_raw = word & 0xFE00`, `t_raw = (word & 0x01FF) << 7`.

**Timing** (2.4–5.5 V supply):
- Minimum 1 ms (1000 µs) after any write before the bus is used for the sensor again.
- Max measurement duration without clock stretching: high 15.5 ms, medium 6.5 ms, low 4.5 ms. Wait that long after a single-shot command before reading. With clock stretching, read after only the 1 ms spacing.
- Soft reset, general call reset and nRESET: wait 1.5 ms afterwards.
- nRESET pin: drive low for at least 1 µs, then high.

**Periodic "no data".** If you fetch before a new periodic result exists, the sensor NACKs the read. When the I2C error's `kind()` is `ErrorKind::NoAcknowledge(_)`, report it as `Ok(None)`.

### Structure: one concept per module and type

Create these modules (all private, with selected types re-exported from `lib.rs`):

| Module | Item | Visibility | Responsibility |
| --- | --- | --- | --- |
| `crc` | `struct Crc8` | crate | `const fn checksum(data: [u8; 2]) -> u8`. |
| `word` | `struct Word(u16)`, `struct CrcMismatch` | crate | `WIRE_LEN = 3`, `const fn decode([u8; 3]) -> Result<Word, CrcMismatch>`, `const fn encode(self) -> [u8; 3]`, `value()`. |
| `scale` | `struct LinearScale { offset_milli: i32, span_milli: i32 }` | crate | `milli_from_raw` (exact integer maths via i64), `float_from_raw`, `raw_from_milli` (rounded, clamped), `raw_from_float` (rounded, clamped, NaN → 0). Shared by temperature and humidity so the maths exists once. |
| `temperature` | `struct Temperature { raw: u16 }` | pub | `from_raw`, `raw`, `celsius`, `fahrenheit`, `millidegrees_celsius`, `millidegrees_fahrenheit`, `from_celsius`, `from_millidegrees_celsius`. |
| `humidity` | `struct Humidity { raw: u16 }` | pub | `from_raw`, `raw`, `percent`, `millipercent` (u32), `from_percent`, `from_millipercent`. |
| `measurement` | `struct Measurement { pub temperature, pub humidity }` | pub | Built from the two response words. |
| `status` | `struct Status { bits: u16 }` | pub | `from_bits`, `bits`, and one `bool` accessor per flag: `alert_pending`, `heater_on`, `humidity_alert`, `temperature_alert`, `reset_detected`, `last_command_failed`, `last_write_checksum_failed`. |
| `serial_number` | `struct SerialNumber(u32)` | pub | `value()`; `Display` as `0x%08x`. |
| `alert` | `enum AlertThreshold { HighSet, HighClear, LowClear, LowSet }`, `struct AlertLimit { pub humidity, pub temperature }` | pub | `AlertLimit::new`, crate-private `to_word` / `from_word`. |
| `address` | `enum Address { Low (default), High }` | pub | `as_u8()`, `From<Address> for u8`. |
| `repeatability` | `enum Repeatability { High (default), Medium, Low }` | pub | `max_measurement_duration_us()`. |
| `clock_stretching` | `enum ClockStretching { Enabled, Disabled (default) }` | pub | |
| `rate` | `enum Rate { HalfHz, Hz1, Hz2, Hz4, Hz10 }` | pub | `period_ms()`. |
| `command` | `enum Command` | crate | One variant per command (parameterised by repeatability, clock stretching, rate and alert threshold). `const fn code()`, `const fn to_bytes()`, and `const fn execution_time_us()`: the measurement duration for a single shot without clock stretching, 1500 for soft reset, otherwise 0. Commands own their timing so callers never pass waits around. |
| `timing` | `struct Timing` | crate | Constants: `COMMAND_SPACING_US = 1000`, `RESET_US = 1500`, `RESET_PULSE_NS = 1000`. |
| `bus` | `struct Bus<I2C, D> { i2c, delay, address }` | crate | The only code that touches I2C: `command(Command)` (write, then wait `max(execution_time, spacing)`), `command_with_word(Command, u16)`, `query::<N>(Command) -> [u16; N]`, `read_words::<N>()` (one read of N×3 bytes into a fixed buffer sized for the maximum of 2 words, checked with an inline `const { assert!(..) }`, every word CRC-verified), `general_call_reset()`, `hard_reset(&mut impl OutputPin)`, `release()`. |
| `mode` | sealed `trait Mode`, `struct SingleShot`, `struct Periodic` | pub | Typestate markers. |
| `error` | `enum Error<E> { I2c(E), Crc }`, `struct ModeChangeError<T, E> { pub driver: T, pub error: E }` | pub | `From<CrcMismatch>`; `Display` and `core::error::Error` for both (bounded on `E: Debug`); `ModeChangeError`'s `Debug` prints only the error. |
| `driver` | `struct Sht3x<I2C, D, M: Mode = SingleShot> { bus, mode: PhantomData<M> }` | pub | `address()`, `release() -> (I2C, D)`, crate-private `change_mode::<N>(self, Command)` returning the driver in mode `N`, or a `ModeChangeError` holding the unchanged driver. A crate-private alias `ModeChange<I2C, D, From, To, E>` names that result type. |
| `single_shot` | `impl Sht3x<I2C, D, SingleShot>` | pub API | `new(i2c, delay, address)` (no bus traffic), `measure(Repeatability)`, `measure_with(Repeatability, ClockStretching)`, `enable_heater`, `disable_heater`, `status`, `clear_status`, `serial_number`, `alert_limit(AlertThreshold)`, `set_alert_limit(AlertThreshold, AlertLimit)`, `soft_reset(&mut self)`, `hard_reset(&mut self, &mut P)`, `general_call_reset`, `start_periodic(self, Rate, Repeatability)`, `start_art(self)`. |
| `periodic` | `impl Sht3x<I2C, D, Periodic>` | pub API | `fetch() -> Result<Option<Measurement>, Error<E>>`, `stop(self)`, `soft_reset(self)`, `hard_reset(self, &mut P)`; the last three return the driver in `SingleShot` mode. |

Rules that keep it modular:
- All I2C traffic goes through `Bus`, and all data words through `Word`, so no operation can skip the CRC or the command spacing.
- Anything two types need (conversion maths, the mode-change result) lives in one shared type or alias.
- Use `const fn` wherever possible; add `#[must_use]` to pure accessors.
- Where numeric casts are deliberate (in `LinearScale` and the clamp in `Humidity::from_millipercent`), allow the specific pedantic cast lints with a comment explaining why each cast is safe.

### Tests

- Unit tests next to each module: the CRC vectors; word round-trip and corruption of either data or checksum; scale range ends, float/integer agreement, inverse round-trips within 1 raw step, clamping (including NaN); temperature and humidity conversions; every status flag set and clear; serial number joining and display; alert-limit packing and round-trip; command codes and execution times.
- Integration tests in `tests/` using `embedded-hal-mock` (`eh1::i2c::Mock`, `eh1::delay::{NoopDelay, CheckedDelay}`, `eh1::digital::Mock`). Put shared helpers in `tests/common/mod.rs` (with `#![allow(dead_code)]`), including an **independent** CRC implementation so the tests do not trust the code under test. Cover: measurement bytes and the exact wait for each repeatability; clock stretching waits only the spacing; CRC error; NACK on a missing sensor; heater codes; status read and clear; serial number; reading every alert threshold and writing one with its CRC; soft reset and general call reset waits; `release` returns a working bus; periodic start/fetch/stop then single-shot again; ART; fetch NACK → `None`; other fetch errors pass through; a failed start hands back a usable single-shot driver; soft and hard reset leave periodic mode; the hard reset pin sequence and delays.
- A doctest in the crate docs showing a measurement against a mock bus.

### Examples

**Hello world.** `examples/hello-sensor/` is a standalone Cargo project the smallest useful program and the one to copy when starting out. Its `Cargo.toml` is a package `hello-sensor` with `publish = false` and three dependencies: `sht-sensor = { path = "../.." }`, `linux-embedded-hal = { version = "0.5", default-features = false, features = ["i2c"] }` and `chrono = { version = "0.4", default-features = false, features = ["clock"] }`. Its `src/main.rs` is about 20 lines: `fn main() -> Result<(), Box<dyn Error>>` opens `I2cdev::new("/dev/i2c-1")`, builds `Sht3x::new(i2c, Delay, Address::Low)`, then loops forever: `measure(Repeatability::High)?`, print one line `"{local time %Y-%m-%d %H:%M:%S}  {°C:6.2} °C  {%RH:6.2} %RH"` to stdout, sleep one second. A failed read ends the program with the error. Make the root `.gitignore` ignore `target/` at any depth so the nested project's build output is ignored too.

**More projects.** `examples/rpi/` (serial number, status, then a reading every second) and `examples/periodic/` (a heater pulse showing its effect, then ten 1 Hz periodic readings). Each is a standalone Cargo project like `hello-sensor`, with `publish = false`, path and `linux-embedded-hal` dependencies, and `src/main.rs` using `linux_embedded_hal::{I2cdev, Delay}` on `/dev/i2c-1`.

### Documentation

A README covering: supported sensors, installation, quick start, an Examples section (the hello-sensor project's code, how to run it on the Pi, sample output, and the `rpi` and `periodic` projects), every capability with a code snippet (address, single shot, repeatability table, clock stretching, periodic, ART, heater and its caveats, status flags, alert mode and its reduced precision, serial number, the three resets), unit conversions, errors and `ModeChangeError`, platform notes (Raspberry Pi clock-stretching problems; Linux drivers that report NACK as a generic I/O error; hot-plugging; power-up time), what is not supported (SCL interface reset, low-supply timings, async), the module/type map, development commands, references and the dual license. Also a `CHANGELOG.md`, both license files, and a GitHub Actions workflow running fmt, pedantic clippy, tests, a `thumbv7em-none-eabihf` build with and without `defmt`, rustdoc with `-D warnings`, and an MSRV (1.81) check.

### Done when

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo test
cargo build --target thumbv7em-none-eabihf --features defmt
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
for e in hello-sensor rpi periodic; do (cd examples/$e && cargo clippy --target aarch64-unknown-linux-gnu -- -D warnings -W clippy::pedantic); done
```

all succeed.
