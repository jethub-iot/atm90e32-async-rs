// SPDX-License-Identifier: (GPL-2.0-or-later OR Apache-2.0)
// Copyright (c) Viacheslav Bocharov <v@baodeep.com> and JetHome (r)

//! ATM90E32 3-phase power meter driver (under construction)

#![no_std]

pub mod error;
pub mod registers;

pub use crate::error::{Error, InitStage};
