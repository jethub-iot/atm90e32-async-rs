// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! ATM90E32 register address map.
//!
//! The ATM90E32 uses 10-bit register addresses, so all constants are `u16`.
//! Addresses are grouped by function: configuration, calibration, and
//! measurement.

// ── Configuration & control ─────────────────────────────────────────

/// Meter enable register. Writing `0x0001` enables metering.
pub const REG_METEREN: u16 = 0x00;
/// System status 0 register. Used by [`probe`](crate::Atm90e32::probe) to
/// detect chip presence.
pub const REG_SYSSTATUS0: u16 = 0x01;
/// Sag / peak detector configuration.
pub const REG_SAGPEAKDETCFG: u16 = 0x05;
/// Zero-crossing detection configuration.
pub const REG_ZXCONFIG: u16 = 0x07;
/// Frequency low threshold.
pub const REG_FREQLOTH: u16 = 0x0C;
/// Frequency high threshold.
pub const REG_FREQHITH: u16 = 0x0D;
/// Pulse constant high word.
pub const REG_PLCONSTH: u16 = 0x31;
/// Pulse constant low word.
pub const REG_PLCONSTL: u16 = 0x32;
/// Metering mode 0 — topology and line frequency selection.
pub const REG_MMODE0: u16 = 0x33;
/// Metering mode 1 — PGA gain for the current channels.
pub const REG_MMODE1: u16 = 0x34;
/// Active power startup threshold.
pub const REG_PSTARTTH: u16 = 0x35;
/// Reactive power startup threshold.
pub const REG_QSTARTTH: u16 = 0x36;
/// Apparent power startup threshold.
pub const REG_SSTARTTH: u16 = 0x37;
/// Per-phase active power accumulation threshold.
pub const REG_PPHASETH: u16 = 0x38;
/// Per-phase reactive power accumulation threshold.
pub const REG_QPHASETH: u16 = 0x39;
/// Software reset register. Writing `0x789A` triggers a soft reset.
pub const REG_SOFTRESET: u16 = 0x70;
/// Configuration register access enable. `0x55AA` unlocks, `0x0000` locks.
pub const REG_CFGREGACCEN: u16 = 0x7F;

// ── Calibration: voltage gains ──────────────────────────────────────

/// Phase A voltage gain.
pub const REG_UGAIN_A: u16 = 0x61;
/// Phase B voltage gain.
pub const REG_UGAIN_B: u16 = 0x65;
/// Phase C voltage gain.
pub const REG_UGAIN_C: u16 = 0x69;

// ── Calibration: current gains ──────────────────────────────────────

/// Phase A current gain.
pub const REG_IGAIN_A: u16 = 0x62;
/// Phase B current gain.
pub const REG_IGAIN_B: u16 = 0x66;
/// Phase C current gain.
pub const REG_IGAIN_C: u16 = 0x6A;

// ── Measurement: voltage RMS ────────────────────────────────────────

/// Phase A RMS voltage (raw, hundredths of a volt).
pub const REG_URMS_A: u16 = 0xD9;
/// Phase B RMS voltage (raw, hundredths of a volt).
pub const REG_URMS_B: u16 = 0xDA;
/// Phase C RMS voltage (raw, hundredths of a volt).
pub const REG_URMS_C: u16 = 0xDB;

// ── Measurement: current RMS ────────────────────────────────────────

/// Phase A RMS current (raw, thousandths of an amp).
pub const REG_IRMS_A: u16 = 0xDD;
/// Phase B RMS current (raw, thousandths of an amp).
pub const REG_IRMS_B: u16 = 0xDE;
/// Phase C RMS current (raw, thousandths of an amp).
pub const REG_IRMS_C: u16 = 0xDF;

// ── Measurement: active power mean (high word) ──────────────────────

/// Phase A active power mean, high 16 bits.
pub const REG_PMEAN_A: u16 = 0xB1;
/// Phase B active power mean, high 16 bits.
pub const REG_PMEAN_B: u16 = 0xB2;
/// Phase C active power mean, high 16 bits.
pub const REG_PMEAN_C: u16 = 0xB3;

// ── Measurement: reactive power mean (high word) ────────────────────

/// Phase A reactive power mean, high 16 bits.
pub const REG_QMEAN_A: u16 = 0xB5;
/// Phase B reactive power mean, high 16 bits.
pub const REG_QMEAN_B: u16 = 0xB6;
/// Phase C reactive power mean, high 16 bits.
pub const REG_QMEAN_C: u16 = 0xB7;

// ── Measurement: active power mean (low word) ───────────────────────

/// Phase A active power mean, low 16 bits.
pub const REG_PMEAN_A_LSB: u16 = 0xC1;
/// Phase B active power mean, low 16 bits.
pub const REG_PMEAN_B_LSB: u16 = 0xC2;
/// Phase C active power mean, low 16 bits.
pub const REG_PMEAN_C_LSB: u16 = 0xC3;

// ── Measurement: reactive power mean (low word) ─────────────────────

/// Phase A reactive power mean, low 16 bits.
pub const REG_QMEAN_A_LSB: u16 = 0xC5;
/// Phase B reactive power mean, low 16 bits.
pub const REG_QMEAN_B_LSB: u16 = 0xC6;
/// Phase C reactive power mean, low 16 bits.
pub const REG_QMEAN_C_LSB: u16 = 0xC7;

// ── Measurement: power factor ───────────────────────────────────────

/// Phase A power factor (signed, thousandths).
pub const REG_PFMEAN_A: u16 = 0xBD;
/// Phase B power factor (signed, thousandths).
pub const REG_PFMEAN_B: u16 = 0xBE;
/// Phase C power factor (signed, thousandths).
pub const REG_PFMEAN_C: u16 = 0xBF;

// ── EMM status ──────────────────────────────────────────────────────

/// EMM state 0 — overcurrent, overvoltage, sequence errors, no-load flags.
pub const REG_EMMSTATE0: u16 = 0x71;
/// EMM state 1 — voltage sag, phase loss, frequency warnings, energy direction.
pub const REG_EMMSTATE1: u16 = 0x72;
/// EMM interrupt status 0 (latched copy of `EMMSTATE0`, cleared on read).
pub const REG_EMMINTSTATE0: u16 = 0x73;
/// EMM interrupt status 1 (latched copy of `EMMSTATE1`, cleared on read).
pub const REG_EMMINTSTATE1: u16 = 0x74;

// ── Measurement: peak current ───────────────────────────────────────

/// Phase A peak current.
pub const REG_IPEAK_A: u16 = 0xF5;
/// Phase B peak current.
pub const REG_IPEAK_B: u16 = 0xF6;
/// Phase C peak current.
pub const REG_IPEAK_C: u16 = 0xF7;

// ── Measurement: line frequency ─────────────────────────────────────

/// Line frequency (raw, hundredths of a hertz).
pub const REG_FREQ: u16 = 0xF8;

// ── Measurement: phase angle ────────────────────────────────────────

/// Phase A mean phase angle (raw, tenths of a degree).
pub const REG_PANGLE_A: u16 = 0xF9;
/// Phase B mean phase angle (raw, tenths of a degree).
pub const REG_PANGLE_B: u16 = 0xFA;
/// Phase C mean phase angle (raw, tenths of a degree).
pub const REG_PANGLE_C: u16 = 0xFB;

// ── Measurement: chip temperature ───────────────────────────────────

/// Chip temperature (signed, degrees Celsius).
pub const REG_TEMP: u16 = 0xFC;
