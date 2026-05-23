# embassy-st7789v

[![Crates.io](https://img.shields.io/crates/v/embassy-st7789v.svg)](https://crates.io/crates/embassy-st7789v)
[![Docs.rs](https://docs.rs/embassy-st7789v/badge.svg)](https://docs.rs/embassy-st7789v)
[![Licence : GPL-2.0-or-later](https://img.shields.io/badge/licence-GPL--2.0--or--later-blue.svg)](LICENSE)

Pilote async `no_std` pour l'écran TFT LCD **ST7789V** 240×320 via SPI,
basé sur [Embassy](https://embassy.dev).

Aucun allocateur requis. Aucun code unsafe.

---

## Fonctionnalités

- Couleurs RGB565 avec constantes nommées (`BLACK`, `RED`, `CYAN` …)
- `fill_screen`, `fill_rect`, `draw_rect`
- `draw_pixel`, `draw_hline`, `draw_vline`
- Police bitmap 5×7 intégrée : lettres ASCII, chiffres et symboles courants
- `draw_str`, `draw_i16`, `draw_u32`, `draw_f32`
- Texte mis à l'échelle : `draw_char_scaled`, `draw_str_scaled`
- Rendu de bitmap 1 bit via `draw_bitmap`
- Réinitialisation matérielle et logicielle
- Contrôle de l'orientation (`MADCTL`) et de l'inversion
- Broche RST optionnelle : utilisez `new_no_rst` si RESET est câblé haut
- Zéro allocation :  `forbid(unsafe_code)`

---

## Matériel supporté

Tout panneau piloté par un ST7789V via SPI 4 fils (DC + CS + SCL + SDA).
Testé sur un module IPS TFT 2.0" (240×320) avec un RP2350 (Raspberry Pi Pico 2).

---

## Câblage — exemple RP2350

| Broche | Symbole | RP2350  | Pin physique |
|--------|---------|---------|-------------|
| 1      | GND     | GND     |             |
| 2      | VDD     | 3.3V    |             |
| 3      | DC     | GPIO 16 | Pin 21       |
| 4      | CS      | GPIO 20 | Pin 26      |
| 5      | SCL     | GPIO 18 | Pin 24      |
| 6      | SDA     | GPIO 19 | Pin 25      |
| 7      | RESET   | GPIO 17 | Pin 22      |


---

## Installation

```toml
[dependencies]
embassy-st7789v    = "0.1"
embassy-time       = "0.5"
embedded-hal       = "1.0"
embedded-hal-async = "1.0"
```

---

## Utilisation

### Initialisation minimale (RP2350 + Embassy)

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

// Dans votre async main :
let mut spi_cfg = SpiConfig::default();
spi_cfg.frequency = 40_000_000;

let spi = Spi::new_txonly(p.SPI0, p.PIN_18, p.PIN_19, p.DMA_CH0, Irqs, spi_cfg);
let spi_bus = SPI_BUS.init(Mutex::new(spi));

let cs  = Output::new(p.PIN_20, Level::High);
let dc  = Output::new(p.PIN_16, Level::Low);
let rst = Output::new(p.PIN_17, Level::High);

let spi_dev = SpiDevice::new(spi_bus, cs);
let mut ecran = St7789v::new(spi_dev, dc, rst);

ecran.init().await.unwrap();
```

### Dessin

```rust
// Remplir l'écran
ecran.fill_screen(Color::BLACK).await.unwrap();

// Rectangles colorés
ecran.fill_rect(0,  0, 79, 79, Color::RED).await.unwrap();
ecran.fill_rect(80, 0, 159, 79, Color::GREEN).await.unwrap();

// Contour de rectangle
ecran.draw_rect(10, 10, 230, 310, Color::WHITE).await.unwrap();

// Lignes
ecran.draw_hline(0, 160, 240, Color::CYAN).await.unwrap();
ecran.draw_vline(120, 0, 320, Color::MAGENTA).await.unwrap();

// Pixel unique
ecran.draw_pixel(120, 160, Color::YELLOW).await.unwrap();
```

### Texte taille normale

```rust
// Chaîne (majuscules et minuscules :la police est insensible à la casse)
ecran.draw_str(8, 10, b"Bonjour ST7789V", Color::WHITE, Color::BLACK).await.unwrap();

// Entier signé
ecran.draw_str(8, 20, b"Temp: ", Color::WHITE, Color::BLACK).await.unwrap();
ecran.draw_i16(56, 20, -42, Color::YELLOW, Color::BLACK).await.unwrap();

// Entier non signé
ecran.draw_str(8, 30, b"Compteur: ", Color::WHITE, Color::BLACK).await.unwrap();
ecran.draw_u32(74, 30, 123456, Color::CYAN, Color::BLACK).await.unwrap();
```

### Texte mis à l'échelle

```rust
// Titre en double taille (chaque caractère : 10×14 px)
ecran.draw_str_scaled(8, 10, b"ERREUR", Color::RED, Color::BLACK, 2).await.unwrap();

// Titre en triple taille (chaque caractère : 15×21 px)
ecran.draw_str_scaled(8, 40, b"OK", Color::GREEN, Color::BLACK, 3).await.unwrap();

// Texte normal (scale 1 = identique à draw_str)
ecran.draw_str_scaled(8, 80, b"details", Color::WHITE, Color::BLACK, 1).await.unwrap();
```

### Affichage de flottants

```rust
// Affiche "-3.14"
ecran.draw_f32(8, 100, -3.14, 2, Color::YELLOW, Color::BLACK).await.unwrap();

// Affiche "23.5" (1 décimale)
ecran.draw_f32(8, 110, 23.456, 1, Color::CYAN, Color::BLACK).await.unwrap();

// Affiche "100" (sans décimale)
ecran.draw_f32(8, 120, 100.0, 0, Color::WHITE, Color::BLACK).await.unwrap();

// Cas spéciaux : affiche "NaN", "+Inf", "-Inf"
ecran.draw_f32(8, 130, f32::NAN, 2, Color::RED, Color::BLACK).await.unwrap();
```

### Couleurs

```rust
// Constantes prédéfinies
let c = Color::RED;

// RGB565 brut (r : 0–31, g : 0–63, b : 0–31)
let c = Color::rgb(31, 40, 0); // orange

// Depuis RGB 8 bits (réduit en RGB565)
let c = Color::rgb8(0xFF, 0x80, 0x00); // orange
```

### Orientation

```rust
ecran.set_orientation(0x00).await.unwrap(); // portrait (défaut)
ecran.set_orientation(0x60).await.unwrap(); // paysage
ecran.set_orientation(0xC0).await.unwrap(); // portrait retourné
ecran.set_orientation(0xA0).await.unwrap(); // paysage retourné
```

### Inversion

Les dalles IPS avec ST7789V nécessitent l'inversion activée — c'est le réglage par défaut après `init`.
Si vos couleurs apparaissent inversées :

```rust
ecran.set_invert(false).await.unwrap();
```

---

## Dépannage

| Symptôme | Cause probable | Solution |
|----------|---------------|----------|
| Gris avec rétroéclairage allumé | SPI n'atteint pas l'écran | Vérifier SCL/SDA/CS/DC |
| Couleurs inversées | Mauvais réglage d'inversion | Appeler `set_invert(false)` après `init` |
| Mauvaise orientation | MADCTL incorrect | Essayer `set_orientation(0x60)` / `0xC0` / `0xA0` |
| Affichage corrompu | SPI trop rapide | Baisser la fréquence à `10_000_000` pour déboguer |

---

## Cargo.toml

```toml
[dependencies]
embassy-st7789v    = "0.1"
embassy-time       = "0.5"
embedded-hal       = "1.0"
embedded-hal-async = "1.0"
```

---

## Licence

Copyright (C) 2026 Jorge Andre Castro
Sous licence [GNU General Public License v2.0 ou ultérieure](LICENSE).