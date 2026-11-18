// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! RGB565 color value for the ST7789V display.

/// RGB565 color value for the ST7789V display.
///
/// Stored internally as a 16-bit big-endian word:
/// `RRRRR GGGGGG BBBBB`
///
/// # Exemples
///
/// ```
/// use embassy_st7789v::Color;
///
/// let red     = Color::rgb(31, 0, 0);
/// let white   = Color::rgb8(255, 255, 255);
/// let custom  = Color::rgb8(0x1A, 0x8C, 0xFF);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color(pub u16);

impl Color {
    /// Creates a color from raw RGB565 components.
    ///
    /// - `r`: red channel, 0–31
    /// - `g`: green channel, 0–63
    /// - `b`: blue channel, 0–31
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u16 & 0x1F) << 11) | ((g as u16 & 0x3F) << 5) | (b as u16 & 0x1F))
    }

    /// Creates a color from 8-bit RGB components (0–255).
    ///
    /// Components are reduced to RGB565 precision:
    /// 5 bits each for red and blue, and 6 bits for green.
    #[inline]
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgb(r >> 3, g >> 2, b >> 3)
    }

    /// Returns the color as two big-endian bytes, ready to send over SPI.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }

    /// Pure black — `#000000`
    pub const BLACK:   Color = Color::rgb(0,  0,  0);
    /// Pure white — `#FFFFFF`
    pub const WHITE:   Color = Color::rgb(31, 63, 31);
    /// Pure red — `#F80000`
    pub const RED:     Color = Color::rgb(31, 0,  0);
    /// Pure green — `#07E000`
    pub const GREEN:   Color = Color::rgb(0,  63, 0);
    /// Pure blue — `#0000F8`
    pub const BLUE:    Color = Color::rgb(0,  0,  31);
    /// Yellow — `#F8FC00`
    pub const YELLOW:  Color = Color::rgb(31, 63, 0);
    /// Cyan — `#00FCF8`
    pub const CYAN:    Color = Color::rgb(0,  63, 31);
    /// Magenta — `#F800F8`
    pub const MAGENTA: Color = Color::rgb(31, 0,  31);
    /// Orange — `#F85000`
    pub const ORANGE:  Color = Color::rgb(31, 40, 0);
    /// Medium gray — `#787C78`
    pub const GRAY:    Color = Color::rgb(15, 31, 15);
}