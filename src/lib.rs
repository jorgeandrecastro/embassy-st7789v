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

#![no_std]
#![forbid(unsafe_code)]

use embassy_time::Timer;
use embedded_hal_async::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

/// Largeur de l'écran en pixels.
pub const SCREEN_W: u16 = 240;

/// Hauteur de l'écran en pixels.
pub const SCREEN_H: u16 = 320;

// ─────────────────────────────────────────────────────────────────────────────
// Color
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Constantes de commandes ST7789V (internes)
// ─────────────────────────────────────────────────────────────────────────────

mod cmd {
    pub const SWRESET:   u8 = 0x01;
    pub const SLPOUT:    u8 = 0x11;
    pub const NORON:     u8 = 0x13;
    pub const INVOFF:    u8 = 0x20;
    pub const INVON:     u8 = 0x21;
    pub const DISPON:    u8 = 0x29;
    pub const CASET:     u8 = 0x2A;
    pub const RASET:     u8 = 0x2B;
    pub const RAMWR:     u8 = 0x2C;
    pub const MADCTL:    u8 = 0x36;
    pub const COLMOD:    u8 = 0x3A;
    pub const PORCTRL:   u8 = 0xB2;
    pub const GCTRL:     u8 = 0xB7;
    pub const VCOMS:     u8 = 0xBB;
    pub const LCMCTRL:   u8 = 0xC0;
    pub const VDVVRHEN:  u8 = 0xC2;
    pub const VRHS:      u8 = 0xC3;
    pub const VDVS:      u8 = 0xC4;
    pub const FRCTRL2:   u8 = 0xC6;
    pub const PWCTRL1:   u8 = 0xD0;
    pub const PVGAMCTRL: u8 = 0xE0;
    pub const NVGAMCTRL: u8 = 0xE1;
}

// ─────────────────────────────────────────────────────────────────────────────
// Police bitmap 5×7
// Index 0–9   : '0'–'9'
// Index 10    : '-'
// Index 11    : ' '
// Index 12–37 : 'A'–'Z' (insensible à la casse)
// Index 38–59 : ponctuation (voir `char_to_glyph`)
// ─────────────────────────────────────────────────────────────────────────────

const FONT: [[u8; 5]; 60] = [
    [0x3E, 0x51, 0x49, 0x45, 0x3E], // 0
    [0x00, 0x42, 0x7F, 0x40, 0x00], // 1
    [0x42, 0x61, 0x51, 0x49, 0x46], // 2
    [0x21, 0x41, 0x45, 0x4B, 0x31], // 3
    [0x18, 0x14, 0x12, 0x7F, 0x10], // 4
    [0x27, 0x45, 0x45, 0x45, 0x39], // 5
    [0x3C, 0x4A, 0x49, 0x49, 0x30], // 6
    [0x01, 0x71, 0x09, 0x05, 0x03], // 7
    [0x36, 0x49, 0x49, 0x49, 0x36], // 8
    [0x06, 0x49, 0x49, 0x29, 0x1E], // 9
    [0x08, 0x08, 0x08, 0x08, 0x08], // 10 = '-'
    [0x00, 0x00, 0x00, 0x00, 0x00], // 11 = ' '
    [0x7E, 0x11, 0x11, 0x11, 0x7E], // 12 = 'A'
    [0x7F, 0x49, 0x49, 0x49, 0x36], // 13 = 'B'
    [0x3E, 0x41, 0x41, 0x41, 0x22], // 14 = 'C'
    [0x7F, 0x41, 0x41, 0x22, 0x1C], // 15 = 'D'
    [0x7F, 0x49, 0x49, 0x49, 0x41], // 16 = 'E'
    [0x7F, 0x09, 0x09, 0x09, 0x01], // 17 = 'F'
    [0x3E, 0x41, 0x49, 0x49, 0x7A], // 18 = 'G'
    [0x7F, 0x08, 0x08, 0x08, 0x7F], // 19 = 'H'
    [0x00, 0x41, 0x7F, 0x41, 0x00], // 20 = 'I'
    [0x20, 0x40, 0x41, 0x3F, 0x01], // 21 = 'J'
    [0x7F, 0x08, 0x14, 0x22, 0x41], // 22 = 'K'
    [0x7F, 0x40, 0x40, 0x40, 0x40], // 23 = 'L'
    [0x7F, 0x02, 0x0C, 0x02, 0x7F], // 24 = 'M'
    [0x7F, 0x04, 0x08, 0x10, 0x7F], // 25 = 'N'
    [0x3E, 0x41, 0x41, 0x41, 0x3E], // 26 = 'O'
    [0x7F, 0x09, 0x09, 0x09, 0x06], // 27 = 'P'
    [0x3E, 0x41, 0x51, 0x21, 0x5E], // 28 = 'Q'
    [0x7F, 0x09, 0x19, 0x29, 0x46], // 29 = 'R'
    [0x46, 0x49, 0x49, 0x49, 0x31], // 30 = 'S'
    [0x01, 0x01, 0x7F, 0x01, 0x01], // 31 = 'T'
    [0x3F, 0x40, 0x40, 0x40, 0x3F], // 32 = 'U'
    [0x1F, 0x20, 0x40, 0x20, 0x1F], // 33 = 'V'
    [0x3F, 0x40, 0x38, 0x40, 0x3F], // 34 = 'W'
    [0x63, 0x14, 0x08, 0x14, 0x63], // 35 = 'X'
    [0x07, 0x08, 0x70, 0x08, 0x07], // 36 = 'Y'
    [0x61, 0x51, 0x49, 0x45, 0x43], // 37 = 'Z'
    [0x00, 0x00, 0x60, 0x60, 0x00], // 38 = '.'
    [0x00, 0x3E, 0x41, 0x41, 0x00], // 39 = '('
    [0x00, 0x41, 0x41, 0x3E, 0x00], // 40 = ')'
    [0x00, 0x40, 0x50, 0x30, 0x00], // 41 = ','
    [0x00, 0x7F, 0x41, 0x41, 0x00], // 42 = '['
    [0x00, 0x41, 0x41, 0x7F, 0x00], // 43 = ']'
    [0x23, 0x13, 0x08, 0x64, 0x62], // 44 = '%'
    [0x08, 0x14, 0x22, 0x41, 0x00], // 45 = '<'
    [0x00, 0x41, 0x22, 0x14, 0x08], // 46 = '>'
    [0x00, 0x24, 0x24, 0x24, 0x00], // 47 = '='
    [0x02, 0x01, 0x51, 0x09, 0x06], // 48 = '?'
    [0x00, 0x00, 0x5F, 0x00, 0x00], // 49 = '!'
    [0x00, 0x36, 0x36, 0x00, 0x00], // 50 = ':'
    [0x08, 0x08, 0x3E, 0x08, 0x08], // 51 = '+'
    [0x20, 0x10, 0x08, 0x04, 0x02], // 52 = '/'
    [0x00, 0x00, 0x7F, 0x00, 0x00], // 53 = '|'
    [0x40, 0x40, 0x40, 0x40, 0x40], // 54 = '_'
    [0x04, 0x02, 0x01, 0x02, 0x04], // 55 = '^'
    [0x14, 0x7F, 0x14, 0x7F, 0x14], // 56 = '#'
    [0x3E, 0x41, 0x5D, 0x55, 0x1E], // 57 = '@'
    [0x32, 0x49, 0x55, 0x22, 0x50], // 58 = '&'
    [0x00, 0x07, 0x00, 0x07, 0x00], // 59 = '"'
];

// ─────────────────────────────────────────────────────────────────────────────
// NoPin — broche fictive pour RST optionnel
// ─────────────────────────────────────────────────────────────────────────────

/// Broche fictive utilisée quand aucune broche de réinitialisation matérielle n'est disponible.
///
/// Utilisez [`St7789v::new_no_rst`] quand la broche RESET est connectée
/// au 3.3V ou gérée en externe.
pub struct NoPin;

impl OutputPin for NoPin {
    fn set_low(&mut self)  -> Result<(), Self::Error> { Ok(()) }
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

impl embedded_hal::digital::ErrorType for NoPin {
    type Error = core::convert::Infallible;
}

// ─────────────────────────────────────────────────────────────────────────────
// Pilote St7789v
// ─────────────────────────────────────────────────────────────────────────────

/// Pilote async pour l'écran TFT LCD ST7789V 240×320.
///
/// Générique sur tout [`SpiDevice`], une broche données/commande [`OutputPin`] (`DC`),
/// et une broche de réinitialisation optionnelle [`OutputPin`] (`RST`, par défaut [`NoPin`]).
///
/// # Construction
///
/// Utilisez [`St7789v::new`] si vous disposez d'une broche RST matérielle, ou
/// [`St7789v::new_no_rst`] si RESET est câblé haut ou absent.
///
/// Appelez toujours [`St7789v::init`] une fois avant de dessiner.
///
/// # Exigences SPI
///
/// - Mode 0 (CPOL=0, CPHA=0)
/// - MSB en premier
/// - Jusqu'à 40 MHz (commencer à 10 MHz pour le débogage)
/// - TX uniquement — MISO n'est pas utilisé
pub struct St7789v<SPI, DC, RST = NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    spi: SPI,
    dc: DC,
    rst: RST,
}

// ── Constructeur (sans RST) ───────────────────────────────────────────────────

impl<SPI, DC> St7789v<SPI, DC, NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    /// Crée un pilote sans broche de réinitialisation matérielle.
    ///
    /// La réinitialisation logicielle dans [`init`](St7789v::init) sera
    /// quand même effectuée, mais la broche RESET doit être maintenue haute en externe.
    pub fn new_no_rst(spi: SPI, dc: DC) -> Self {
        Self { spi, dc, rst: NoPin }
    }
}

// ── Constructeur (avec RST) ───────────────────────────────────────────────────

impl<SPI, DC, RST> St7789v<SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Crée un pilote avec une broche de réinitialisation matérielle.
    ///
    /// `rst` sera mise à l'état bas puis haut pendant [`init`](St7789v::init).
    pub fn new(spi: SPI, dc: DC, rst: RST) -> Self {
        Self { spi, dc, rst }
    }

    // ── Helpers SPI bas niveau ────────────────────────────────────────────────

    #[inline]
    async fn write_cmd(&mut self, c: u8) -> Result<(), SPI::Error> {
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

    /// Définit la fenêtre de pixels active pour les écritures `RAMWR` suivantes.
    #[inline]
    async fn set_window_only(
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

    // ── Initialisation ────────────────────────────────────────────────────────

    /// Initialise l'écran.
    ///
    /// Doit être appelé une fois après la mise sous tension, avant toute opération
    /// de dessin. Effectue la réinitialisation matérielle (si une broche RST est fournie),
    /// la réinitialisation logicielle, puis envoie la séquence complète d'initialisation
    /// des registres.
    ///
    /// # Erreurs
    ///
    /// Retourne l'erreur du périphérique SPI en cas d'échec de communication.
    #[must_use = "vérifiez le résultat — un init raté laisse l'écran dans un état indéfini"]
    pub async fn init(&mut self) -> Result<(), SPI::Error> {
        // Réinitialisation matérielle
        let _ = self.rst.set_low();
        Timer::after_millis(10).await;
        let _ = self.rst.set_high();
        Timer::after_millis(120).await;

        // Réinitialisation logicielle
        self.write_cmd(cmd::SWRESET).await?;
        Timer::after_millis(150).await;

        // Sortie du mode veille
        self.write_cmd(cmd::SLPOUT).await?;
        Timer::after_millis(10).await;

        // Format de pixel : RGB565
        self.cmd1(cmd::COLMOD, 0x55).await?;

        // Contrôle du porche
        self.cmdn(cmd::PORCTRL, &[0x0C, 0x0C, 0x00, 0x33, 0x33]).await?;

        // Contrôle de la grille
        self.cmd1(cmd::GCTRL, 0x35).await?;

        // Tension VCOM
        self.cmd1(cmd::VCOMS, 0x19).await?;

        // Contrôle LCM
        self.cmd1(cmd::LCMCTRL, 0x2C).await?;

        // Activation VDV/VRH
        self.cmdn(cmd::VDVVRHEN, &[0x01, 0xFF]).await?;

        // Réglage VRH
        self.cmd1(cmd::VRHS, 0x12).await?;

        // Réglage VDV
        self.cmd1(cmd::VDVS, 0x20).await?;

        // Fréquence de rafraîchissement : 60 Hz
        self.cmd1(cmd::FRCTRL2, 0x0F).await?;

        // Contrôle d'alimentation 1
        self.cmdn(cmd::PWCTRL1, &[0xA4, 0xA1]).await?;

        // Contrôle d'accès mémoire : portrait, origine en haut à gauche
        // Si l'écran ne s'affiche pas ou est mal orienté, essayez :
        // 0x60 (paysage), 0xC0 (portrait retourné), 0xA0 (paysage retourné)
        self.cmd1(cmd::MADCTL, 0x00).await?;

        // Gamma positif
        self.cmdn(cmd::PVGAMCTRL, &[
            0xD0, 0x04, 0x0D, 0x11, 0x13, 0x2B, 0x3F, 0x54,
            0x4C, 0x18, 0x0D, 0x0B, 0x1F, 0x23,
        ]).await?;

        // Gamma négatif
        self.cmdn(cmd::NVGAMCTRL, &[
            0xD0, 0x04, 0x0C, 0x11, 0x13, 0x2C, 0x3F, 0x44,
            0x51, 0x2F, 0x1F, 0x1F, 0x20, 0x23,
        ]).await?;

        // Mode d'affichage normal
        self.write_cmd(cmd::NORON).await?;

        // Inversion activée : obligatoire pour les dalles IPS avec ST7789V
        self.write_cmd(cmd::INVON).await?;

        // Affichage allumé
        self.write_cmd(cmd::DISPON).await?;
        Timer::after_millis(10).await;

        Ok(())
    }

    // ── Primitives de dessin ──────────────────────────────────────────────────

    /// Remplit tout l'écran 240×320 avec `couleur`.
    pub async fn fill_screen(&mut self, color: Color) -> Result<(), SPI::Error> {
        self.fill_rect(0, 0, SCREEN_W - 1, SCREEN_H - 1, color).await
    }

    /// Remplit un rectangle de `(x0, y0)` à `(x1, y1)` inclus avec `couleur`.
    ///
    /// Les coordonnées sont limitées aux bords de l'écran.
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

        // Envoi par blocs de 64 pixels pour éviter les gros buffers sur la pile
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

    /// Dessine un pixel unique en `(x, y)`.
    ///
    /// Les coordonnées hors limites sont ignorées silencieusement.
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

    /// Dessine une ligne horizontale depuis `(x, y)` de longueur `w`.
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

    /// Dessine une ligne verticale depuis `(x, y)` de hauteur `h`.
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

    /// Dessine le contour d'un rectangle de `(x0, y0)` à `(x1, y1)`.
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

    /// Affiche un bitmap 1 bit compressé en `(x, y)`.
    ///
    /// Les pixels à `1` sont dessinés en `fg`, les pixels à `0` en `bg`.
    /// Les bits sont compressés MSB en premier, une ligne par `ceil(w/8)` octets.
    ///
    /// # Paramètres
    ///
    /// - `w`, `h` : dimensions en pixels
    /// - `data`   : bitmap compressé, `ceil(w/8) * h` octets
    /// - `fg`     : couleur de premier plan (bits à 1)
    /// - `bg`     : couleur d'arrière-plan (bits à 0)
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

    // ── Police / texte ────────────────────────────────────────────────────────

    /// Convertit un octet ASCII en index de glyphe dans la table [`FONT`].
    /// Retourne `None` pour les caractères non supportés.
    fn char_to_glyph(c: u8) -> Option<usize> {
        match c {
            b'0'..=b'9' => Some((c - b'0') as usize),
            b'-'        => Some(10),
            b' '        => Some(11),
            b'A'..=b'Z' => Some((c - b'A') as usize + 12),
            b'a'..=b'z' => Some((c - b'a') as usize + 12),
            b'.'        => Some(38),
            b'('        => Some(39),
            b')'        => Some(40),
            b','        => Some(41),
            b'['        => Some(42),
            b']'        => Some(43),
            b'%'        => Some(44),
            b'<'        => Some(45),
            b'>'        => Some(46),
            b'='        => Some(47),
            b'?'        => Some(48),
            b'!'        => Some(49),
            b':'        => Some(50),
            b'+'        => Some(51),
            b'/'        => Some(52),
            b'|'        => Some(53),
            b'_'        => Some(54),
            b'^'        => Some(55),
            b'#'        => Some(56),
            b'@'        => Some(57),
            b'&'        => Some(58),
            b'"'        => Some(59),
            _           => None,
        }
    }

    /// Dessine un glyphe 5×7 unique en `(x, y)`.
    ///
    /// Le glyphe occupe une cellule de 5×7 pixels. Utilisez [`draw_str`](Self::draw_str)
    /// pour afficher du texte directement.
    ///
    /// Retourne la coordonnée x immédiatement après le glyphe (soit `x + 6`,
    /// incluant 1 pixel d'espacement).
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

        // 5×7 = 35 pixels × 2 octets = 70 octets : un seul write par glyphe
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

    /// Affiche une chaîne d'octets ASCII en `(x, y)`.
    ///
    /// Les caractères non supportés avancent le curseur de 6 pixels sans dessiner.
    /// L'affichage s'arrête si le bord droit de l'écran est atteint.
    ///
    /// Retourne la coordonnée x après le dernier caractère.
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// ecran.draw_str(8, 20, b"TEMP: ", Color::WHITE, Color::BLACK).await?;
    /// ecran.draw_i16(56, 20, temperature, Color::YELLOW, Color::BLACK).await?;
    /// ```
    pub async fn draw_str(
        &mut self,
        mut x: u16, y: u16,
        text: &[u8],
        fg: Color, bg: Color,
    ) -> Result<u16, SPI::Error> {
        for &c in text {
            if x + 5 >= SCREEN_W { break; }
            if let Some(idx) = Self::char_to_glyph(c) {
                x = self.draw_char(x, y, idx, fg, bg).await?;
            } else {
                x = x.saturating_add(6);
            }
        }
        Ok(x)
    }

    /// Affiche un entier signé 16 bits en `(x, y)`.
    ///
    /// Un glyphe `'-'` est ajouté en tête pour les valeurs négatives.
    /// Retourne la coordonnée x après le dernier chiffre.
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

    /// Affiche un entier non signé 32 bits en `(x, y)`.
    ///
    /// Retourne la coordonnée x après le dernier chiffre.
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

    // ── Texte mis à l'échelle ─────────────────────────────────────────────────

    /// Dessine un glyphe 5×7 agrandi d'un facteur `scale` en `(x, y)`.
    ///
    /// Chaque pixel de la police est rendu comme un carré de `scale × scale` pixels.
    /// `scale = 1` est identique à [`draw_char`](Self::draw_char).
    /// `scale = 2` donne un glyphe de 10×14 pixels, `scale = 3` un glyphe de 15×21, etc.
    ///
    /// Retourne la coordonnée x après le glyphe (soit `x + (6 * scale as u16)`).
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// // Titre en grand (scale 3 = 15×21 px par caractère)
    /// ecran.draw_char_scaled(10, 10, idx, Color::YELLOW, Color::BLACK, 3).await?;
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

        // Buffer d'une ligne horizontale mise à l'échelle (max scale=8 → 5×8×2 = 80 octets)
        let mut ligne_buf = [0u8; 5 * 8 * 2];
        let ligne_len = (5 * scale as usize) * 2;

        for ligne in 0..7usize {
            // Préparer les pixels de la ligne mis à l'échelle
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
            // Répéter la ligne `scale` fois verticalement
            for _ in 0..scale {
                let _ = self.dc.set_high();
                self.spi.write(&ligne_buf[..ligne_len]).await?;
            }
        }

        Ok(x + 6 * s)
    }

    /// Affiche une chaîne d'octets ASCII mise à l'échelle en `(x, y)`.
    ///
    /// Chaque caractère est agrandi d'un facteur `scale` via [`draw_char_scaled`](Self::draw_char_scaled).
    /// L'espacement entre les caractères est également mis à l'échelle (`scale` pixels).
    ///
    /// Retourne la coordonnée x après le dernier caractère.
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// // Titre en double taille
    /// ecran.draw_str_scaled(8, 10, b"ERREUR", Color::RED, Color::BLACK, 2).await?;
    ///
    /// // Texte normal
    /// ecran.draw_str_scaled(8, 40, b"details ici", Color::WHITE, Color::BLACK, 1).await?;
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
            if let Some(idx) = Self::char_to_glyph(c) {
                x = self.draw_char_scaled(x, y, idx, fg, bg, scale).await?;
            } else {
                x = x.saturating_add(6 * s);
            }
        }
        Ok(x)
    }

    // ── Affichage de flottants ────────────────────────────────────────────────

    /// Affiche un nombre flottant `f32` en `(x, y)` avec `decimales` chiffres après la virgule.
    ///
    /// Gère les valeurs négatives, zéro, et les valeurs non représentables
    /// (`NaN`, `+Inf`, `-Inf`) avec des messages lisibles.
    ///
    /// Retourne la coordonnée x après le dernier caractère affiché.
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// // Affiche "-3.14" en jaune
    /// ecran.draw_f32(8, 60, -3.14, 2, Color::YELLOW, Color::BLACK).await?;
    ///
    /// // Affiche "23.5" (1 décimale)
    /// ecran.draw_f32(8, 70, 23.456, 1, Color::CYAN, Color::BLACK).await?;
    ///
    /// // Affiche "100" (0 décimale)
    /// ecran.draw_f32(8, 80, 100.0, 0, Color::WHITE, Color::BLACK).await?;
    /// ```
    pub async fn draw_f32(
        &mut self,
        mut x: u16, y: u16,
        val: f32,
        decimales: u8,
        fg: Color, bg: Color,
    ) -> Result<u16, SPI::Error> {
        // Cas spéciaux
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

        // Signe
        let negatif = val < 0.0;
        let mut abs = if negatif { -val } else { val };

        if negatif {
            x = self.draw_char(x, y, 10, fg, bg).await?; // '-'
        }

        // Arrondi à la dernière décimale demandée
        let facteur = {
            let mut f = 1u32;
            for _ in 0..decimales { f *= 10; }
            f
        };
        // On arrondit en ajoutant 0.5 à la dernière position
        abs += 0.5 / facteur as f32;

        // Partie entière
        let entier = abs as u32;
        x = self.draw_u32(x, y, entier, fg, bg).await?;

        // Partie décimale
        if decimales > 0 {
            // Point décimal (index 38 = '.')
            x = self.draw_char(x, y, 38, fg, bg).await?;

            // Extraire les chiffres décimaux avec zéros de tête
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

    // ── Contrôle de l'affichage ───────────────────────────────────────────────

    /// Définit le registre de contrôle d'accès mémoire (`MADCTL`).
    ///
    /// Contrôle l'orientation de l'affichage et l'ordre RGB/BGR.
    ///
    /// | Valeur | Orientation              |
    /// |--------|--------------------------|
    /// | `0x00` | Portrait (défaut)        |
    /// | `0x60` | Paysage                  |
    /// | `0xC0` | Portrait retourné        |
    /// | `0xA0` | Paysage retourné         |
    pub async fn set_orientation(&mut self, madctl: u8) -> Result<(), SPI::Error> {
        self.cmd1(cmd::MADCTL, madctl).await
    }

    /// Active ou désactive l'inversion des couleurs.
    ///
    /// Les dalles IPS avec ST7789V nécessitent généralement l'inversion activée (`true`),
    /// ce qui est le réglage par défaut après [`init`](St7789v::init). Passez `false`
    /// si les couleurs apparaissent inversées sur votre dalle.
    pub async fn set_invert(&mut self, invert: bool) -> Result<(), SPI::Error> {
        if invert {
            self.write_cmd(cmd::INVON).await
        } else {
            self.write_cmd(cmd::INVOFF).await
        }
    }
}