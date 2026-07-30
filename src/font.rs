// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later

//! Police bitmap 5×7.
//!
//! - Index 0–9   : '0'–'9'
//! - Index 10    : '-'
//! - Index 11    : ' '
//! - Index 12–37 : 'A'–'Z' (insensible à la casse)
//! - Index 38–59 : ponctuation (voir [`char_to_glyph`])
//! - Index 60–72 : symboles grecs et mathématiques, accessibles via
//!   des codes étendus non-ASCII (`0x80`–`0x8C`, voir les constantes
//!   [`LAMBDA`], [`THETA`], [`PI`], [`DELTA`], [`DEGREE`], [`PLUS_MINUS`],
//!   [`TIMES`], [`DIVIDE`], [`SQRT`], [`INFINITY`], [`APPROX`], [`LE`], [`GE`])
//!
//! ## Symboles étendus
//!
//! Comme [`crate::St7789v::draw_str`] et consorts prennent un `&[u8]` brut
//! (pas de l'UTF-8), les symboles non-ASCII sont représentés par des codes
//! dans la plage `0x80..=0x8C`, qui ne rentrent jamais en collision avec
//! l'ASCII standard. Utilisez les constantes fournies plutôt que les
//! valeurs numériques brutes :
//!
//! ```no_run
//! use embassy_st7789v::{Color, PI, DEGREE};
//!
//! ecran.draw_str(8, 10, &[b'T', b'=', 0x32, DEGREE, b'C'], Color::WHITE, Color::BLACK).await?;
//! // ou, mélangé avec des littéraux b'...' normalement :
//! let msg: [u8; 3] = [b'R', b'=', PI];
//! ecran.draw_str(8, 20, &msg, Color::WHITE, Color::BLACK).await?;
//! ```

pub(crate) const FONT: [[u8; 5]; 73] = [
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
    [0x60, 0x10, 0x0F, 0x10, 0x60], // 60 = 'λ' (lambda) — Y inversé simple
    [0x3E, 0x49, 0x49, 0x49, 0x3E], // 61 = 'θ' (theta)
    [0x01, 0x7D, 0x01, 0x7D, 0x01], // 62 = 'π' (pi)
    [0x78, 0x46, 0x41, 0x46, 0x78], // 63 = 'Δ' (delta majuscule)
    [0x06, 0x09, 0x09, 0x06, 0x00], // 64 = '°' (degré)
    [0x48, 0x48, 0x5E, 0x48, 0x48], // 65 = '±' (plus ou moins)
    [0x22, 0x14, 0x08, 0x14, 0x22], // 66 = '×' (multiplication)
    [0x08, 0x08, 0x49, 0x08, 0x08], // 67 = '÷' (division)
    [0x70, 0x4C, 0x03, 0x01, 0x01], // 68 = '√' (racine carrée)
    [0x1C, 0x22, 0x1C, 0x22, 0x1C], // 69 = '∞' — boucles pleines, mieux centrées
    [0x00, 0x24, 0x12, 0x12, 0x24], // 70 = '≈' (approximativement égal)
    [0x60, 0x68, 0x64, 0x62, 0x61], // 71 = '≤' (inférieur ou égal)
    [0x61, 0x62, 0x64, 0x68, 0x60], // 72 = '≥' (supérieur ou égal)
];

// ─────────────────────────────────────────────────────────────────────────────
// Codes étendus (0x80–0x8C) pour les symboles grecs et mathématiques
// ─────────────────────────────────────────────────────────────────────────────

/// Code étendu pour 'λ' (lambda minuscule).
pub const LAMBDA: u8 = 0x80;
/// Code étendu pour 'θ' (theta minuscule).
pub const THETA: u8 = 0x81;
/// Code étendu pour 'π' (pi).
pub const PI: u8 = 0x82;
/// Code étendu pour 'Δ' (delta majuscule).
pub const DELTA: u8 = 0x83;
/// Code étendu pour '°' (symbole degré).
pub const DEGREE: u8 = 0x84;
/// Code étendu pour '±' (plus ou moins).
pub const PLUS_MINUS: u8 = 0x85;
/// Code étendu pour '×' (multiplication).
pub const TIMES: u8 = 0x86;
/// Code étendu pour '÷' (division).
pub const DIVIDE: u8 = 0x87;
/// Code étendu pour '√' (racine carrée).
pub const SQRT: u8 = 0x88;
/// Code étendu pour '∞' (infini).
pub const INFINITY: u8 = 0x89;
/// Code étendu pour '≈' (approximativement égal).
pub const APPROX: u8 = 0x8A;
/// Code étendu pour '≤' (inférieur ou égal).
pub const LE: u8 = 0x8B;
/// Code étendu pour '≥' (supérieur ou égal).
pub const GE: u8 = 0x8C;

/// Convertit un octet ASCII ou un code étendu en index de glyphe dans la table [`FONT`].
/// Retourne `None` pour les caractères non supportés.
pub(crate) fn char_to_glyph(c: u8) -> Option<usize> {
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
        LAMBDA      => Some(60),
        THETA       => Some(61),
        PI          => Some(62),
        DELTA       => Some(63),
        DEGREE      => Some(64),
        PLUS_MINUS  => Some(65),
        TIMES       => Some(66),
        DIVIDE      => Some(67),
        SQRT        => Some(68),
        INFINITY    => Some(69),
        APPROX      => Some(70),
        LE          => Some(71),
        GE          => Some(72),
        _           => None,
    }
}