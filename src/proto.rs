// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Sans-I/O helpers — pure functions with no SPI, no async, no delays.
//!
//! This module is the testable core of the driver. Everything here is a
//! deterministic transformation: byte frame building, response parsing,
//! raw register → engineering unit conversion, and materializing the init
//! write sequence as a bounded array of (address, value, stage) records.
//!
//! The driver module in this crate is just a thin async transport layer on
//! top of these helpers.
//!
//! ## Stability
//!
//! This module is currently marked `#[doc(hidden)]` at the crate root so
//! unit tests in the `tests/` directory can reach it, but the specific API
//! shape is not part of the public semver contract. Depend on it at your
//! own risk.

use crate::config::{Config, LineFreq};
use crate::error::InitStage;
use crate::registers::*;

// ── Frame building and parsing ───────────────────────────────────────

/// Read-flag bit set in the first byte of a read frame.
const READ_FLAG: u8 = 0x80;

/// Build a 4-byte SPI transmit buffer for reading a 16-bit register.
///
/// Frame layout: `[0x80 | addr[9:8], addr[7:0], 0x00, 0x00]`.
/// The chip responds with the 16-bit register value in bytes 2-3.
pub fn build_read_frame(addr: u16) -> [u8; 4] {
    let addr_h = READ_FLAG | ((addr >> 8) as u8 & 0x03);
    let addr_l = (addr & 0xFF) as u8;
    [addr_h, addr_l, 0x00, 0x00]
}

/// Build a 4-byte SPI transmit buffer for writing a 16-bit register.
///
/// Frame layout: `[addr[9:8], addr[7:0], value[15:8], value[7:0]]`.
/// The read/write bit (bit 7 of byte 0) is cleared for a write.
pub fn build_write_frame(addr: u16, value: u16) -> [u8; 4] {
    let addr_h = (addr >> 8) as u8 & 0x03;
    let addr_l = (addr & 0xFF) as u8;
    [addr_h, addr_l, (value >> 8) as u8, value as u8]
}

/// Parse the 4-byte SPI receive buffer from a read transaction.
///
/// Bytes 0-1 are the echoed address, bytes 2-3 are the register value
/// in big-endian order.
pub fn parse_read_response(rx: &[u8; 4]) -> u16 {
    ((rx[2] as u16) << 8) | rx[3] as u16
}

/// Combine a 32-bit signed value from the high and low 16-bit register
/// words exposed by the ATM90E32 for active/reactive power readings.
pub fn combine_power_words(high: u16, low: u16) -> i32 {
    ((high as i32) << 16) | (low as i32)
}

// ── Engineering-unit conversions ─────────────────────────────────────

/// Scale factor to convert a raw 32-bit power word to watts (or vars).
///
/// Matches the ESPHome `atm90e32` component.
pub const POWER_SCALE: f32 = 0.00032;

/// Convert a raw voltage RMS register value to volts.
///
/// The chip reports voltage in hundredths of a volt.
pub fn voltage_raw_to_volts(raw: u16) -> f32 {
    raw as f32 / 100.0
}

/// Convert a raw current RMS register value to amps.
///
/// The chip reports current in thousandths of an amp.
pub fn current_raw_to_amps(raw: u16) -> f32 {
    raw as f32 / 1000.0
}

/// Convert a 32-bit active or reactive power register pair to watts (or vars).
pub fn power_raw_to_watts(high: u16, low: u16) -> f32 {
    combine_power_words(high, low) as f32 * POWER_SCALE
}

/// Convert a raw power-factor register value to a dimensionless factor.
///
/// The raw value is a signed 16-bit integer in thousandths (so `1000` → 1.0,
/// `-1000` → -1.0).
pub fn power_factor_raw_to_unitless(raw: u16) -> f32 {
    (raw as i16) as f32 / 1000.0
}

/// Convert a raw frequency register value to hertz.
///
/// The chip reports frequency in hundredths of a hertz.
pub fn frequency_raw_to_hz(raw: u16) -> f32 {
    raw as f32 / 100.0
}

// ── Init sequence as data ────────────────────────────────────────────

/// A single write step in the post-reset initialization sequence.
///
/// The driver iterates [`build_init_sequence`] and performs one
/// `write_register` per step. On failure the SPI error is wrapped into
/// `Error::InitFailed(step.stage)` so the caller can tell which step broke.
///
/// The soft reset is **not** part of this sequence — it is issued before
/// the sequence and followed by a delay; only then is the sequence played.
#[derive(Debug, Clone, Copy)]
pub struct InitStep {
    /// Register address to write.
    pub addr: u16,
    /// Value to write.
    pub value: u16,
    /// Diagnostic stage label for error reporting.
    pub stage: InitStage,
}

/// Number of steps in the init sequence returned by [`build_init_sequence`].
///
/// Includes everything from "unlock config" to "lock config" (the soft
/// reset is not counted).
pub const INIT_STEP_COUNT: usize = 22;

/// Materialise the ATM90E32 initialization write sequence from a [`Config`].
///
/// The returned array can be iterated by the async driver to perform one
/// register write per entry. The sequence does **not** include the initial
/// soft reset — that is issued separately by the driver (with its own delay
/// afterwards).
#[rustfmt::skip]
pub fn build_init_sequence(cfg: &Config) -> [InitStep; INIT_STEP_COUNT] {
    // Frequency thresholds depend on mains frequency.
    let (freq_hi, freq_lo) = match cfg.line_freq_hz {
        LineFreq::Hz60 => (6300u16, 5700u16), // 63.00 / 57.00 Hz
        LineFreq::Hz50 => (5300u16, 4700u16), // 53.00 / 47.00 Hz
    };

    // MMode0: for 60 Hz operation bit 12 must be set.
    let mmode0 = match cfg.line_freq_hz {
        LineFreq::Hz60 => cfg.mmode0_base | (1 << 12),
        LineFreq::Hz50 => cfg.mmode0_base,
    };

    [
        InitStep { addr: REG_CFGREGACCEN,   value: 0x55AA,                stage: InitStage::UnlockConfig },
        InitStep { addr: REG_METEREN,       value: 0x0001,                stage: InitStage::EnableMeter },
        InitStep { addr: REG_SAGPEAKDETCFG, value: cfg.sag_peak_det_cfg,  stage: InitStage::WriteSagPeak },
        InitStep { addr: REG_PLCONSTH,      value: cfg.pl_constant_high,  stage: InitStage::WritePlConst },
        InitStep { addr: REG_PLCONSTL,      value: cfg.pl_constant_low,   stage: InitStage::WritePlConst },
        InitStep { addr: REG_ZXCONFIG,      value: cfg.zx_config,         stage: InitStage::WriteZxConfig },
        InitStep { addr: REG_MMODE0,        value: mmode0,                stage: InitStage::WriteMMode0 },
        InitStep { addr: REG_MMODE1,        value: cfg.pga_gain.mmode1(), stage: InitStage::WriteMMode1 },
        InitStep { addr: REG_FREQHITH,      value: freq_hi,               stage: InitStage::WriteFreqThresholds },
        InitStep { addr: REG_FREQLOTH,      value: freq_lo,               stage: InitStage::WriteFreqThresholds },
        InitStep { addr: REG_PSTARTTH,      value: cfg.pstart_threshold,  stage: InitStage::WriteStartupThresholds },
        InitStep { addr: REG_QSTARTTH,      value: cfg.qstart_threshold,  stage: InitStage::WriteStartupThresholds },
        InitStep { addr: REG_SSTARTTH,      value: cfg.sstart_threshold,  stage: InitStage::WriteStartupThresholds },
        InitStep { addr: REG_PPHASETH,      value: cfg.pphase_threshold,  stage: InitStage::WriteStartupThresholds },
        InitStep { addr: REG_QPHASETH,      value: cfg.qphase_threshold,  stage: InitStage::WriteStartupThresholds },
        InitStep { addr: REG_UGAIN_A,       value: cfg.voltage_gain[0],   stage: InitStage::WriteVoltageGains },
        InitStep { addr: REG_UGAIN_B,       value: cfg.voltage_gain[1],   stage: InitStage::WriteVoltageGains },
        InitStep { addr: REG_UGAIN_C,       value: cfg.voltage_gain[2],   stage: InitStage::WriteVoltageGains },
        InitStep { addr: REG_IGAIN_A,       value: cfg.current_gain[0],   stage: InitStage::WriteCurrentGains },
        InitStep { addr: REG_IGAIN_B,       value: cfg.current_gain[1],   stage: InitStage::WriteCurrentGains },
        InitStep { addr: REG_IGAIN_C,       value: cfg.current_gain[2],   stage: InitStage::WriteCurrentGains },
        InitStep { addr: REG_CFGREGACCEN,   value: 0x0000,                stage: InitStage::LockConfig },
    ]
}
