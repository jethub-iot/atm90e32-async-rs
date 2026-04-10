// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Measurement result types.

/// A full snapshot of the 3-phase measurements from a single
/// [`Atm90e32::read_all_phases`] call.
///
/// All values are in engineering units:
///
/// * `voltage` — RMS phase voltage, volts
/// * `current` — RMS phase current, amps
/// * `power` — active power, watts
/// * `reactive` — reactive power, vars
/// * `pf` — power factor, dimensionless in `[-1.0, 1.0]`
/// * `frequency` — mains frequency, hertz
///
/// Each `[f32; 3]` array is indexed in phase order A, B, C.
///
/// The field layout is deliberately stable across patch releases so
/// consumers can destructure by name safely.
///
/// [`Atm90e32::read_all_phases`]: crate::Atm90e32::read_all_phases
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PhaseReadings {
    /// RMS phase voltage [V], per phase (A, B, C).
    pub voltage: [f32; 3],
    /// RMS phase current [A], per phase (A, B, C).
    pub current: [f32; 3],
    /// Active power [W], per phase (A, B, C).
    pub power: [f32; 3],
    /// Reactive power [var], per phase (A, B, C).
    pub reactive: [f32; 3],
    /// Power factor, dimensionless, per phase (A, B, C).
    pub pf: [f32; 3],
    /// Mains frequency [Hz].
    pub frequency: f32,
}
