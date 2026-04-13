// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Measurement result types.

/// A full snapshot of the 3-phase measurements from a single
/// [`Atm90e32::read_all_phases`] call.
///
/// All values are **raw register values** — lossless, no floating-point
/// conversion. Use the [`proto`](crate::proto) helpers to convert to
/// engineering units when needed:
///
/// | Field | Raw unit | Converter |
/// |-------|----------|-----------|
/// | `voltage` | hundredths of a volt (u16) | [`voltage_raw_to_volts`](crate::proto::voltage_raw_to_volts) |
/// | `current` | thousandths of an amp (u16) | [`current_raw_to_amps`](crate::proto::current_raw_to_amps) |
/// | `power` | signed 32-bit combined word (i32) | [`power_combined_to_watts`](crate::proto::power_combined_to_watts) |
/// | `reactive` | signed 32-bit combined word (i32) | [`power_combined_to_watts`](crate::proto::power_combined_to_watts) |
/// | `pf` | signed thousandths (i16) | [`power_factor_raw_to_unitless`](crate::proto::power_factor_raw_to_unitless) |
/// | `frequency` | hundredths of a hertz (u16) | [`frequency_raw_to_hz`](crate::proto::frequency_raw_to_hz) |
/// | `phase_angle` | tenths of a degree (u16) | [`phase_angle_raw_to_degrees`](crate::proto::phase_angle_raw_to_degrees) |
///
/// Each three-element array is indexed in phase order A, B, C.
///
/// [`Atm90e32::read_all_phases`]: crate::Atm90e32::read_all_phases
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PhaseReadings {
    /// RMS phase voltage, per phase (A, B, C). Raw: hundredths of a volt.
    pub voltage: [u16; 3],
    /// RMS phase current, per phase (A, B, C). Raw: thousandths of an amp.
    pub current: [u16; 3],
    /// Active power, per phase (A, B, C). Raw: signed 32-bit combined hi+lo register words.
    pub power: [i32; 3],
    /// Reactive power, per phase (A, B, C). Raw: signed 32-bit combined hi+lo register words.
    pub reactive: [i32; 3],
    /// Power factor, per phase (A, B, C). Raw: signed thousandths (-1000..=1000).
    pub pf: [i16; 3],
    /// Mains frequency. Raw: hundredths of a hertz.
    pub frequency: u16,
    /// Mean phase angle, per phase (A, B, C). Raw: tenths of a degree.
    pub phase_angle: [u16; 3],
}
