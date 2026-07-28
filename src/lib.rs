// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! # embassy-st7789v
//!
//! Pilote async `no_std` pour l'écran TFT LCD **ST7789V** 240×320 via SPI,
//! basé sur [Embassy](https://embassy.dev).
//!
//! ## Fonctionnalités
//!
//! - Couleurs RGB565 avec constantes nommées
//! - `fill_screen`, `fill_rect`, `draw_rect`
//! - `draw_pixel`, `draw_hline`, `draw_vline`
//! - Police bitmap 5×7 intégrée : ASCII, chiffres, symboles
//! - `draw_str`, `draw_i16`, `draw_u32`, `draw_f32`
//! - Texte mis à l'échelle avec `draw_char_scaled` et `draw_str_scaled`
//! - Rendu de bitmap 1 bit via `draw_bitmap`
//! - Réinitialisation matérielle et logicielle
//! - Contrôle de l'orientation (`MADCTL`) et de l'inversion
//! - Zéro allocation : `forbid(unsafe_code)`
//! - Framebuffer complet 240×320 RGB565 via `St7789vBuffered` : zéro clignotement
//!
//! ## Câblage (exemple RP2350)
//!
//! | Écran  | GPIO  | Pin physique |
//! |--------|-------|-------------|
//! | VDD    | 3.3V  |             |
//! | GND    | GND   |             |
//! | SCL    | GP18  | Pin 24      |
//! | SDA    | GP19  | Pin 25      |
//! | DC     | GP16  | Pin 21      |
//! | CS     | GP20  | Pin 26      |
//! | RESET  | GP17  | Pin 22      |
//!
//! ## Démarrage rapide
//!
//! ```no_run
//! use embassy_st7789v::{Color, St7789v};
//!
//! let mut ecran = St7789v::new(spi_device, broche_dc, broche_rst);
//! ecran.init().await.unwrap();
//! ecran.fill_screen(Color::BLACK).await.unwrap();
//! ecran.draw_str(8, 10, b"BONJOUR MONDE", Color::WHITE, Color::BLACK).await.unwrap();
//! ```

//! // Avec framebuffer (zéro clignotement) :
//! let mut ecran = St7789vBuffered::new(St7789v::new(spi, dc, rst));
//! ecran.driver().init().await.unwrap();
//! ecran.fill_screen_buf(Color::BLACK);
//! ecran.draw_str_buf(8, 10, b"BONJOUR", Color::WHITE, Color::BLACK);
//! ecran.flush().await.unwrap();

#![no_std]
#![forbid(unsafe_code)]

mod cmd;
mod font;
mod nopin;
mod color;
mod driver;
mod buffered;

pub use color::Color;
pub use nopin::NoPin;
pub use driver::St7789v;
pub use buffered::St7789vBuffered;

/// Largeur de l'écran en pixels.
pub const SCREEN_W: u16 = 240;

/// Hauteur de l'écran en pixels.
pub const SCREEN_H: u16 = 320;