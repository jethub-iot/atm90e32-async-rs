// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Initialization configuration for the ATM90E32.
//!
//! [`Config`] holds every value written during [`Atm90e32::init`].
//! All fields are public and [`Default`] provides the known-good values
//! used by the ESPHome `atm90e32` component for a 50 Hz 3-phase 4-wire grid.
//!
//! The common tuning knobs — voltage/current gains, line frequency, PGA gain
//! — have dedicated builder methods. The other fields are advanced and most
//! callers should leave them at their defaults.
//!
//! [`Atm90e32::init`]: crate::Atm90e32::init

/// Mains line frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LineFreq {
    /// 50 Hz (Europe, most of Asia, Africa, Australia).
    Hz50,
    /// 60 Hz (North America, parts of Japan, Brazil).
    Hz60,
}

/// Programmable Gain Amplifier setting for the current channels.
///
/// Controls the MMode1 register value written during init.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PgaGain {
    /// 1× gain — raw MMode1 value `0x00`.
    X1 = 0x00,
    /// 2× gain — raw MMode1 value `0x15`.
    X2 = 0x15,
    /// 4× gain — raw MMode1 value `0x2A`.
    X4 = 0x2A,
}

impl PgaGain {
    /// Raw MMode1 register value for this PGA setting.
    pub const fn mmode1(self) -> u16 {
        self as u16
    }
}

/// Initialization configuration.
///
/// Construct with [`Config::default`] and override fields via the builder
/// methods, e.g.:
///
/// ```no_run
/// use atm90e32_async::{Config, LineFreq, PgaGain};
/// let cfg = Config::default()
///     .with_voltage_gain([39470, 39470, 39470])
///     .with_current_gain([65327, 65327, 65327])
///     .with_line_freq(LineFreq::Hz50)
///     .with_pga_gain(PgaGain::X2);
/// ```
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    // ── Calibration ─────────────────────────────────────────────────
    /// Per-phase voltage gain (A, B, C). Chip-specific; caller must set.
    pub voltage_gain: [u16; 3],
    /// Per-phase current gain (A, B, C). Chip-specific; caller must set.
    pub current_gain: [u16; 3],
    /// Mains line frequency.
    pub line_freq_hz: LineFreq,
    /// PGA gain for the current channels.
    pub pga_gain: PgaGain,

    // ── Advanced (ESPHome defaults) ─────────────────────────────────
    /// Sag/peak detector configuration (`REG_SAGPEAKDETCFG`). Default: `0xFF3F`
    /// (peak detector 255 ms, sag period 63 ms).
    pub sag_peak_det_cfg: u16,
    /// PL constant high word (`REG_PLCONSTH`). Default: `0x0861`.
    pub pl_constant_high: u16,
    /// PL constant low word (`REG_PLCONSTL`). Default: `0xC468`.
    pub pl_constant_low: u16,
    /// Zero-crossing configuration (`REG_ZXCONFIG`). Default: `0xD654`.
    pub zx_config: u16,
    /// Base value for MMode0 (`REG_MMODE0`). Default: `0x87` (3P4W).
    /// Bit 12 is toggled automatically based on [`line_freq_hz`](Self::line_freq_hz).
    pub mmode0_base: u16,

    /// Active power startup threshold (`REG_PSTARTTH`). Default: `0x1D4C`.
    pub pstart_threshold: u16,
    /// Reactive power startup threshold (`REG_QSTARTTH`). Default: `0x1D4C`.
    pub qstart_threshold: u16,
    /// Apparent power startup threshold (`REG_SSTARTTH`). Default: `0x1D4C`.
    pub sstart_threshold: u16,
    /// Per-phase active power accumulation threshold (`REG_PPHASETH`). Default: `0x02EE`.
    pub pphase_threshold: u16,
    /// Per-phase reactive power accumulation threshold (`REG_QPHASETH`). Default: `0x02EE`.
    pub qphase_threshold: u16,

    /// Delay in milliseconds after soft reset before further writes. Default: `6`.
    pub post_reset_delay_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Construct a config with the default (ESPHome) values.
    ///
    /// The voltage and current gain fields are initialized to `[1, 1, 1]`
    /// which is **not** a useful calibration — callers must override them
    /// with values suited to their transformer/CT combination.
    pub const fn new() -> Self {
        Self {
            voltage_gain: [1, 1, 1],
            current_gain: [1, 1, 1],
            line_freq_hz: LineFreq::Hz50,
            pga_gain: PgaGain::X1,
            sag_peak_det_cfg: 0xFF3F,
            pl_constant_high: 0x0861,
            pl_constant_low: 0xC468,
            zx_config: 0xD654,
            mmode0_base: 0x87,
            pstart_threshold: 0x1D4C,
            qstart_threshold: 0x1D4C,
            sstart_threshold: 0x1D4C,
            pphase_threshold: 0x02EE,
            qphase_threshold: 0x02EE,
            post_reset_delay_ms: 6,
        }
    }

    /// Set the per-phase voltage gain.
    pub const fn with_voltage_gain(mut self, gain: [u16; 3]) -> Self {
        self.voltage_gain = gain;
        self
    }

    /// Set the per-phase current gain.
    pub const fn with_current_gain(mut self, gain: [u16; 3]) -> Self {
        self.current_gain = gain;
        self
    }

    /// Set the mains line frequency.
    pub const fn with_line_freq(mut self, freq: LineFreq) -> Self {
        self.line_freq_hz = freq;
        self
    }

    /// Set the PGA gain for the current channels.
    pub const fn with_pga_gain(mut self, gain: PgaGain) -> Self {
        self.pga_gain = gain;
        self
    }

    /// Set the post-reset delay in milliseconds.
    pub const fn with_post_reset_delay_ms(mut self, delay_ms: u32) -> Self {
        self.post_reset_delay_ms = delay_ms;
        self
    }
}
