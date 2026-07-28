// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Valeur de couleur RGB565 pour l'écran ST7789V.

/// Valeur de couleur RGB565 pour l'écran ST7789V.
///
/// Stockée en interne comme un mot 16 bits big-endian :
/// `RRRRR GGGGGG BBBBB`
///
/// # Exemples
///
/// ```
/// use embassy_st7789v::Color;
///
/// let rouge   = Color::rgb(31, 0, 0);
/// let blanc   = Color::rgb8(255, 255, 255);
/// let custom  = Color::rgb8(0x1A, 0x8C, 0xFF);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color(pub u16);

impl Color {
    /// Crée une couleur à partir des composantes RGB565 brutes.
    ///
    /// - `r` : canal rouge,  0–31
    /// - `g` : canal vert,   0–63
    /// - `b` : canal bleu,   0–31
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u16 & 0x1F) << 11) | ((g as u16 & 0x3F) << 5) | (b as u16 & 0x1F))
    }

    /// Crée une couleur à partir des composantes RGB sur 8 bits (0–255).
    ///
    /// Les composantes sont réduites à la précision RGB565 :
    /// rouge et bleu sur 5 bits, vert sur 6 bits.
    #[inline]
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgb(r >> 3, g >> 2, b >> 3)
    }

    /// Retourne la couleur sous forme de deux octets big-endian, prêts à envoyer via SPI.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }

    /// Noir pur — `#000000`
    pub const BLACK:   Color = Color::rgb(0,  0,  0);
    /// Blanc pur — `#FFFFFF`
    pub const WHITE:   Color = Color::rgb(31, 63, 31);
    /// Rouge pur — `#F80000`
    pub const RED:     Color = Color::rgb(31, 0,  0);
    /// Vert pur — `#07E000`
    pub const GREEN:   Color = Color::rgb(0,  63, 0);
    /// Bleu pur — `#0000F8`
    pub const BLUE:    Color = Color::rgb(0,  0,  31);
    /// Jaune — `#F8FC00`
    pub const YELLOW:  Color = Color::rgb(31, 63, 0);
    /// Cyan — `#00FCF8`
    pub const CYAN:    Color = Color::rgb(0,  63, 31);
    /// Magenta — `#F800F8`
    pub const MAGENTA: Color = Color::rgb(31, 0,  31);
    /// Orange — `#F85000`
    pub const ORANGE:  Color = Color::rgb(31, 40, 0);
    /// Gris moyen — `#787C78`
    pub const GRAY:    Color = Color::rgb(15, 31, 15);
}