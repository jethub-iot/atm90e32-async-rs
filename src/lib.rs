// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! ATM90E32 3-phase power meter driver (under construction)

#![no_std]

pub mod config;
pub mod error;
#[doc(hidden)]
pub mod proto;
pub mod registers;

pub use crate::config::{Config, LineFreq, PgaGain};
pub use crate::error::{Error, InitStage};
