// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Wrapper avec framebuffer complet en RAM pour le ST7789V.

use embedded_hal_async::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

use crate::cmd;
use crate::color::Color;
use crate::font::{self, FONT};
use crate::nopin::NoPin;
use crate::driver::St7789v;
use crate::{SCREEN_W, SCREEN_H};

/// Wrapper avec framebuffer complet en RAM (150 Ko).
///
/// Toutes les opérations de dessin écrivent en RAM uniquement.
/// Appelle [`flush`](St7789vBuffered::flush) une fois par frame pour envoyer
/// le résultat à l'écran en une seule passe SPI : zéro clignotement.
///
/// # Mémoire requise
/// 240 × 320 × 2 = **153 600 octets**. Nécessite un MCU avec suffisamment de RAM
/// (RP2350 : 520 Ko ✓, RP2040 : 264 Ko ✓, STM32F103 : 20 Ko ✗).
pub struct St7789vBuffered<SPI, DC, RST = NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    inner: St7789v<SPI, DC, RST>,
    fb: [u8; SCREEN_W as usize * SCREEN_H as usize * 2],
}

impl<SPI, DC, RST> St7789vBuffered<SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Crée un wrapper framebuffer à partir d'un pilote déjà initialisé.
    pub fn new(driver: St7789v<SPI, DC, RST>) -> Self {
        Self {
            inner: driver,
            fb: [0u8; SCREEN_W as usize * SCREEN_H as usize * 2],
        }
    }

    /// Accès au pilote sous-jacent (pour `init`, `set_orientation`, etc.).
    pub fn driver(&mut self) -> &mut St7789v<SPI, DC, RST> {
        &mut self.inner
    }

    // ── Primitives RAM ────────────────────────────────────────────────────────

    /// Écrit un pixel dans le framebuffer (rien n'est envoyé à l'écran).
    #[inline]
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Color) {
        if x >= SCREEN_W || y >= SCREEN_H { return; }
        let idx = (y as usize * SCREEN_W as usize + x as usize) * 2;
        let [hi, lo] = color.to_be_bytes();
        self.fb[idx]     = hi;
        self.fb[idx + 1] = lo;
    }

    /// Remplit tout l'écran dans le framebuffer.
    pub fn fill_screen_buf(&mut self, color: Color) {
        self.fill_rect_buf(0, 0, SCREEN_W - 1, SCREEN_H - 1, color);
    }

    /// Remplit un rectangle dans le framebuffer.
    pub fn fill_rect_buf(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
        let [hi, lo] = color.to_be_bytes();
        for y in y0..=y1.min(SCREEN_H - 1) {
            let row = y as usize * SCREEN_W as usize;
            for x in x0..=x1.min(SCREEN_W - 1) {
                let idx = (row + x as usize) * 2;
                self.fb[idx]     = hi;
                self.fb[idx + 1] = lo;
            }
        }
    }

    /// Dessine le contour d'un rectangle dans le framebuffer.
    pub fn draw_rect_buf(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
        for x in x0..=x1 { self.set_pixel(x, y0, color); self.set_pixel(x, y1, color); }
        for y in y0..=y1 { self.set_pixel(x0, y, color); self.set_pixel(x1, y, color); }
    }

    /// Écrit une chaîne ASCII dans le framebuffer via `set_pixel`.
    ///
    /// Retourne la coordonnée x après le dernier caractère.
    pub fn draw_str_buf(
        &mut self,
        mut x: u16, y: u16,
        text: &[u8],
        fg: Color, bg: Color,
    ) -> u16 {
        for &c in text {
            if x + 5 >= SCREEN_W { break; }
            if let Some(idx) = font::char_to_glyph(c) {
                x = self.draw_char_buf(x, y, idx, fg, bg);
            } else {
                x = x.saturating_add(6);
            }
        }
        x
    }

    /// Écrit un glyphe 5×7 dans le framebuffer.
    pub fn draw_char_buf(
        &mut self,
        x: u16, y: u16,
        glyph_idx: usize,
        fg: Color, bg: Color,
    ) -> u16 {
        for ligne in 0..7u16 {
            for col in 0..5u16 {
                let allume = (FONT[glyph_idx][col as usize] >> ligne) & 1 == 1;
                self.set_pixel(x + col, y + ligne, if allume { fg } else { bg });
            }
        }
        x + 6
    }

    /// Écrit un entier signé 16 bits dans le framebuffer.
    pub fn draw_i16_buf(&mut self, mut x: u16, y: u16, val: i16, fg: Color, bg: Color) -> u16 {
        if val < 0 {
            x = self.draw_char_buf(x, y, 10, fg, bg); // '-'
        }
        let mut n = val.unsigned_abs();
        let mut chiffres = [0u8; 5];
        let mut compte = 0usize;
        loop {
            chiffres[compte] = (n % 10) as u8;
            n /= 10;
            compte += 1;
            if n == 0 { break; }
        }
        for i in (0..compte).rev() {
            x = self.draw_char_buf(x, y, chiffres[i] as usize, fg, bg);
        }
        x
    }

    // ── Flush ─────────────────────────────────────────────────────────────────

    /// Envoie le framebuffer complet à l'écran en une seule passe SPI.
    ///
    /// À appeler **une fois par frame**, après avoir tout dessiné en RAM.
    /// Aucun clignotement car l'écran reçoit les pixels déjà composés.
    pub async fn flush(&mut self) -> Result<(), SPI::Error> {
        self.inner.set_window_only(0, 0, SCREEN_W - 1, SCREEN_H - 1).await?;
        self.inner.write_cmd(cmd::RAMWR).await?;

        const CHUNK: usize = 512;
        let _ = self.inner.dc.set_high();
        for chunk in self.fb.chunks(CHUNK) {
            self.inner.spi.write(chunk).await?;
        }
        Ok(())
    }

    /// Écrit un entier non signé 32 bits dans le framebuffer.
    pub fn draw_u32_buf(&mut self, mut x: u16, y: u16, val: u32, fg: Color, bg: Color) -> u16 {
        let mut n = val;
        let mut chiffres = [0u8; 10];
        let mut compte = 0usize;
        loop {
            chiffres[compte] = (n % 10) as u8;
            n /= 10;
            compte += 1;
            if n == 0 { break; }
        }
        for i in (0..compte).rev() {
            x = self.draw_char_buf(x, y, chiffres[i] as usize, fg, bg);
        }
        x
    }

    /// Dessine un glyphe 5×7 agrandi d'un facteur `scale` dans le framebuffer.
    pub fn draw_char_scaled_buf(
        &mut self,
        x: u16, y: u16,
        glyph_idx: usize,
        fg: Color, bg: Color,
        scale: u8,
    ) -> u16 {
        let s = scale as u16;
        for ligne in 0..7u16 {
            for col in 0..5u16 {
                let allume = (FONT[glyph_idx][col as usize] >> ligne) & 1 == 1;
                for sy in 0..s {
                    for sx in 0..s {
                        self.set_pixel(x + col * s + sx, y + ligne * s + sy,
                            if allume { fg } else { bg });
                    }
                }
            }
        }
        x + 6 * s
    }

    /// Affiche une chaîne d'octets ASCII mise à l'échelle dans le framebuffer.
    pub fn draw_str_scaled_buf(
        &mut self,
        mut x: u16, y: u16,
        text: &[u8],
        fg: Color, bg: Color,
        scale: u8,
    ) -> u16 {
        let s = scale as u16;
        for &c in text {
            if x + 5 * s >= SCREEN_W { break; }
            if let Some(idx) = font::char_to_glyph(c) {
                x = self.draw_char_scaled_buf(x, y, idx, fg, bg, scale);
            } else {
                x = x.saturating_add(6 * s);
            }
        }
        x
    }
}