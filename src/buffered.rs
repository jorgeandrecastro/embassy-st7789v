// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Full in-memory framebuffer wrapper for the ST7789V.

use embedded_hal_async::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

use crate::cmd;
use crate::color::Color;
use crate::font::{self, FONT};
use crate::nopin::NoPin;
use crate::driver::St7789v;
use crate::{SCREEN_W, SCREEN_H};

/// Wrapper with a full in-memory framebuffer (150 KiB).
///
/// All drawing operations write to RAM only.
/// Call [`flush`](St7789vBuffered::flush) once per frame to send
/// the result to the display in one SPI transfer, avoiding flicker.
///
/// # Memory requirement
/// 240 × 320 × 2 = **153,600 bytes**. Requires an MCU with enough RAM
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
    /// Creates a framebuffer wrapper from a driver.
    pub fn new(driver: St7789v<SPI, DC, RST>) -> Self {
        Self {
            inner: driver,
            fb: [0u8; SCREEN_W as usize * SCREEN_H as usize * 2],
        }
    }

    /// Accesses the underlying driver (for `init`, `set_orientation`, etc.).
    pub fn driver(&mut self) -> &mut St7789v<SPI, DC, RST> {
        &mut self.inner
    }

    // ── Primitives RAM ────────────────────────────────────────────────────────

    /// Writes a pixel to the framebuffer (nothing is sent to the display).
    #[inline]
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Color) {
        if x >= SCREEN_W || y >= SCREEN_H { return; }
        let idx = (y as usize * SCREEN_W as usize + x as usize) * 2;
        let [hi, lo] = color.to_be_bytes();
        self.fb[idx]     = hi;
        self.fb[idx + 1] = lo;
    }

    /// Fills the framebuffer with a color.
    pub fn fill_screen_buf(&mut self, color: Color) {
        self.fill_rect_buf(0, 0, SCREEN_W - 1, SCREEN_H - 1, color);
    }

    /// Fills a rectangle in the framebuffer.
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

    /// Draws a rectangle outline in the framebuffer.
    pub fn draw_rect_buf(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
        for x in x0..=x1 { self.set_pixel(x, y0, color); self.set_pixel(x, y1, color); }
        for y in y0..=y1 { self.set_pixel(x0, y, color); self.set_pixel(x1, y, color); }
    }

    /// Writes an ASCII string to the framebuffer via `set_pixel`.
    ///
    /// Returns the x coordinate after the last character.
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

    /// Writes a 5×7 glyph to the framebuffer.
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

    /// Writes a signed 16-bit integer to the framebuffer.
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

    /// Sends the entire framebuffer to the display in one SPI transfer.
    ///
    /// Call **once per frame**, after drawing everything in RAM.
    /// This avoids flicker because the display receives the already-composited pixels.
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

    /// Writes an unsigned 32-bit integer to the framebuffer.
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

    /// Draws a 5×7 glyph enlarged by a `scale` factor in the framebuffer.
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

    /// Writes a scaled ASCII byte string to the framebuffer.
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