// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Error types returned by the driver.

/// Errors that can occur while talking to the ATM90E32.
///
/// The generic parameter `E` is the underlying SPI transport error type
/// (usually `<SPI as embedded_hal_async::spi::ErrorType>::Error`).
///
/// This enum is marked `#[non_exhaustive]` so future variants can be added
/// without a semver-breaking change.
#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// Underlying SPI transport error.
    Spi(E),
    /// Probe did not find a responsive chip. The SysStatus0 register read
    /// back as `0x0000` or `0xFFFF`, indicating either a missing device or
    /// a floating SPI bus.
    NotPresent,
    /// The initialization sequence failed at the given stage.
    InitFailed(InitStage),
}

/// The stage of [`Atm90e32::init`](crate::Atm90e32::init) at which a failure
/// occurred.
///
/// Useful for diagnostics and logging. Marked `#[non_exhaustive]` to allow
/// future variants.
///
/// [`Atm90e32::init`]: crate::Atm90e32::init
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InitStage {
    /// Soft reset register write (`REG_SOFTRESET`).
    SoftReset,
    /// Unlocking the configuration registers (`REG_CFGREGACCEN = 0x55AA`).
    UnlockConfig,
    /// Enabling metering (`REG_METEREN = 0x0001`).
    EnableMeter,
    /// Writing the sag/peak detector configuration (`REG_SAGPEAKDETCFG`).
    WriteSagPeak,
    /// Writing the PL constant pair (`REG_PLCONSTH`, `REG_PLCONSTL`).
    WritePlConst,
    /// Writing the zero-crossing configuration (`REG_ZXCONFIG`).
    WriteZxConfig,
    /// Writing the metering mode 0 register (`REG_MMODE0`).
    WriteMMode0,
    /// Writing the metering mode 1 register (`REG_MMODE1`) — PGA gain.
    WriteMMode1,
    /// Writing the frequency high/low threshold pair.
    WriteFreqThresholds,
    /// Writing the startup thresholds (P/Q/S start and per-phase thresholds).
    WriteStartupThresholds,
    /// Writing the per-phase voltage gain registers.
    WriteVoltageGains,
    /// Writing the per-phase current gain registers.
    WriteCurrentGains,
    /// Locking the configuration registers (`REG_CFGREGACCEN = 0x0000`).
    LockConfig,
}
