# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-11-22

First release as `sht-sensor`, forked from [`sht3x`](https://github.com/miek/sht3x-rs).

### Added

- Periodic measurement at 0.5, 1, 2, 4 and 10 Hz, and accelerated response time (ART) mode, with `fetch` returning `None` when no new result is ready.
- The on-chip heater (`enable_heater`, `disable_heater`).
- Alert mode: reading and writing all four alert limits.
- Serial number readout.
- Hard reset through the nRESET pin, and I2C general call reset.
- `Temperature` and `Humidity` types with float and exact integer conversions in both directions (Celsius, Fahrenheit, %RH).
- Named accessors for every status register flag.
- Typestate measurement modes, so periodic-mode commands and single-shot commands cannot be mixed; failed mode changes hand the driver back.
- `core::error::Error` and `Display` for the error types.
- Optional `defmt` support.
- Integration tests against `embedded-hal-mock`, and Linux examples.

### Changed

- Ported to `embedded-hal` 1.0 (`I2c`, `DelayNs`).
- The driver owns its delay provider instead of borrowing one per call.
- Measurement waits use the datasheet's maximum durations (4.5 / 6.5 / 15.5 ms).

### Removed

- The `bitflags` dependency.
