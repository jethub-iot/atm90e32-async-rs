# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-10

### Added

* Initial release of the `atm90e32` async `no_std` driver.
* `Atm90e32<SPI, D>` struct generic over
  `embedded_hal_async::spi::SpiDevice` and
  `embedded_hal_async::delay::DelayNs`, with no hard dependency on
  any specific async runtime.
* `Config` struct with builder-style setters covering every value
  written during `init()`: per-phase voltage and current gains,
  line frequency (50/60 Hz), PGA gain (x1/x2/x4), sag/peak detector,
  pulse constant, zero-crossing, MMode0/1, startup thresholds,
  per-phase accumulation thresholds, post-reset delay.
  `Config::default()` provides the known-good ESPHome values.
* Typed `LineFreq` and `PgaGain` enums.
* `probe()` for chip presence detection via `SysStatus0`.
* `init()` with per-step error reporting via
  `Error::InitFailed(InitStage)`.
* Bulk 3-phase readout (`read_all_phases`) producing a
  `PhaseReadings` snapshot with RMS voltage, RMS current, active
  power, reactive power, power factor, and mains frequency.
* Per-phase helpers: `read_voltage`, `read_current`,
  `read_active_power`, `read_reactive_power`, `read_power_factor`,
  `read_frequency`.
* Low-level escape hatch: `read_register`, `write_register`, plus the
  full `REG_*` constant map in the `registers` module.
* Sans-I/O `proto` module exposing the byte-level wire format and
  unit conversions as pure functions, covered by 18 host-runnable
  unit tests that need no mock SPI.
* Optional `defmt` feature that derives `defmt::Format` on public
  types.
* Compile-check example in `examples/basic.rs` plus a real
  Embassy + esp-hal integration sketch in its doc comment.
* GitHub Actions CI: rustfmt, clippy, test, doc on host;
  build+clippy on `thumbv7em-none-eabihf` and
  `riscv32imc-unknown-none-elf` (with and without the `defmt`
  feature); MSRV 1.75 build.

### Validated on hardware

* JetHome PM380 (`jxd-pm380-e1eth-powermeter` board), ESP32, Embassy
  async runtime, 3-phase 230 V / 50 Hz input.

### Not yet implemented

Energy accumulation registers, harmonic analysis, sag/swell
detection, zero-crossing interrupts, calibration assist helpers,
blocking (non-async) API variant, ATM90E36 and ATM90E26 support.
See README for the full "what's in v0.1" / "not yet" breakdown.

[Unreleased]: https://github.com/jethub-iot/atm90e32/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jethub-iot/atm90e32/releases/tag/v0.1.0
