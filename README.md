# atm90e32

[![CI](https://github.com/jethub-iot/atm90e32/actions/workflows/ci.yml/badge.svg)](https://github.com/jethub-iot/atm90e32/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/atm90e32.svg)](https://crates.io/crates/atm90e32)
[![Documentation](https://docs.rs/atm90e32/badge.svg)](https://docs.rs/atm90e32)
[![License: GPL-2.0-or-later OR Apache-2.0](https://img.shields.io/badge/license-GPL--2.0--or--later%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-red.svg)](#msrv)

Async `no_std` driver for the Microchip / Atmel **ATM90E32** 3-phase
SPI power metering IC.

Built on top of [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async),
with no hard dependency on any specific async runtime — works with
Embassy, RTIC2, or any executor that provides `SpiDevice` and `DelayNs`.

## Architecture

The crate is split into a **sans-I/O core** and an **async transport** layer:

* `proto` — pure functions: byte-frame building, response parsing,
  raw → engineering-unit conversions, and the post-reset init sequence
  materialised as data (`[InitStep; 22]`). 100% host-testable.
* `driver` — the `Atm90e32<SPI, D>` struct: wraps SPI reads/writes,
  drives the init sequence, issues delays through a generic `DelayNs`.

The practical effect is that the tricky parts — wire format, 32-bit
power-register assembly, signed power-factor decoding, line-frequency
dependent MMode0 bit flip — are covered by 18 host unit tests and
don't need a mock SPI bus.

## Features

What v0.1 does:

* Chip presence probe via `SysStatus0`
* Full init sequence (soft reset, unlock, enable, calibration gains,
  startup thresholds, freq thresholds, PGA, MMode0/1, lock) driven by
  a single `Config` struct
* Bulk 3-phase readout: RMS voltage, RMS current, active power,
  reactive power, power factor, mains frequency
* Per-phase reads for each of the above
* Raw register read/write escape hatch (`read_register` / `write_register`)
* Typed errors: `Error::Spi(E)`, `Error::NotPresent`,
  `Error::InitFailed(InitStage)` with a per-step breakdown
* Optional `defmt::Format` derives behind the `defmt` feature
* No heap, no global state, no hard runtime dependency

What v0.1 does **not** do (PRs welcome):

* Energy accumulation (`EPosA`/`EPosT`/…)
* Harmonic analysis registers
* Sag / swell detection and zero-crossing interrupts
* Calibration assist helpers (auto-gain)
* ATM90E36 family support (planned — the code layout anticipates it)
* ATM90E26 (8-bit addressing, out of scope)

## Quick start

```rust,no_run
use atm90e32::{Atm90e32, Config, LineFreq, PgaGain};

async fn run<SPI, D>(spi: SPI, delay: D) -> Result<(), atm90e32::Error<SPI::Error>>
where
    SPI: embedded_hal_async::spi::SpiDevice,
    D:   embedded_hal_async::delay::DelayNs,
{
    let mut meter = Atm90e32::new(spi, delay);
    meter.probe().await?;

    let cfg = Config::default()
        .with_voltage_gain([39470, 39470, 39470])
        .with_current_gain([65327, 65327, 65327])
        .with_line_freq(LineFreq::Hz50)
        .with_pga_gain(PgaGain::X2);
    meter.init(&cfg).await?;

    let r = meter.read_all_phases().await?;
    // r.voltage, r.current, r.power, r.reactive, r.pf, r.frequency
    Ok(())
}
```

For a real integration on an ESP32 + Embassy + esp-hal, see the sketch
in [`examples/basic.rs`](examples/basic.rs).

## Hardware requirements

* SPI mode 3 (CPOL=1, CPHA=1), MSB-first
* SPI clock ≤ 16 MHz (datasheet limit)
* One GPIO for chip select — exposed through the caller's
  `SpiDevice` implementation (e.g. `embedded-hal-bus::spi::ExclusiveDevice`)

The driver itself is transport-agnostic and works with any
`embedded-hal-async::spi::SpiDevice` implementation.

## Datasheet

[ATM90E32AS datasheet (Microchip)](https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-46002-SE-M90E32AS-Datasheet.pdf)

## MSRV

Minimum supported Rust version is **1.75**, required by
`embedded-hal-async` 1.0 (async-fn-in-traits). MSRV is part of the
semver contract and will only be bumped with a minor version release.

## License

Copyright (c) Viacheslav Bocharov \<v@baodeep.com\> and JetHome (r).

Dual-licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
* GNU General Public License, Version 2.0 or later
  ([LICENSE-GPL](LICENSE-GPL) or
  <https://www.gnu.org/licenses/old-licenses/gpl-2.0.html>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this project by you shall be dual-licensed
as above, without any additional terms or conditions.
