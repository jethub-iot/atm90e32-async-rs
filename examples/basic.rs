// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! Minimal generic usage example for the `atm90e32-async` driver.
//!
//! This example is `cargo check`-friendly on the host. It uses no-op stubs
//! for the `embedded-hal-async` SPI and delay traits, so it compiles
//! everywhere but does not talk to real hardware.
//!
//! For a real integration on an ESP32 with Embassy, see the sketch below.
//!
//! ## Real Embassy + esp-hal integration (sketch)
//!
//! ```ignore
//! use atm90e32_async::{Atm90e32, Config, LineFreq, PgaGain};
//! use embassy_executor::Spawner;
//! use embassy_time::Delay;
//! use embedded_hal_bus::spi::ExclusiveDevice;
//! use esp_hal::{gpio::Output, spi::master::Spi};
//!
//! #[embassy_executor::task]
//! async fn meter_task(
//!     mut meter: Atm90e32<
//!         ExclusiveDevice<Spi<'static, esp_hal::Async>, Output<'static>, Delay>,
//!         Delay,
//!     >,
//! ) {
//!     let cfg = Config::default()
//!         .with_voltage_gain([39470, 39470, 39470])
//!         .with_current_gain([65327, 65327, 65327])
//!         .with_line_freq(LineFreq::Hz50)
//!         .with_pga_gain(PgaGain::X2);
//!
//!     meter.probe().await.expect("no chip on bus");
//!     meter.init(&cfg).await.expect("init failed");
//!
//!     loop {
//!         match meter.read_all_phases().await {
//!             Ok(r) => {
//!                 esp_println::println!(
//!                     "U={:?} I={:?} P={:?} F={:.2}",
//!                     r.voltage, r.current, r.power, r.frequency
//!                 );
//!             }
//!             Err(e) => esp_println::println!("SPI error: {:?}", e),
//!         }
//!         embassy_time::Timer::after_secs(2).await;
//!     }
//! }
//! ```

use core::convert::Infallible;

use atm90e32_async::{Atm90e32, Config, LineFreq, PgaGain};

/// A no-op SPI device that always succeeds — compile-check stub only.
struct DummySpi;

impl embedded_hal_async::spi::ErrorType for DummySpi {
    type Error = Infallible;
}

impl embedded_hal_async::spi::SpiDevice for DummySpi {
    async fn transaction(
        &mut self,
        _ops: &mut [embedded_hal_async::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A no-op delay — compile-check stub only.
struct DummyDelay;

impl embedded_hal_async::delay::DelayNs for DummyDelay {
    async fn delay_ns(&mut self, _ns: u32) {}
}

async fn run() -> Result<(), atm90e32_async::Error<Infallible>> {
    let mut meter = Atm90e32::new(DummySpi, DummyDelay);

    // probe() against the dummy SPI reads 0x0000, which the driver treats
    // as "chip not present". We deliberately ignore that here: this is a
    // compile-check example, not a functional one.
    let _ = meter.probe().await;

    let cfg = Config::default()
        .with_voltage_gain([39470, 39470, 39470])
        .with_current_gain([65327, 65327, 65327])
        .with_line_freq(LineFreq::Hz50)
        .with_pga_gain(PgaGain::X2);
    let _ = meter.init(&cfg).await;

    Ok(())
}

fn main() {
    // Hand-rolled minimal poll loop so the example has no async-runtime
    // dev-dependency.
    use core::future::Future;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null(), &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);

    // Safety: the vtable functions ignore the data pointer, so a null
    // pointer is sound here.
    #[allow(unsafe_code)]
    let waker = unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);

    let mut fut = core::pin::pin!(run());
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(res) => {
            let ok: bool = res.is_ok();
            println!("example finished ok={}", ok);
        }
        Poll::Pending => unreachable!("dummy stubs never yield"),
    }
}
