// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Broche fictive pour RST optionnel.

use embedded_hal::digital::OutputPin;

/// Broche fictive utilisée quand aucune broche de réinitialisation matérielle n'est disponible.
///
/// Utilisez [`crate::St7789v::new_no_rst`] quand la broche RESET est connectée
/// au 3.3V ou gérée en externe.
pub struct NoPin;

impl OutputPin for NoPin {
    fn set_low(&mut self)  -> Result<(), Self::Error> { Ok(()) }
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

impl embedded_hal::digital::ErrorType for NoPin {
    type Error = core::convert::Infallible;
}