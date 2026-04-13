// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Host-runnable unit tests for the sans-I/O `proto` module.
//!
//! No SPI, no mocks, no embedded target required. These tests exercise the
//! exact byte-level wire format and the raw → engineering unit conversions,
//! which are the parts of the driver most prone to silent regressions.

use atm90e32_async::proto::*;
use atm90e32_async::registers::*;
use atm90e32_async::status::PhaseStatus;
use atm90e32_async::{Config, InitStage, LineFreq, PgaGain};

// ── Frame building / parsing ─────────────────────────────────────────

#[test]
fn read_frame_high_bit_set_and_addr_10bit_masked() {
    // Lowest address: only the read flag is set.
    assert_eq!(build_read_frame(0x000), [0x80, 0x00, 0x00, 0x00]);
    // Highest 10-bit address: both addr bits and read flag in byte 0.
    assert_eq!(build_read_frame(0x3FF), [0x83, 0xFF, 0x00, 0x00]);
    // Typical register from the datasheet (SAGPEAKDETCFG = 0x05).
    assert_eq!(build_read_frame(0x05), [0x80, 0x05, 0x00, 0x00]);
    // 10-bit overflow guard: anything above 0x3FF must mask to 10 bits.
    assert_eq!(build_read_frame(0x1D4C), [0x81, 0x4C, 0x00, 0x00]);
}

#[test]
fn write_frame_clears_read_flag_and_carries_big_endian_value() {
    assert_eq!(build_write_frame(0x7F, 0x55AA), [0x00, 0x7F, 0x55, 0xAA]);
    assert_eq!(build_write_frame(0x70, 0x789A), [0x00, 0x70, 0x78, 0x9A]);
    assert_eq!(build_write_frame(0xF8, 0x0000), [0x00, 0xF8, 0x00, 0x00]);
}

#[test]
fn parse_read_response_big_endian() {
    assert_eq!(parse_read_response(&[0x00, 0x00, 0xDE, 0xAD]), 0xDEAD);
    assert_eq!(parse_read_response(&[0xFF, 0xFF, 0x00, 0x01]), 0x0001);
    assert_eq!(parse_read_response(&[0x12, 0x34, 0xAB, 0xCD]), 0xABCD);
}

// ── Engineering-unit conversions ────────────────────────────────────

#[test]
fn voltage_conversion_matches_esphome() {
    assert_eq!(voltage_raw_to_volts(23000), 230.0);
    assert_eq!(voltage_raw_to_volts(0), 0.0);
    assert_eq!(voltage_raw_to_volts(100), 1.0);
}

#[test]
fn current_conversion_matches_esphome() {
    assert_eq!(current_raw_to_amps(5500), 5.5);
    assert_eq!(current_raw_to_amps(1000), 1.0);
    assert_eq!(current_raw_to_amps(0), 0.0);
}

#[test]
fn frequency_conversion_matches_esphome() {
    assert_eq!(frequency_raw_to_hz(5000), 50.0);
    assert_eq!(frequency_raw_to_hz(6000), 60.0);
    assert_eq!(frequency_raw_to_hz(4998), 49.98);
}

#[test]
fn power_factor_sign_extension() {
    // +1.0 PF
    assert_eq!(power_factor_raw_to_unitless(1000), 1.0);
    // -1.0 PF: -1000 as i16 == 0xFC18 as u16
    assert_eq!(power_factor_raw_to_unitless(0xFC18), -1.0);
    // 0 PF
    assert_eq!(power_factor_raw_to_unitless(0), 0.0);
    // Arbitrary positive
    assert_eq!(power_factor_raw_to_unitless(500), 0.5);
}

#[test]
fn power_word_combination_32bit() {
    // Combines 32-bit signed value from (high, low) u16 pair.
    assert_eq!(combine_power_words(0x0001, 0x0000), 0x0001_0000);
    assert_eq!(combine_power_words(0x0000, 0x00FF), 0x0000_00FF);
    // Negative values: high MSB set.
    assert_eq!(combine_power_words(0xFFFF, 0xFFFF), -1);
    assert_eq!(combine_power_words(0xFFFE, 0x0000), -(0x0002_0000));
}

#[test]
fn power_raw_to_watts_applies_power_scale() {
    // 0x0001_0000 * 0.00032 = 65536 * 0.00032 = 20.97152
    let watts = power_raw_to_watts(0x0001, 0x0000);
    assert!((watts - 65536.0 * 0.00032).abs() < f32::EPSILON);
    // Zero → zero
    assert_eq!(power_raw_to_watts(0, 0), 0.0);
}

#[test]
fn power_combined_to_watts_matches_split() {
    // Must produce the same result as power_raw_to_watts for any (hi, lo) pair.
    let hi: u16 = 0x0001;
    let lo: u16 = 0x0000;
    let combined = combine_power_words(hi, lo);
    assert_eq!(
        power_combined_to_watts(combined),
        power_raw_to_watts(hi, lo)
    );
    // Negative
    assert_eq!(
        power_combined_to_watts(-1),
        power_raw_to_watts(0xFFFF, 0xFFFF)
    );
    // Zero
    assert_eq!(power_combined_to_watts(0), 0.0);
}

// ── Init sequence materialisation ───────────────────────────────────

#[test]
fn init_sequence_has_expected_length() {
    let seq = build_init_sequence(&Config::default());
    assert_eq!(seq.len(), INIT_STEP_COUNT);
    assert_eq!(seq.len(), 22);
}

#[test]
fn init_sequence_starts_with_unlock_and_ends_with_lock() {
    let seq = build_init_sequence(&Config::default());
    // First step unlocks the config registers.
    assert_eq!(seq[0].addr, REG_CFGREGACCEN);
    assert_eq!(seq[0].value, 0x55AA);
    assert_eq!(seq[0].stage, InitStage::UnlockConfig);
    // Last step locks them again.
    let last = seq[seq.len() - 1];
    assert_eq!(last.addr, REG_CFGREGACCEN);
    assert_eq!(last.value, 0x0000);
    assert_eq!(last.stage, InitStage::LockConfig);
}

#[test]
fn init_sequence_enables_metering_early() {
    let seq = build_init_sequence(&Config::default());
    // METEREN = 0x0001 must be one of the first few writes.
    assert_eq!(seq[1].addr, REG_METEREN);
    assert_eq!(seq[1].value, 0x0001);
    assert_eq!(seq[1].stage, InitStage::EnableMeter);
}

#[test]
fn init_50hz_clears_mmode0_bit12() {
    let cfg = Config::default().with_line_freq(LineFreq::Hz50);
    let seq = build_init_sequence(&cfg);
    let mmode0 = seq.iter().find(|s| s.addr == REG_MMODE0).unwrap();
    assert_eq!(mmode0.value, 0x87);
    assert_eq!(mmode0.value & (1 << 12), 0);
}

#[test]
fn init_60hz_sets_mmode0_bit12() {
    let cfg = Config::default().with_line_freq(LineFreq::Hz60);
    let seq = build_init_sequence(&cfg);
    let mmode0 = seq.iter().find(|s| s.addr == REG_MMODE0).unwrap();
    assert_eq!(mmode0.value, 0x87 | (1 << 12));
}

#[test]
fn init_freq_thresholds_depend_on_line_frequency() {
    let cfg50 = Config::default().with_line_freq(LineFreq::Hz50);
    let seq50 = build_init_sequence(&cfg50);
    assert_eq!(
        seq50.iter().find(|s| s.addr == REG_FREQHITH).unwrap().value,
        5300
    );
    assert_eq!(
        seq50.iter().find(|s| s.addr == REG_FREQLOTH).unwrap().value,
        4700
    );

    let cfg60 = Config::default().with_line_freq(LineFreq::Hz60);
    let seq60 = build_init_sequence(&cfg60);
    assert_eq!(
        seq60.iter().find(|s| s.addr == REG_FREQHITH).unwrap().value,
        6300
    );
    assert_eq!(
        seq60.iter().find(|s| s.addr == REG_FREQLOTH).unwrap().value,
        5700
    );
}

#[test]
fn init_pga_gain_propagates_to_mmode1() {
    for (gain, expected) in [
        (PgaGain::X1, 0x00),
        (PgaGain::X2, 0x15),
        (PgaGain::X4, 0x2A),
    ] {
        let cfg = Config::default().with_pga_gain(gain);
        let seq = build_init_sequence(&cfg);
        let mmode1 = seq.iter().find(|s| s.addr == REG_MMODE1).unwrap();
        assert_eq!(mmode1.value, expected, "gain {:?}", gain);
    }
}

#[test]
fn init_voltage_gains_are_threaded_through() {
    let cfg = Config::default().with_voltage_gain([111, 222, 333]);
    let seq = build_init_sequence(&cfg);
    assert_eq!(
        seq.iter().find(|s| s.addr == REG_UGAIN_A).unwrap().value,
        111
    );
    assert_eq!(
        seq.iter().find(|s| s.addr == REG_UGAIN_B).unwrap().value,
        222
    );
    assert_eq!(
        seq.iter().find(|s| s.addr == REG_UGAIN_C).unwrap().value,
        333
    );
}

#[test]
fn init_current_gains_are_threaded_through() {
    let cfg = Config::default().with_current_gain([444, 555, 666]);
    let seq = build_init_sequence(&cfg);
    assert_eq!(
        seq.iter().find(|s| s.addr == REG_IGAIN_A).unwrap().value,
        444
    );
    assert_eq!(
        seq.iter().find(|s| s.addr == REG_IGAIN_B).unwrap().value,
        555
    );
    assert_eq!(
        seq.iter().find(|s| s.addr == REG_IGAIN_C).unwrap().value,
        666
    );
}

// ── Phase angle conversion ──────────────────────────────────────────

#[test]
fn phase_angle_raw_to_degrees_conversion() {
    assert_eq!(phase_angle_raw_to_degrees(0), 0.0);
    assert_eq!(phase_angle_raw_to_degrees(900), 90.0);
    assert_eq!(phase_angle_raw_to_degrees(1800), 180.0);
    assert_eq!(phase_angle_raw_to_degrees(3600), 360.0);
    // Fractional: 45.5 degrees
    assert!((phase_angle_raw_to_degrees(455) - 45.5).abs() < f32::EPSILON);
}

// ── Chip temperature conversion ─────────────────────────────────────

#[test]
fn chip_temperature_positive() {
    assert_eq!(chip_temperature_raw(25), 25.0);
    assert_eq!(chip_temperature_raw(0), 0.0);
    assert_eq!(chip_temperature_raw(85), 85.0);
}

#[test]
fn chip_temperature_negative() {
    // -25 as i16 == 0xFFE7 as u16
    assert_eq!(chip_temperature_raw(0xFFE7), -25.0);
    // -1 as i16 == 0xFFFF as u16
    assert_eq!(chip_temperature_raw(0xFFFF), -1.0);
}

// ── PhaseStatus from EMM registers ──────────────────────────────────

#[test]
fn phase_status_all_clear() {
    let st = PhaseStatus::from_emm(0x0000, 0x0000);
    assert!(st.is_ok());
    assert_eq!(st, PhaseStatus::default());
}

#[test]
fn phase_status_overcurrent_phase_a() {
    let st = PhaseStatus::from_emm(0x8000, 0x0000);
    assert!(st.overcurrent[0]);
    assert!(!st.overcurrent[1]);
    assert!(!st.overcurrent[2]);
    assert!(!st.is_ok());
}

#[test]
fn phase_status_overcurrent_all_three() {
    // Bits 15, 14, 13 set = 0xE000
    let st = PhaseStatus::from_emm(0xE000, 0x0000);
    assert!(st.overcurrent[0]);
    assert!(st.overcurrent[1]);
    assert!(st.overcurrent[2]);
}

#[test]
fn phase_status_overvoltage_all_three() {
    // Bits 12, 11, 10 set = 0x1C00
    let st = PhaseStatus::from_emm(0x1C00, 0x0000);
    assert!(st.overvoltage[0]);
    assert!(st.overvoltage[1]);
    assert!(st.overvoltage[2]);
}

#[test]
fn phase_status_voltage_sag_all_three() {
    // State1 bits 14, 13, 12 set = 0x7000
    let st = PhaseStatus::from_emm(0x0000, 0x7000);
    assert!(st.voltage_sag[0]);
    assert!(st.voltage_sag[1]);
    assert!(st.voltage_sag[2]);
}

#[test]
fn phase_status_phase_loss_all_three() {
    // State1 bits 10, 9, 8 set = 0x0700
    let st = PhaseStatus::from_emm(0x0000, 0x0700);
    assert!(st.phase_loss[0]);
    assert!(st.phase_loss[1]);
    assert!(st.phase_loss[2]);
}

#[test]
fn phase_status_freq_high_and_low() {
    // State1 bit 15 = freq_high, bit 11 = freq_low → 0x8800
    let st = PhaseStatus::from_emm(0x0000, 0x8800);
    assert!(st.freq_high);
    assert!(st.freq_low);
    assert!(!st.is_ok());
}

#[test]
fn phase_status_sequence_errors() {
    // State0 bit 9 = UREV, bit 8 = IREV → 0x0300
    let st = PhaseStatus::from_emm(0x0300, 0x0000);
    assert!(st.voltage_seq_error);
    assert!(st.current_seq_error);
}
