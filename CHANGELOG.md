# Changelog

All notable changes to this project are documented here.
This format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [0.6.1] — 2026-11-18

### Added
- `time-riscv` Cargo feature to select the custom Embassy time dependency for the RP2350 Hazard3 RISC-V core.
- Documentation for the RISC-V time backend in the README.

### Fixed
- Use the `embassy_time` library name for the RISC-V time dependency.

---

## [0.6.0] — 2026-08-03

### Added
- `St7789v::blit_u16`: sends an external RGB565 buffer to a rectangular window in one continuous SPI transfer.
- README documentation and a usage example for the new `blit_u16` method.

### Changed
- Updated the README for version 0.6.0.
- Aligned installation examples and version references with the 0.6 release.

---

## [0.5.0] — 2026-07-30

### Changed
- Updated the README for version 0.5.0.
- Aligned installation examples and version references with the 0.5 release.
- Documented the mathematical and Greek symbols added to the built-in bitmap font: `π`, `°`, `±`, `×`, `÷`, `√`, `∞`, `≈`, `≤`, and `≥`.
- Added a minimal example using the constants exported by the crate (`PI`, `DEGREE`, `PLUS_MINUS`, etc.) with `draw_str`.
- No functional changes to the existing drawing API.

---

## [0.4.0] — 2026-07-28

### Changed
- Refactored the driver into separate modules: `cmd`, `font`, `nopin`, `color`, `driver`, and `buffered`.
- Re-exported the main types from the crate root (`Color`, `NoPin`, `St7789v`, and `St7789vBuffered`) to simplify usage.
- Improved the internal code structure without changing the existing drawing API.

---

## [0.3.0] — 2026-06-06

### Added
Scaled text in the framebuffer.

**`St7789vBuffered::draw_char_scaled_buf`** draws a 5×7 glyph enlarged by an integer `scale: u8` factor in the framebuffer. Each source pixel is rendered as a `scale × scale` square.

**`St7789vBuffered::draw_str_scaled_buf`** writes a scaled ASCII string to the framebuffer. Character spacing scales with `scale`, and drawing stops automatically at the right edge of the display.

These methods complement `draw_char_scaled` and `draw_str_scaled`, which are available on `St7789v` (the direct driver), by providing the same behavior in flicker-free framebuffer mode.

Example:

```rust
// Double size (10×14 px per character)
display.draw_str_scaled_buf(8, 10, b"HELLO", Color::WHITE, Color::BLACK, 2);

// Triple size (15×21 px per character)
display.draw_str_scaled_buf(8, 40, b"OK", Color::GREEN, Color::BLACK, 3);

// Regular size — identical to draw_str_buf
display.draw_str_scaled_buf(8, 80, b"details", Color::CYAN, Color::BLACK, 1);
```

Note: `scale` is an integer (`u8`). Fractional values (for example, 1.5) are not
supported because each bitmap-font pixel must map to a whole square of screen
pixels. Use a larger source font for intermediate sizes.

---

## [0.2.0] — 2026

### Added
- `St7789vBuffered`: a full 240×320 RGB565 framebuffer wrapper (153,600 bytes)
  - `fill_screen_buf` — fills the screen in RAM
  - `fill_rect_buf` — fills a rectangle in RAM
  - `draw_rect_buf` — draws a rectangle outline in RAM
  - `set_pixel` — writes one pixel in RAM
  - `draw_char_buf` — draws a 5×7 glyph in RAM
  - `draw_str_buf` — writes an ASCII string in RAM
  - `draw_i16_buf` — writes a signed 16-bit integer in RAM
  - `draw_u32_buf` — writes an unsigned 32-bit integer in RAM
  - `driver()` — access to the underlying driver (`init`, `set_orientation`, etc.)
  - `flush()` — sends the entire framebuffer to the display in one SPI transfer
- Changed `write_cmd` and `set_window_only` visibility to `pub(crate)` for `St7789vBuffered`.

### Changed
- No changes to the existing API; fully backward compatible.

---

## [0.1.0] — 2026

### Added
- Asynchronous `no_std` driver for the ST7789V 240×320 display over SPI (Embassy).
- RGB565 `Color` type with named constants (`BLACK`, `WHITE`, `RED`, `GREEN`,
  `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`, `ORANGE`, `GRAY`).
- `Color::rgb` and `Color::rgb8` for custom colors.
- `St7789v::new` (with RST) and `St7789v::new_no_rst` (without RST).
- `NoPin` dummy pin for optional RST.
- `init` — complete ST7789V register initialization sequence.
- `fill_screen`, `fill_rect`, and `draw_rect`.
- `draw_pixel`, `draw_hline`, and `draw_vline`.
- Built-in 5×7 bitmap font (ASCII, digits, and common symbols).
- `draw_char` and `draw_str`.
- `draw_i16`, `draw_u32`, and `draw_f32` (including NaN / ±Inf handling).
- `draw_char_scaled` and `draw_str_scaled` — text enlarged by an integer factor.
- `draw_bitmap` — MSB-first 1-bit bitmap rendering.
- `set_orientation` — MADCTL control (portrait, landscape, and flipped modes).
- `set_invert` — color inversion (required by some IPS panels).
