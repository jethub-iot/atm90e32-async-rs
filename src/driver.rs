// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Async driver struct for the ATM90E32.
//!
//! This module is the thin async transport layer on top of the sans-I/O
//! `proto` module. Everything that touches the SPI bus or needs a delay
//! lives here; everything that can be tested on the host lives in `proto`.

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::SpiDevice;

use crate::config::Config;
use crate::error::{Error, InitStage};
use crate::proto::{self, build_init_sequence};
use crate::readings::PhaseReadings;
use crate::registers::*;

/// Phase selector for per-phase read methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Phase {
    /// Phase A (first phase).
    A,
    /// Phase B (second phase).
    B,
    /// Phase C (third phase).
    C,
}

impl Phase {
    /// Index into a `[T; 3]` array in A, B, C order.
    #[inline]
    fn index(self) -> usize {
        match self {
            Phase::A => 0,
            Phase::B => 1,
            Phase::C => 2,
        }
    }
}

/// Async ATM90E32 driver.
///
/// The driver owns a handle to an `embedded-hal-async` [`SpiDevice`] and a
/// [`DelayNs`] used for the post-reset wait in [`init`](Self::init). Both
/// are generic so the crate does not pull in any platform-specific runtime.
///
/// ## Example
///
/// ```no_run
/// # use atm90e32_async::{Atm90e32, Config, LineFreq, PgaGain};
/// # async fn demo<SPI, D>(spi: SPI, delay: D) -> Result<(), atm90e32_async::Error<SPI::Error>>
/// # where
/// #     SPI: embedded_hal_async::spi::SpiDevice,
/// #     D:   embedded_hal_async::delay::DelayNs,
/// # {
/// let mut meter = Atm90e32::new(spi, delay);
/// meter.probe().await?;
/// let cfg = Config::default()
///     .with_voltage_gain([39470, 39470, 39470])
///     .with_current_gain([65327, 65327, 65327])
///     .with_line_freq(LineFreq::Hz50)
///     .with_pga_gain(PgaGain::X2);
/// meter.init(&cfg).await?;
/// let readings = meter.read_all_phases().await?;
/// # Ok(())
/// # }
/// ```
pub struct Atm90e32<SPI, D> {
    spi: SPI,
    delay: D,
}

impl<SPI, D> Atm90e32<SPI, D>
where
    SPI: SpiDevice,
    D: DelayNs,
{
    /// Create a new driver.
    ///
    /// Does no I/O. The SPI device must be already configured for SPI
    /// mode 3 (CPOL=1, CPHA=1), MSB-first, 16-bit or 8-bit word size,
    /// clocked at ≤ 16 MHz (datasheet).
    pub fn new(spi: SPI, delay: D) -> Self {
        Self { spi, delay }
    }

    /// Destroy the driver and return the owned SPI device and delay.
    pub fn release(self) -> (SPI, D) {
        (self.spi, self.delay)
    }

    /// Read a 16-bit register.
    pub async fn read_register(&mut self, addr: u16) -> Result<u16, Error<SPI::Error>> {
        let tx = proto::build_read_frame(addr);
        let mut rx = [0u8; 4];
        self.spi.transfer(&mut rx, &tx).await.map_err(Error::Spi)?;
        Ok(proto::parse_read_response(&rx))
    }

    /// Write a 16-bit register.
    pub async fn write_register(&mut self, addr: u16, value: u16) -> Result<(), Error<SPI::Error>> {
        let tx = proto::build_write_frame(addr, value);
        self.spi.write(&tx).await.map_err(Error::Spi)
    }

    /// Probe for a chip on the bus.
    ///
    /// Reads `REG_SYSSTATUS0` and returns its value. If the bus reads back
    /// as `0x0000` or `0xFFFF` (floating / no device), returns
    /// [`Error::NotPresent`].
    pub async fn probe(&mut self) -> Result<u16, Error<SPI::Error>> {
        let status = self.read_register(REG_SYSSTATUS0).await?;
        if status == 0x0000 || status == 0xFFFF {
            Err(Error::NotPresent)
        } else {
            Ok(status)
        }
    }

    /// Initialise metering with the given configuration.
    ///
    /// Performs a soft reset, waits [`Config::post_reset_delay_ms`]
    /// milliseconds, and then plays the write sequence produced by
    /// [`build_init_sequence`]. Any SPI failure is wrapped into
    /// `Error::InitFailed(stage)` with the stage identifying which step
    /// broke.
    pub async fn init(&mut self, config: &Config) -> Result<(), Error<SPI::Error>> {
        // 1. Soft reset: fixed magic value 0x789A per datasheet.
        self.write_register(REG_SOFTRESET, 0x789A)
            .await
            .map_err(|_| Error::InitFailed(InitStage::SoftReset))?;
        self.delay.delay_ms(config.post_reset_delay_ms).await;

        // 2. Play the data-driven init sequence.
        for step in build_init_sequence(config).iter() {
            self.write_register(step.addr, step.value)
                .await
                .map_err(|_| Error::InitFailed(step.stage))?;
        }

        Ok(())
    }

    // ── 3-phase bulk readout ─────────────────────────────────────────

    /// Read all three phases in one call.
    ///
    /// Issues 16 SPI read transactions (1 frequency + 3×U + 3×I + 6×P,Q
    /// high/low + 3×PF) and returns the values in engineering units.
    pub async fn read_all_phases(&mut self) -> Result<PhaseReadings, Error<SPI::Error>> {
        let ua = self.read_register(REG_URMS_A).await?;
        let ub = self.read_register(REG_URMS_B).await?;
        let uc = self.read_register(REG_URMS_C).await?;

        let ia = self.read_register(REG_IRMS_A).await?;
        let ib = self.read_register(REG_IRMS_B).await?;
        let ic = self.read_register(REG_IRMS_C).await?;

        let pa_h = self.read_register(REG_PMEAN_A).await?;
        let pa_l = self.read_register(REG_PMEAN_A_LSB).await?;
        let pb_h = self.read_register(REG_PMEAN_B).await?;
        let pb_l = self.read_register(REG_PMEAN_B_LSB).await?;
        let pc_h = self.read_register(REG_PMEAN_C).await?;
        let pc_l = self.read_register(REG_PMEAN_C_LSB).await?;

        let qa_h = self.read_register(REG_QMEAN_A).await?;
        let qa_l = self.read_register(REG_QMEAN_A_LSB).await?;
        let qb_h = self.read_register(REG_QMEAN_B).await?;
        let qb_l = self.read_register(REG_QMEAN_B_LSB).await?;
        let qc_h = self.read_register(REG_QMEAN_C).await?;
        let qc_l = self.read_register(REG_QMEAN_C_LSB).await?;

        let pfa = self.read_register(REG_PFMEAN_A).await?;
        let pfb = self.read_register(REG_PFMEAN_B).await?;
        let pfc = self.read_register(REG_PFMEAN_C).await?;

        let freq = self.read_register(REG_FREQ).await?;

        Ok(PhaseReadings {
            voltage: [
                proto::voltage_raw_to_volts(ua),
                proto::voltage_raw_to_volts(ub),
                proto::voltage_raw_to_volts(uc),
            ],
            current: [
                proto::current_raw_to_amps(ia),
                proto::current_raw_to_amps(ib),
                proto::current_raw_to_amps(ic),
            ],
            power: [
                proto::power_raw_to_watts(pa_h, pa_l),
                proto::power_raw_to_watts(pb_h, pb_l),
                proto::power_raw_to_watts(pc_h, pc_l),
            ],
            reactive: [
                proto::power_raw_to_watts(qa_h, qa_l),
                proto::power_raw_to_watts(qb_h, qb_l),
                proto::power_raw_to_watts(qc_h, qc_l),
            ],
            pf: [
                proto::power_factor_raw_to_unitless(pfa),
                proto::power_factor_raw_to_unitless(pfb),
                proto::power_factor_raw_to_unitless(pfc),
            ],
            frequency: proto::frequency_raw_to_hz(freq),
        })
    }

    // ── Per-phase helpers ────────────────────────────────────────────

    /// Read the RMS voltage of a single phase in volts.
    pub async fn read_voltage(&mut self, phase: Phase) -> Result<f32, Error<SPI::Error>> {
        const REGS: [u16; 3] = [REG_URMS_A, REG_URMS_B, REG_URMS_C];
        let raw = self.read_register(REGS[phase.index()]).await?;
        Ok(proto::voltage_raw_to_volts(raw))
    }

    /// Read the RMS current of a single phase in amps.
    pub async fn read_current(&mut self, phase: Phase) -> Result<f32, Error<SPI::Error>> {
        const REGS: [u16; 3] = [REG_IRMS_A, REG_IRMS_B, REG_IRMS_C];
        let raw = self.read_register(REGS[phase.index()]).await?;
        Ok(proto::current_raw_to_amps(raw))
    }

    /// Read the active power of a single phase in watts.
    pub async fn read_active_power(&mut self, phase: Phase) -> Result<f32, Error<SPI::Error>> {
        const HI: [u16; 3] = [REG_PMEAN_A, REG_PMEAN_B, REG_PMEAN_C];
        const LO: [u16; 3] = [REG_PMEAN_A_LSB, REG_PMEAN_B_LSB, REG_PMEAN_C_LSB];
        let idx = phase.index();
        let hi = self.read_register(HI[idx]).await?;
        let lo = self.read_register(LO[idx]).await?;
        Ok(proto::power_raw_to_watts(hi, lo))
    }

    /// Read the reactive power of a single phase in vars.
    pub async fn read_reactive_power(&mut self, phase: Phase) -> Result<f32, Error<SPI::Error>> {
        const HI: [u16; 3] = [REG_QMEAN_A, REG_QMEAN_B, REG_QMEAN_C];
        const LO: [u16; 3] = [REG_QMEAN_A_LSB, REG_QMEAN_B_LSB, REG_QMEAN_C_LSB];
        let idx = phase.index();
        let hi = self.read_register(HI[idx]).await?;
        let lo = self.read_register(LO[idx]).await?;
        Ok(proto::power_raw_to_watts(hi, lo))
    }

    /// Read the power factor of a single phase (dimensionless, range `-1.0..=1.0`).
    pub async fn read_power_factor(&mut self, phase: Phase) -> Result<f32, Error<SPI::Error>> {
        const REGS: [u16; 3] = [REG_PFMEAN_A, REG_PFMEAN_B, REG_PFMEAN_C];
        let raw = self.read_register(REGS[phase.index()]).await?;
        Ok(proto::power_factor_raw_to_unitless(raw))
    }

    /// Read the mains line frequency in hertz.
    pub async fn read_frequency(&mut self) -> Result<f32, Error<SPI::Error>> {
        let raw = self.read_register(REG_FREQ).await?;
        Ok(proto::frequency_raw_to_hz(raw))
    }
}
