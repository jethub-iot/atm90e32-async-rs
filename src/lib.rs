// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Async `no_std` driver for the Microchip / Atmel **ATM90E32** 3-phase
//! SPI power metering IC.
//!
//! This crate provides a lightweight, portable driver built on top of
//! [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async).
//! It has no dependency on any specific async runtime and works with
//! Embassy, RTIC2, or any other executor that implements
//! [`SpiDevice`](embedded_hal_async::spi::SpiDevice) and
//! [`DelayNs`](embedded_hal_async::delay::DelayNs).
//!
//! ## Architecture
//!
//! The crate follows a **sans-I/O** split:
//!
//! * [`proto`] — pure functions: byte-frame building, response parsing,
//!   raw → engineering unit conversions, and the post-reset init
//!   sequence expressed as data. No I/O, no async — 100% host-testable.
//! * [`driver`] — async transport: owns the SPI device and delay,
//!   invokes the `proto` helpers, maps errors, and sequences the init
//!   writes.
//!
//! ## Features
//!
//! * `defmt` *(optional)* — derives
//!   [`defmt::Format`](https://docs.rs/defmt/latest/defmt/trait.Format.html)
//!   on public types for ergonomic logging with the `defmt` ecosystem.
//!
//! ## Quick start
//!
//! ```no_run
//! # use atm90e32_async::{Atm90e32, Config, LineFreq, PgaGain};
//! # async fn demo<SPI, D>(spi: SPI, delay: D) -> Result<(), atm90e32_async::Error<SPI::Error>>
//! # where
//! #     SPI: embedded_hal_async::spi::SpiDevice,
//! #     D:   embedded_hal_async::delay::DelayNs,
//! # {
//! let mut meter = Atm90e32::new(spi, delay);
//! meter.probe().await?;
//!
//! let cfg = Config::default()
//!     .with_voltage_gain([39470, 39470, 39470])
//!     .with_current_gain([65327, 65327, 65327])
//!     .with_line_freq(LineFreq::Hz50)
//!     .with_pga_gain(PgaGain::X2);
//! meter.init(&cfg).await?;
//!
//! loop {
//!     let r = meter.read_all_phases().await?;
//!     // use r.voltage, r.current, r.power, r.reactive, r.pf, r.frequency
//!     # break;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## What's in v0.1
//!
//! The current release implements the minimum feature set used by
//! production JetHome PM380 meters: RMS voltage/current, active and
//! reactive power, power factor, and mains frequency on all three
//! phases. Not yet implemented (contributions welcome): energy
//! accumulation, harmonic analysis, sag/swell detection,
//! zero-crossing interrupts, and ATM90E36 family support.
//!
//! ## License
//!
//! Dual-licensed under either
//! [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
//! [GPL-2.0-or-later](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html)
//! at the user's option.

#![no_std]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod driver;
pub mod error;
#[doc(hidden)]
pub mod proto;
pub mod readings;
pub mod registers;
pub mod status;

pub use crate::config::{Config, LineFreq, PgaGain};
pub use crate::driver::{Atm90e32, Phase};
pub use crate::error::{Error, InitStage};
pub use crate::readings::PhaseReadings;
pub use crate::status::PhaseStatus;
