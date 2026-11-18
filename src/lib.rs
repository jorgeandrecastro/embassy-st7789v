// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! # embassy-st7789v
//!
//! Asynchronous `no_std` driver for the **ST7789V** 240×320 TFT LCD over SPI,
//! based on [Embassy](https://embassy.dev).
//!
//! ## Features
//!
//! - RGB565 colors with named constants
//! - `fill_screen`, `fill_rect`, `draw_rect`
//! - `draw_pixel`, `draw_hline`, `draw_vline`
//! - Built-in 5×7 bitmap font: ASCII, digits, and symbols
//! - `draw_str`, `draw_i16`, `draw_u32`, `draw_f32`
//! - Scaled text with `draw_char_scaled` and `draw_str_scaled`
//! - 1-bit bitmap rendering with `draw_bitmap`
//! - Hardware and software reset
//! - Orientation (`MADCTL`) and inversion control
//! - Zero allocation: `forbid(unsafe_code)`
//! - Full 240×320 RGB565 framebuffer with `St7789vBuffered` for flicker-free updates
//!
//! ## Wiring example (RP2350)
//!
//! | Display | GPIO  | Physical pin |
//! |--------|-------|-------------|
//! | VDD    | 3.3V  |             |
//! | GND    | GND   |             |
//! | SCL    | GP18  | Pin 24      |
//! | SDA    | GP19  | Pin 25      |
//! | DC     | GP16  | Pin 21      |
//! | CS     | GP20  | Pin 26      |
//! | RESET  | GP17  | Pin 22      |
//!
//! ## Quick start
//!
//! ```no_run
//! use embassy_st7789v::{Color, St7789v};
//!
//! let mut display = St7789v::new(spi_device, dc_pin, rst_pin);
//! display.init().await.unwrap();
//! display.fill_screen(Color::BLACK).await.unwrap();
//! display.draw_str(8, 10, b"HELLO WORLD", Color::WHITE, Color::BLACK).await.unwrap();
//! ```

//! // With a framebuffer (flicker-free):
//! let mut display = St7789vBuffered::new(St7789v::new(spi, dc, rst));
//! display.driver().init().await.unwrap();
//! display.fill_screen_buf(Color::BLACK);
//! display.draw_str_buf(8, 10, b"HELLO", Color::WHITE, Color::BLACK);
//! display.flush().await.unwrap();

#![no_std]
#![forbid(unsafe_code)]

// ============================================================
// EMBASSY TIME BACKEND SELECTION
// ============================================================

// ARM / Cortex-M: official Embassy time backend
#[cfg(feature = "time-upstream")]
extern crate embassy_time;

// RISC-V: custom Embassy time backend
#[cfg(feature = "time-riscv")]
extern crate embassy_time;

// Prevent selecting both backends simultaneously.
#[cfg(all(feature = "time-upstream", feature = "time-riscv"))]
compile_error!(
    "Features \"time-upstream\" and \"time-riscv\" are mutually exclusive."
);

// Require exactly one backend.
#[cfg(not(any(feature = "time-upstream", feature = "time-riscv")))]
compile_error!(
    "Select one Embassy time backend: \"time-upstream\" or \"time-riscv\"."
);


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

/// Display width in pixels.
pub const SCREEN_W: u16 = 240;

/// Display height in pixels.
pub const SCREEN_H: u16 = 320;

/// Constants for mathematical symbols in the built-in bitmap font.
pub use font::{
    LAMBDA, THETA, PI, DELTA, DEGREE, PLUS_MINUS,
    TIMES, DIVIDE, SQRT, INFINITY, APPROX, LE, GE,
};