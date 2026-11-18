// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Dummy pin for optional RST.

use embedded_hal::digital::OutputPin;

/// Dummy pin used when no hardware reset pin is available.
///
/// Use [`crate::St7789v::new_no_rst`] when the RESET pin is tied to 3.3V
/// or controlled externally.
pub struct NoPin;

impl OutputPin for NoPin {
    fn set_low(&mut self)  -> Result<(), Self::Error> { Ok(()) }
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

impl embedded_hal::digital::ErrorType for NoPin {
    type Error = core::convert::Infallible;
}