// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Phase and frequency status decoded from the ATM90E32 EMM state registers.

/// Decoded status flags from `EMMState0` (0x71) and `EMMState1` (0x72).
///
/// Each flag represents a hardware-detected condition. The chip sets these
/// bits in real time; the driver reads the two 16-bit registers and unpacks
/// them into this struct via [`from_emm`](Self::from_emm).
///
/// All fields default to `false` (no anomalies detected).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PhaseStatus {
    /// Overcurrent detected (I >= 65.53 A), per phase (A, B, C).
    pub overcurrent: [bool; 3],
    /// Overvoltage detected (> 122% of nominal), per phase (A, B, C).
    pub overvoltage: [bool; 3],
    /// Voltage sag detected (< 78% of nominal), per phase (A, B, C).
    pub voltage_sag: [bool; 3],
    /// Phase loss detected (phase absent), per phase (A, B, C).
    pub phase_loss: [bool; 3],
    /// Frequency above high threshold.
    pub freq_high: bool,
    /// Frequency below low threshold.
    pub freq_low: bool,
    /// Voltage phase sequence error (reverse rotation).
    pub voltage_seq_error: bool,
    /// Current phase sequence error (reverse rotation).
    pub current_seq_error: bool,
}

// ── EMMState0 bit positions (0x71) ──────────────────────────────────
//
// Bit 15: OI Phase A   (overcurrent)
// Bit 14: OI Phase B
// Bit 13: OI Phase C
// Bit 12: OV Phase A   (overvoltage)
// Bit 11: OV Phase B
// Bit 10: OV Phase C
// Bit  9: UREV_WN      (voltage sequence error)
// Bit  8: IREV_WN      (current sequence error)

const S0_OI_A: u16 = 1 << 15;
const S0_OI_B: u16 = 1 << 14;
const S0_OI_C: u16 = 1 << 13;
const S0_OV_A: u16 = 1 << 12;
const S0_OV_B: u16 = 1 << 11;
const S0_OV_C: u16 = 1 << 10;
const S0_UREV: u16 = 1 << 9;
const S0_IREV: u16 = 1 << 8;

// ── EMMState1 bit positions (0x72) ──────────────────────────────────
//
// Bit 15: FREQ_HI      (frequency above threshold)
// Bit 14: SAG Phase A   (voltage sag)
// Bit 13: SAG Phase B
// Bit 12: SAG Phase C
// Bit 11: FREQ_LO      (frequency below threshold)
// Bit 10: Phase Loss A
// Bit  9: Phase Loss B
// Bit  8: Phase Loss C

const S1_FREQ_HI: u16 = 1 << 15;
const S1_SAG_A: u16 = 1 << 14;
const S1_SAG_B: u16 = 1 << 13;
const S1_SAG_C: u16 = 1 << 12;
const S1_FREQ_LO: u16 = 1 << 11;
const S1_LOSS_A: u16 = 1 << 10;
const S1_LOSS_B: u16 = 1 << 9;
const S1_LOSS_C: u16 = 1 << 8;

impl PhaseStatus {
    /// Decode status from the raw `EMMState0` and `EMMState1` register values.
    pub fn from_emm(state0: u16, state1: u16) -> Self {
        Self {
            overcurrent: [
                state0 & S0_OI_A != 0,
                state0 & S0_OI_B != 0,
                state0 & S0_OI_C != 0,
            ],
            overvoltage: [
                state0 & S0_OV_A != 0,
                state0 & S0_OV_B != 0,
                state0 & S0_OV_C != 0,
            ],
            voltage_sag: [
                state1 & S1_SAG_A != 0,
                state1 & S1_SAG_B != 0,
                state1 & S1_SAG_C != 0,
            ],
            phase_loss: [
                state1 & S1_LOSS_A != 0,
                state1 & S1_LOSS_B != 0,
                state1 & S1_LOSS_C != 0,
            ],
            freq_high: state1 & S1_FREQ_HI != 0,
            freq_low: state1 & S1_FREQ_LO != 0,
            voltage_seq_error: state0 & S0_UREV != 0,
            current_seq_error: state0 & S0_IREV != 0,
        }
    }

    /// Returns `true` if no anomalies are detected (all flags are `false`).
    pub fn is_ok(&self) -> bool {
        *self == Self::default()
    }
}
