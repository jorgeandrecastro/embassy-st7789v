// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Pilote async pour l'écran ST7789V (sans framebuffer).

use embassy_time::Timer;
use embedded_hal_async::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

use crate::cmd;
use crate::color::Color;
use crate::font::{self, FONT};
use crate::nopin::NoPin;
use crate::{SCREEN_W, SCREEN_H};

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
/// - TX uniquement : MISO n'est pas utilisé
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

    /// Définit la fenêtre de pixels active pour les écritures `RAMWR` suivantes.
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
            if let Some(idx) = font::char_to_glyph(c) {
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
            if let Some(idx) = font::char_to_glyph(c) {
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