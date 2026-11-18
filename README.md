# embassy-st7789v

[![Crates.io](https://img.shields.io/crates/v/embassy-st7789v.svg)](https://crates.io/crates/embassy-st7789v)
[![Docs.rs](https://docs.rs/embassy-st7789v/badge.svg)](https://docs.rs/embassy-st7789v)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)

An asynchronous `no_std` driver for the **ST7789V** 240×320 TFT LCD over SPI,
based on [Embassy](https://embassy.dev).

**No unsafe code.**

## Version 0.6.1

### Added

- The `time-riscv` Cargo feature selects a custom Embassy time dependency for the
  RP2350 Hazard3 RISC-V core; see [Installation](#rp2350-hazard3-risc-v-time-backend).

---

## Features

- RGB565 colors with named constants (`BLACK`, `RED`, `CYAN`, etc.)
- `fill_screen`, `fill_rect`, and `draw_rect`
- `draw_pixel`, `draw_hline`, and `draw_vline`
- `blit_u16` to quickly write an RGB565 buffer to a rectangular area
- Built-in 5×7 bitmap font: ASCII letters, digits, common symbols, and mathematical/Greek symbols (`PI`, `DEGREE`, `PLUS_MINUS`, `TIMES`, `DIVIDE`, `SQRT`, `INFINITY`, `APPROX`, `LE`, `GE`)
- `draw_str`, `draw_i16`, `draw_u32`, and `draw_f32`
- Scaled text: `draw_char_scaled`, `draw_str_scaled`, `draw_str_scaled_buf`, and `draw_char_scaled_buf`
- 1-bit bitmap rendering with `draw_bitmap`
- Hardware and software reset
- Orientation (`MADCTL`) and inversion control
- Optional RST pin: use `new_no_rst` when RESET is tied high
- Zero allocation and `forbid(unsafe_code)`
- Full 240×320 RGB565 framebuffer with `St7789vBuffered` for flicker-free updates

---

## Supported hardware

Any 4-wire SPI panel driven by an ST7789V (DC + CS + SCL + SDA).
Tested with a 2.0" IPS TFT module (240×320) and an RP2350 (Raspberry Pi Pico 2).

---

## Wiring example — RP2350

| Pin | Signal | RP2350 | Physical pin |
|-----|--------|--------|--------------|
| 1   | GND    | GND    |              |
| 2   | VDD    | 3.3V   |              |
| 3   | DC     | GPIO 16 | Pin 21       |
| 4   | CS     | GPIO 20 | Pin 26       |
| 5   | SCL    | GPIO 18 | Pin 24       |
| 6   | SDA    | GPIO 19 | Pin 25       |
| 7   | RESET  | GPIO 17 | Pin 22       |

---

## Installation

For ARM/Cortex-M targets, the official Embassy time backend is enabled by default:

```toml
[dependencies]
embassy-st7789v = "0.6.1"
embassy-time = "0.5"
embedded-hal = "1.0"
embedded-hal-async = "1.0"
```

### RP2350 Hazard3 RISC-V time backend

The `time-riscv` feature selects the `embassy-time-riscv` dependency
for the RP2350 Hazard3 RISC-V core. The two time backend features are mutually
exclusive, so selecting `time-riscv` requires disabling the default `time-upstream`
feature.

```toml
[dependencies]
embassy-st7789v = { version = "0.6.1", default-features = false, features = ["time-riscv"] }
embedded-hal = "1.0"
embedded-hal-async = "1.0"
```

The current manifest does not include a local Cargo patch. Any local `[patch.crates-io]`
configuration belongs in the root manifest of the project being built.

---

## Usage

### Minimal initialization (RP2350 + Embassy)

```rust
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{Config as SpiConfig, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_st7789v::{Color, St7789v};
use static_cell::StaticCell;

type SpiBus = Mutex<NoopRawMutex, Spi<'static, embassy_rp::peripherals::SPI0, embassy_rp::spi::Async>>;
static SPI_BUS: StaticCell<SpiBus> = StaticCell::new();

// In your async main:
let mut spi_cfg = SpiConfig::default();
spi_cfg.frequency = 40_000_000;

let spi = Spi::new_txonly(p.SPI0, p.PIN_18, p.PIN_19, p.DMA_CH0, Irqs, spi_cfg);
let spi_bus = SPI_BUS.init(Mutex::new(spi));

let cs = Output::new(p.PIN_20, Level::High);
let dc = Output::new(p.PIN_16, Level::Low);
let rst = Output::new(p.PIN_17, Level::High);

let spi_dev = SpiDevice::new(spi_bus, cs);
let mut display = St7789v::new(spi_dev, dc, rst);

display.init().await.unwrap();
```

### Drawing

```rust
// Fill the screen
display.fill_screen(Color::BLACK).await.unwrap();

// Colored rectangles
display.fill_rect(0, 0, 79, 79, Color::RED).await.unwrap();
display.fill_rect(80, 0, 159, 79, Color::GREEN).await.unwrap();

// Rectangle outline
display.draw_rect(10, 10, 230, 310, Color::WHITE).await.unwrap();

// Lines
display.draw_hline(0, 160, 240, Color::CYAN).await.unwrap();
display.draw_vline(120, 0, 320, Color::MAGENTA).await.unwrap();

// Single pixel
display.draw_pixel(120, 160, Color::YELLOW).await.unwrap();

// RGB565 buffer to a rectangular area (2×2 pixels)
let pixels = [0xF800u16, 0x07E0u16, 0x001Fu16, 0xFFFFu16];
display.blit_u16(0, 0, 1, 1, &pixels).await.unwrap();
```

### Regular-size text

```rust
// String (the font is case-insensitive)
display.draw_str(8, 10, b"Hello ST7789V", Color::WHITE, Color::BLACK).await.unwrap();

// Signed integer
display.draw_str(8, 20, b"Temp: ", Color::WHITE, Color::BLACK).await.unwrap();
display.draw_i16(56, 20, -42, Color::YELLOW, Color::BLACK).await.unwrap();

// Unsigned integer
display.draw_str(8, 30, b"Counter: ", Color::WHITE, Color::BLACK).await.unwrap();
display.draw_u32(74, 30, 123456, Color::CYAN, Color::BLACK).await.unwrap();
```

### Mathematical symbols

The built-in bitmap font supports several mathematical and Greek symbols, available
through extended codes exported by the crate: `PI`, `DEGREE`, `PLUS_MINUS`, `TIMES`,
`DIVIDE`, `SQRT`, `INFINITY`, `APPROX`, `LE`, `GE`, and more.

```rust
use embassy_st7789v::{Color, PI, DEGREE, PLUS_MINUS};

// Minimal example: display "π", "°", and "±"
display.draw_str(8, 10, b"PI: ", Color::WHITE, Color::BLACK).await.unwrap();
display.draw_str(32, 10, &[PI], Color::CYAN, Color::BLACK).await.unwrap();
display.draw_str(8, 20, &[b'3', b'0', DEGREE, b'C'], Color::YELLOW, Color::BLACK).await.unwrap();
display.draw_str(8, 30, &[b'1', b' ', PLUS_MINUS, b' ', b'1'], Color::GREEN, Color::BLACK).await.unwrap();
```

### Scaled text

```rust
// Double-size title (each character: 10×14 px)
display.draw_str_scaled(8, 10, b"ERROR", Color::RED, Color::BLACK, 2).await.unwrap();

// Triple-size title (each character: 15×21 px)
display.draw_str_scaled(8, 40, b"OK", Color::GREEN, Color::BLACK, 3).await.unwrap();

// Regular-size text (scale 1 is identical to draw_str)
display.draw_str_scaled(8, 80, b"details", Color::WHITE, Color::BLACK, 1).await.unwrap();

// Triple-size title (each character: 15×21 px) with the framebuffer
display.draw_str_scaled_buf(2, 270, b"NO PATH!", Color::RED, Color::BLACK, 3);
```

### Floating-point values

```rust
// Displays "-3.14"
display.draw_f32(8, 100, -3.14, 2, Color::YELLOW, Color::BLACK).await.unwrap();

// Displays "23.5" (1 decimal place)
display.draw_f32(8, 110, 23.456, 1, Color::CYAN, Color::BLACK).await.unwrap();

// Displays "100" (no decimal places)
display.draw_f32(8, 120, 100.0, 0, Color::WHITE, Color::BLACK).await.unwrap();

// Special cases: displays "NaN", "+Inf", or "-Inf"
display.draw_f32(8, 130, f32::NAN, 2, Color::RED, Color::BLACK).await.unwrap();
```

### Colors

```rust
// Predefined constants
let color = Color::RED;

// Raw RGB565 (r: 0–31, g: 0–63, b: 0–31)
let color = Color::rgb(31, 40, 0); // orange

// From 8-bit RGB (reduced to RGB565)
let color = Color::rgb8(0xFF, 0x80, 0x00); // orange
```

### Orientation

```rust
display.set_orientation(0x00).await.unwrap(); // portrait (default)
display.set_orientation(0x60).await.unwrap(); // landscape
display.set_orientation(0xC0).await.unwrap(); // upside-down portrait
display.set_orientation(0xA0).await.unwrap(); // upside-down landscape
```

### Inversion

IPS panels with ST7789V typically require inversion, which is enabled by default
after `init`. If the colors look inverted:

```rust
display.set_invert(false).await.unwrap();
```

---

### Framebuffer (flicker-free)

To avoid flicker, use `St7789vBuffered`: drawing operations update RAM, and `flush`
sends the complete frame to the display.

Memory required: 240 × 320 × 2 = **150 KiB** (RP2350 ✓, RP2040 ✓, STM32F103 ✗).

```rust
let mut display = St7789vBuffered::new(St7789v::new(spi_dev, dc, rst));
display.driver().init().await.unwrap();

loop {
    display.fill_screen_buf(Color::BLACK);
    display.draw_str_buf(8, 10, b"HELLO", Color::WHITE, Color::BLACK);
    display.draw_u32_buf(8, 20, counter, Color::YELLOW, Color::BLACK);
    display.flush().await.unwrap(); // one transfer, no flicker
}
```

| Method | Direct equivalent |
|---|---|
| `fill_screen_buf` | `fill_screen` |
| `fill_rect_buf` | `fill_rect` |
| `draw_rect_buf` | `draw_rect` |
| `draw_str_buf` | `draw_str` |
| `draw_char_buf` | `draw_char` |
| `draw_i16_buf` | `draw_i16` |
| `draw_u32_buf` | `draw_u32` |
| `flush` | — (SPI transfer) |
| `draw_char_scaled_buf` | `draw_char_scaled` |
| `draw_str_scaled_buf` | `draw_str_scaled` |

---

## Troubleshooting

| Symptom | Likely cause | Solution |
|---------|--------------|----------|
| Gray screen with backlight on | SPI is not reaching the display | Check SCL/SDA/CS/DC |
| Inverted colors | Incorrect inversion setting | Call `set_invert(false)` after `init` |
| Incorrect orientation | Wrong MADCTL setting | Try `set_orientation(0x60)`, `0xC0`, or `0xA0` |
| Corrupted display | SPI clock is too fast | Reduce the frequency to `10_000_000` for debugging |
| Visible flicker | Drawing directly without a framebuffer | Use `St7789vBuffered` and `flush` |

---

## Cargo.toml

```toml
[dependencies]
embassy-st7789v = "0.6.1"
embassy-time = "0.5"
embedded-hal = "1.0"
embedded-hal-async = "1.0"
```

For the RP2350 Hazard3 RISC-V backend, use the `time-riscv` configuration shown in
[Installation](#rp2350-hazard3-risc-v-time-backend) instead.

---

## License

Copyright (C) 2026 Jorge Andre Castro

Licensed under the [GNU General Public License v2.0 or later](LICENSE).
