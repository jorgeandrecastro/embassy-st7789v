// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Asynchronous ST7789V driver (without a framebuffer).

use embassy_time::Timer;
use embedded_hal_async::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

use crate::cmd;
use crate::color::Color;
use crate::font::{self, FONT};
use crate::nopin::NoPin;
use crate::{SCREEN_W, SCREEN_H};

/// Asynchronous driver for the ST7789V 240×320 TFT LCD.
///
/// Generic over any [`SpiDevice`], a data/command [`OutputPin`] (`DC`),
/// and an optional reset [`OutputPin`] (`RST`, [`NoPin`] by default).
///
/// # Construction
///
/// Use [`St7789v::new`] if you have a hardware RST pin, or
/// [`St7789v::new_no_rst`] if RESET is tied high or absent.
///
/// Always call [`St7789v::init`] once before drawing.
///
/// # Exigences SPI
///
/// - Mode 0 (CPOL=0, CPHA=0)
/// - MSB en premier
/// - Up to 40 MHz (start at 10 MHz for debugging)
/// - TX only: MISO is not used
pub struct St7789v<SPI, DC, RST = NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    pub(crate) spi: SPI,
    pub(crate) dc: DC,
    rst: RST,
}

// ── Constructor (without RST) ─────────────────────────────────────────────────

impl<SPI, DC> St7789v<SPI, DC, NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    /// Creates a driver without a hardware reset pin.
    ///
    /// The software reset in [`init`](St7789v::init) will still run, but
    /// the RESET pin must be held high externally.
    pub fn new_no_rst(spi: SPI, dc: DC) -> Self {
        Self { spi, dc, rst: NoPin }
    }
}

// ── Constructor (with RST) ────────────────────────────────────────────────────

impl<SPI, DC, RST> St7789v<SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a driver with a hardware reset pin.
    ///
    /// `rst` will be driven low and then high during [`init`](St7789v::init).
    pub fn new(spi: SPI, dc: DC, rst: RST) -> Self {
        Self { spi, dc, rst }
    }

    // ── Low-level SPI helpers ─────────────────────────────────────────────────

    #[inline]
    pub(crate) async fn write_cmd(&mut self, c: u8) -> Result<(), SPI::Error> {
        let _ = self.dc.set_low();
        self.spi.write(&[c]).await
    }

    #[inline]
    async fn write_data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        let _ = self.dc.set_high();
        self.spi.write(data).await
    }

    #[inline]
    async fn cmd1(&mut self, c: u8, d: u8) -> Result<(), SPI::Error> {
        self.write_cmd(c).await?;
        self.write_data(&[d]).await
    }

    #[inline]
    async fn cmdn(&mut self, c: u8, data: &[u8]) -> Result<(), SPI::Error> {
        self.write_cmd(c).await?;
        self.write_data(data).await
    }

    /// Sets the active pixel window for subsequent `RAMWR` writes.
    #[inline]
    pub(crate) async fn set_window_only(
        &mut self,
        x0: u16, y0: u16,
        x1: u16, y1: u16,
    ) -> Result<(), SPI::Error> {
        self.write_cmd(cmd::CASET).await?;
        self.write_data(&[
            (x0 >> 8) as u8, x0 as u8,
            (x1 >> 8) as u8, x1 as u8,
        ]).await?;
        self.write_cmd(cmd::RASET).await?;
        self.write_data(&[
            (y0 >> 8) as u8, y0 as u8,
            (y1 >> 8) as u8, y1 as u8,
        ]).await
    }

    // ── Initialization ────────────────────────────────────────────────────────

    /// Initializes the display.
    ///
    /// Must be called once after power-on, before any drawing operation.
    /// Performs a hardware reset (if an RST pin is provided), a software reset,
    /// and then sends the complete register initialization sequence.
    ///
    /// # Errors
    ///
    /// Returns the SPI device error if communication fails.
    #[must_use = "check the result — a failed init leaves the display in an undefined state"]
    pub async fn init(&mut self) -> Result<(), SPI::Error> {
        // Hardware reset
        let _ = self.rst.set_low();
        Timer::after_millis(10).await;
        let _ = self.rst.set_high();
        Timer::after_millis(120).await;

        // Software reset
        self.write_cmd(cmd::SWRESET).await?;
        Timer::after_millis(150).await;

        // Exit sleep mode
        self.write_cmd(cmd::SLPOUT).await?;
        Timer::after_millis(10).await;

        // Pixel format: RGB565
        self.cmd1(cmd::COLMOD, 0x55).await?;

        // Porch control
        self.cmdn(cmd::PORCTRL, &[0x0C, 0x0C, 0x00, 0x33, 0x33]).await?;

        // Gate control
        self.cmd1(cmd::GCTRL, 0x35).await?;

        // VCOM voltage
        self.cmd1(cmd::VCOMS, 0x19).await?;

        // LCM control
        self.cmd1(cmd::LCMCTRL, 0x2C).await?;

        // Enable VDV/VRH
        self.cmdn(cmd::VDVVRHEN, &[0x01, 0xFF]).await?;

        // VRH setting
        self.cmd1(cmd::VRHS, 0x12).await?;

        // VDV setting
        self.cmd1(cmd::VDVS, 0x20).await?;

        // Refresh rate: 60 Hz
        self.cmd1(cmd::FRCTRL2, 0x0F).await?;

        // Power control 1
        self.cmdn(cmd::PWCTRL1, &[0xA4, 0xA1]).await?;

        // Memory access control: portrait, origin at the top left
        // If the display is blank or incorrectly oriented, try:
        // 0x60 (landscape), 0xC0 (upside-down portrait), 0xA0 (upside-down landscape)
        self.cmd1(cmd::MADCTL, 0x00).await?;

        // Positive gamma
        self.cmdn(cmd::PVGAMCTRL, &[
            0xD0, 0x04, 0x0D, 0x11, 0x13, 0x2B, 0x3F, 0x54,
            0x4C, 0x18, 0x0D, 0x0B, 0x1F, 0x23,
        ]).await?;

        // Negative gamma
        self.cmdn(cmd::NVGAMCTRL, &[
            0xD0, 0x04, 0x0C, 0x11, 0x13, 0x2C, 0x3F, 0x44,
            0x51, 0x2F, 0x1F, 0x1F, 0x20, 0x23,
        ]).await?;

        // Normal display mode
        self.write_cmd(cmd::NORON).await?;

        // Inversion enabled: required by IPS panels with ST7789V
        self.write_cmd(cmd::INVON).await?;

        // Turn on the display
        self.write_cmd(cmd::DISPON).await?;
        Timer::after_millis(10).await;

        Ok(())
    }

    // ── Drawing primitives ────────────────────────────────────────────────────

    /// Fills the entire 240×320 display with `color`.
    pub async fn fill_screen(&mut self, color: Color) -> Result<(), SPI::Error> {
        self.fill_rect(0, 0, SCREEN_W - 1, SCREEN_H - 1, color).await
    }

    /// Fills the rectangle from `(x0, y0)` through `(x1, y1)` inclusive with `color`.
    ///
    /// Coordinates are clamped to the display bounds.
    pub async fn fill_rect(
        &mut self,
        x0: u16, y0: u16,
        x1: u16, y1: u16,
        color: Color,
    ) -> Result<(), SPI::Error> {
        let x0 = x0.min(SCREEN_W - 1);
        let y0 = y0.min(SCREEN_H - 1);
        let x1 = x1.min(SCREEN_W - 1);
        let y1 = y1.min(SCREEN_H - 1);

        self.set_window_only(x0, y0, x1, y1).await?;
        self.write_cmd(cmd::RAMWR).await?;

        let [hi, lo] = color.to_be_bytes();
        let w = (x1 - x0 + 1) as usize;
        let h = (y1 - y0 + 1) as usize;

        // Send 64-pixel chunks to avoid large stack buffers.
        const BUF_PIX: usize = 64;
        let mut buf = [0u8; BUF_PIX * 2];
        for i in 0..BUF_PIX {
            buf[i * 2]     = hi;
            buf[i * 2 + 1] = lo;
        }

        let mut restant = w * h;
        while restant > 0 {
            let bloc = restant.min(BUF_PIX);
            let _ = self.dc.set_high();
            self.spi.write(&buf[..bloc * 2]).await?;
            restant -= bloc;
        }
        Ok(())
    }

    /// Draws a single pixel at `(x, y)`.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    pub async fn draw_pixel(
        &mut self,
        x: u16, y: u16,
        color: Color,
    ) -> Result<(), SPI::Error> {
        if x >= SCREEN_W || y >= SCREEN_H { return Ok(()); }
        self.set_window_only(x, y, x, y).await?;
        self.write_cmd(cmd::RAMWR).await?;
        self.write_data(&color.to_be_bytes()).await
    }

    /// Draws a horizontal line of length `w` starting at `(x, y)`.
    pub async fn draw_hline(
        &mut self,
        x: u16, y: u16,
        w: u16,
        color: Color,
    ) -> Result<(), SPI::Error> {
        if y >= SCREEN_H || x >= SCREEN_W { return Ok(()); }
        let x1 = (x + w - 1).min(SCREEN_W - 1);
        self.fill_rect(x, y, x1, y, color).await
    }

    /// Draws a vertical line of height `h` starting at `(x, y)`.
    pub async fn draw_vline(
        &mut self,
        x: u16, y: u16,
        h: u16,
        color: Color,
    ) -> Result<(), SPI::Error> {
        if x >= SCREEN_W || y >= SCREEN_H { return Ok(()); }
        let y1 = (y + h - 1).min(SCREEN_H - 1);
        self.fill_rect(x, y, x, y1, color).await
    }

    /// Draws the outline of the rectangle from `(x0, y0)` to `(x1, y1)`.
    pub async fn draw_rect(
        &mut self,
        x0: u16, y0: u16,
        x1: u16, y1: u16,
        color: Color,
    ) -> Result<(), SPI::Error> {
        let w = x1.saturating_sub(x0) + 1;
        let h = y1.saturating_sub(y0) + 1;
        self.draw_hline(x0, y0, w, color).await?;
        self.draw_hline(x0, y1, w, color).await?;
        self.draw_vline(x0, y0, h, color).await?;
        self.draw_vline(x1, y0, h, color).await
    }

    /// Draws a compressed 1-bit bitmap at `(x, y)`.
    ///
    /// Pixels set to `1` are drawn in `fg`; pixels set to `0` are drawn in `bg`.
    /// Bits are packed MSB-first, with `ceil(w/8)` bytes per row.
    ///
    /// # Parameters
    ///
    /// - `w`, `h`: dimensions in pixels
    /// - `data`: packed bitmap, `ceil(w/8) * h` bytes
    /// - `fg`: foreground color (bits set to 1)
    /// - `bg`: background color (bits set to 0)
    pub async fn draw_bitmap(
        &mut self,
        x: u16, y: u16,
        w: u16, h: u16,
        data: &[u8],
        fg: Color, bg: Color,
    ) -> Result<(), SPI::Error> {
        if x >= SCREEN_W || y >= SCREEN_H { return Ok(()); }
        let x1 = (x + w - 1).min(SCREEN_W - 1);
        let y1 = (y + h - 1).min(SCREEN_H - 1);

        self.set_window_only(x, y, x1, y1).await?;
        self.write_cmd(cmd::RAMWR).await?;

        let stride = ((w + 7) / 8) as usize;
        let [fh, fl] = fg.to_be_bytes();
        let [bh, bl] = bg.to_be_bytes();

        for ligne in 0..h as usize {
            for col in 0..w as usize {
                let idx_octet = ligne * stride + col / 8;
                let bit = 7 - (col % 8);
                let allume = idx_octet < data.len() && (data[idx_octet] >> bit) & 1 == 1;
                let _ = self.dc.set_high();
                if allume {
                    self.spi.write(&[fh, fl]).await?;
                } else {
                    self.spi.write(&[bh, bl]).await?;
                }
            }
        }
        Ok(())
    }

    // ── Font / text ───────────────────────────────────────────────────────────

    /// Draws a single 5×7 glyph at `(x, y)`.
    ///
    /// The glyph occupies a 5×7 pixel cell. Use [`draw_str`](Self::draw_str)
    /// to draw text directly.
    ///
    /// Returns the x coordinate immediately after the glyph (`x + 6`,
    /// including one pixel of spacing).
    pub async fn draw_char(
        &mut self,
        x: u16, y: u16,
        glyph_idx: usize,
        fg: Color, bg: Color,
    ) -> Result<u16, SPI::Error> {
        let x1 = (x + 4).min(SCREEN_W - 1);
        let y1 = (y + 6).min(SCREEN_H - 1);

        self.set_window_only(x, y, x1, y1).await?;
        self.write_cmd(cmd::RAMWR).await?;

        let [fh, fl] = fg.to_be_bytes();
        let [bh, bl] = bg.to_be_bytes();

        // 5×7 = 35 pixels × 2 bytes = 70 bytes: one write per glyph.
        let mut buf = [0u8; 70];
        for ligne in 0..7usize {
            for col in 0..5usize {
                let allume = (FONT[glyph_idx][col] >> ligne) & 1 == 1;
                let i = (ligne * 5 + col) * 2;
                if allume {
                    buf[i]     = fh;
                    buf[i + 1] = fl;
                } else {
                    buf[i]     = bh;
                    buf[i + 1] = bl;
                }
            }
        }
        let _ = self.dc.set_high();
        self.spi.write(&buf).await?;

        Ok(x + 6)
    }

    /// Draws an ASCII byte string at `(x, y)`.
    ///
    /// Unsupported characters advance the cursor by 6 pixels without drawing.
    /// Drawing stops when the right edge of the display is reached.
    ///
    /// Returns the x coordinate after the last character.
    ///
    /// # Example
    ///
    /// ```no_run
    /// display.draw_str(8, 20, b"TEMP: ", Color::WHITE, Color::BLACK).await?;
    /// display.draw_i16(56, 20, temperature, Color::YELLOW, Color::BLACK).await?;
    /// ```
    pub async fn draw_str(
        &mut self,
        mut x: u16, y: u16,
        text: &[u8],
        fg: Color, bg: Color,
    ) -> Result<u16, SPI::Error> {
        for &c in text {
            if x + 5 >= SCREEN_W { break; }
            if let Some(idx) = font::char_to_glyph(c) {
                x = self.draw_char(x, y, idx, fg, bg).await?;
            } else {
                x = x.saturating_add(6);
            }
        }
        Ok(x)
    }

    /// Draws a signed 16-bit integer at `(x, y)`.
    ///
    /// A `'-'` glyph is prepended for negative values.
    /// Returns the x coordinate after the last digit.
    pub async fn draw_i16(
        &mut self,
        mut x: u16, y: u16,
        val: i16,
        fg: Color, bg: Color,
    ) -> Result<u16, SPI::Error> {
        if val < 0 {
            x = self.draw_char(x, y, 10, fg, bg).await?; // '-'
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
            x = self.draw_char(x, y, chiffres[i] as usize, fg, bg).await?;
        }
        Ok(x)
    }

    /// Draws an unsigned 32-bit integer at `(x, y)`.
    ///
    /// Returns the x coordinate after the last digit.
    pub async fn draw_u32(
        &mut self,
        mut x: u16, y: u16,
        val: u32,
        fg: Color, bg: Color,
    ) -> Result<u16, SPI::Error> {
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
            x = self.draw_char(x, y, chiffres[i] as usize, fg, bg).await?;
        }
        Ok(x)
    }

    // ── Scaled text ───────────────────────────────────────────────────────────

    /// Draws a 5×7 glyph enlarged by a `scale` factor at `(x, y)`.
    ///
    /// Each font pixel is rendered as a `scale × scale` square.
    /// `scale = 1` is identical to [`draw_char`](Self::draw_char).
    /// `scale = 2` produces a 10×14 pixel glyph, `scale = 3` a 15×21 pixel glyph, etc.
    ///
    /// Returns the x coordinate after the glyph (`x + (6 * scale as u16)`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Large title (scale 3 = 15×21 px per character)
    /// display.draw_char_scaled(10, 10, idx, Color::YELLOW, Color::BLACK, 3).await?;
    /// ```
    pub async fn draw_char_scaled(
        &mut self,
        x: u16, y: u16,
        glyph_idx: usize,
        fg: Color, bg: Color,
        scale: u8,
    ) -> Result<u16, SPI::Error> {
        if scale == 0 { return Ok(x); }
        let s = scale as u16;

        let w = 5 * s;
        let h = 7 * s;

        let x1 = (x + w - 1).min(SCREEN_W - 1);
        let y1 = (y + h - 1).min(SCREEN_H - 1);

        self.set_window_only(x, y, x1, y1).await?;
        self.write_cmd(cmd::RAMWR).await?;

        let [fh, fl] = fg.to_be_bytes();
        let [bh, bl] = bg.to_be_bytes();

        // Scaled horizontal row buffer (max scale=8 → 5×8×2 = 80 bytes).
        let mut ligne_buf = [0u8; 5 * 8 * 2];
        let ligne_len = (5 * scale as usize) * 2;

        for ligne in 0..7usize {
            // Prepare the scaled row pixels.
            for col in 0..5usize {
                let allume = (FONT[glyph_idx][col] >> ligne) & 1 == 1;
                for sx in 0..scale as usize {
                    let i = (col * scale as usize + sx) * 2;
                    if allume {
                        ligne_buf[i]     = fh;
                        ligne_buf[i + 1] = fl;
                    } else {
                        ligne_buf[i]     = bh;
                        ligne_buf[i + 1] = bl;
                    }
                }
            }
            // Repeat the row `scale` times vertically.
            for _ in 0..scale {
                let _ = self.dc.set_high();
                self.spi.write(&ligne_buf[..ligne_len]).await?;
            }
        }

        Ok(x + 6 * s)
    }

    /// Draws a scaled ASCII byte string at `(x, y)`.
    ///
    /// Each character is enlarged by a `scale` factor using
    /// [`draw_char_scaled`](Self::draw_char_scaled).
    /// Character spacing is scaled as well (`scale` pixels).
    ///
    /// Returns the x coordinate after the last character.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Double-size title
    /// display.draw_str_scaled(8, 10, b"ERROR", Color::RED, Color::BLACK, 2).await?;
    ///
    /// // Regular-size text
    /// display.draw_str_scaled(8, 40, b"details here", Color::WHITE, Color::BLACK, 1).await?;
    /// ```
    pub async fn draw_str_scaled(
        &mut self,
        mut x: u16, y: u16,
        text: &[u8],
        fg: Color, bg: Color,
        scale: u8,
    ) -> Result<u16, SPI::Error> {
        if scale == 0 { return Ok(x); }
        let s = scale as u16;
        for &c in text {
            if x + 5 * s >= SCREEN_W { break; }
            if let Some(idx) = font::char_to_glyph(c) {
                x = self.draw_char_scaled(x, y, idx, fg, bg, scale).await?;
            } else {
                x = x.saturating_add(6 * s);
            }
        }
        Ok(x)
    }

    // ── Floating-point display ────────────────────────────────────────────────

    /// Draws an `f32` value at `(x, y)` with `decimales` digits after the decimal point.
    ///
    /// Handles negative values, zero, and non-finite values (`NaN`, `+Inf`, `-Inf`)
    /// with readable strings.
    ///
    /// Returns the x coordinate after the last character drawn.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Draw "-3.14" in yellow
    /// display.draw_f32(8, 60, -3.14, 2, Color::YELLOW, Color::BLACK).await?;
    ///
    /// // Draw "23.5" (1 decimal place)
    /// display.draw_f32(8, 70, 23.456, 1, Color::CYAN, Color::BLACK).await?;
    ///
    /// // Draw "100" (0 decimal places)
    /// display.draw_f32(8, 80, 100.0, 0, Color::WHITE, Color::BLACK).await?;
    /// ```
    pub async fn draw_f32(
        &mut self,
        mut x: u16, y: u16,
        val: f32,
        decimales: u8,
        fg: Color, bg: Color,
    ) -> Result<u16, SPI::Error> {
        // Special cases
        if val.is_nan() {
            return self.draw_str(x, y, b"NaN", fg, bg).await;
        }
        if val.is_infinite() {
            if val > 0.0 {
                return self.draw_str(x, y, b"+Inf", fg, bg).await;
            } else {
                return self.draw_str(x, y, b"-Inf", fg, bg).await;
            }
        }

        // Sign
        let negatif = val < 0.0;
        let mut abs = if negatif { -val } else { val };

        if negatif {
            x = self.draw_char(x, y, 10, fg, bg).await?; // '-'
        }

        // Round to the requested number of decimal places.
        let facteur = {
            let mut f = 1u32;
            for _ in 0..decimales { f *= 10; }
            f
        };
        // Round by adding 0.5 at the last requested position.
        abs += 0.5 / facteur as f32;

        // Integer part
        let entier = abs as u32;
        x = self.draw_u32(x, y, entier, fg, bg).await?;

        // Fractional part
        if decimales > 0 {
            // Decimal point (index 38 = '.')
            x = self.draw_char(x, y, 38, fg, bg).await?;

            // Extract decimal digits, preserving leading zeros.
            let mut frac = abs - entier as f32;
            let mut chiffres = [0u8; 8];
            for i in 0..decimales as usize {
                frac *= 10.0;
                let d = frac as u8;
                chiffres[i] = d;
                frac -= d as f32;
            }
            for i in 0..decimales as usize {
                x = self.draw_char(x, y, chiffres[i] as usize, fg, bg).await?;
            }
        }

        Ok(x)
    }

    // ── Display control ───────────────────────────────────────────────────────

    /// Sets the memory access control register (`MADCTL`).
    ///
    /// Controls display orientation and RGB/BGR order.
    ///
    /// | Valeur | Orientation              |
    /// |--------|--------------------------|
    /// | `0x00` | Portrait (default)       |
    /// | `0x60` | Paysage                  |
    /// | `0xC0` | Upside-down portrait     |
    /// | `0xA0` | Upside-down landscape    |
    pub async fn set_orientation(&mut self, madctl: u8) -> Result<(), SPI::Error> {
        self.cmd1(cmd::MADCTL, madctl).await
    }

    /// Enables or disables color inversion.
    ///
    /// ST7789V IPS panels generally require inversion to be enabled (`true`),
    /// which is the default after [`init`](St7789v::init). Pass `false` if
    /// the colors appear inverted on your panel.
    pub async fn set_invert(&mut self, invert: bool) -> Result<(), SPI::Error> {
        if invert {
            self.write_cmd(cmd::INVON).await
        } else {
            self.write_cmd(cmd::INVOFF).await
        }
    }
    /// Sends an external RGB565 buffer (already composed in RAM by the caller)
    /// to a rectangular window in one continuous SPI transfer.
   ///
   /// Opens the `CASET`/`RASET`/`RAMWR` window **only once**, then streams
   /// all pixels without resetting it between pixels. By contrast, repeated
   /// calls to [`draw_pixel`](Self::draw_pixel) reopen the window for every
   /// pixel and cause visibly gradual drawing.
   ///
   /// `pixels.len()` must equal `(x1 - x0 + 1) * (y1 - y0 + 1)`.
   pub async fn blit_u16(
    &mut self,
    x0: u16, y0: u16,
    x1: u16, y1: u16,
    pixels: &[u16],
   ) -> Result<(), SPI::Error> {
    let x0 = x0.min(SCREEN_W - 1);
    let y0 = y0.min(SCREEN_H - 1);
    let x1 = x1.min(SCREEN_W - 1);
    let y1 = y1.min(SCREEN_H - 1);

    self.set_window_only(x0, y0, x1, y1).await?;
    self.write_cmd(cmd::RAMWR).await?;

    const CHUNK_PIX: usize = 256;
    let mut buf = [0u8; CHUNK_PIX * 2];

    let _ = self.dc.set_high();
    for chunk in pixels.chunks(CHUNK_PIX) {
        for (i, &p) in chunk.iter().enumerate() {
            let [hi, lo] = p.to_be_bytes();
            buf[i * 2]     = hi;
            buf[i * 2 + 1] = lo;
        }
        self.spi.write(&buf[..chunk.len() * 2]).await?;
    }
    Ok(())
   } 
}