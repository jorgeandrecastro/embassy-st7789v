# Changelog

Toutes les modifications notables de ce projet sont documentées ici.
Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [0.2.0] — 2026

### Ajouté
- `St7789vBuffered` : wrapper framebuffer complet 240×320 RGB565 (153 600 octets)
  - `fill_screen_buf` — remplit tout l'écran en RAM
  - `fill_rect_buf` — remplit un rectangle en RAM
  - `draw_rect_buf` — contour de rectangle en RAM
  - `set_pixel` — pixel unique en RAM
  - `draw_char_buf` — glyphe 5×7 en RAM
  - `draw_str_buf` — chaîne ASCII en RAM
  - `draw_i16_buf` — entier signé 16 bits en RAM
  - `draw_u32_buf` — entier non signé 32 bits en RAM
  - `driver()` — accès au pilote sous-jacent (`init`, `set_orientation`, etc.)
  - `flush()` — envoi du framebuffer complet à l'écran en une seule passe SPI
- `write_cmd` et `set_window_only` passés en `pub(crate)` pour `St7789vBuffered`

### Modifié
- Aucun changement d'API existante — compatibilité ascendante totale

---

## [0.1.0] — 2026

### Ajouté
- Pilote async `no_std` pour ST7789V 240×320 via SPI (Embassy)
- `Color` RGB565 avec constantes nommées (`BLACK`, `WHITE`, `RED`, `GREEN`,
  `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`, `ORANGE`, `GRAY`)
- `Color::rgb` et `Color::rgb8` pour couleurs personnalisées
- `St7789v::new` (avec RST) et `St7789v::new_no_rst` (sans RST)
- `NoPin` — broche fictive pour RST optionnel
- `init` — séquence complète d'initialisation des registres ST7789V
- `fill_screen`, `fill_rect`, `draw_rect`
- `draw_pixel`, `draw_hline`, `draw_vline`
- Police bitmap 5×7 intégrée (ASCII, chiffres, symboles courants)
- `draw_char`, `draw_str`
- `draw_i16`, `draw_u32`, `draw_f32` (avec gestion NaN / ±Inf)
- `draw_char_scaled`, `draw_str_scaled` — texte agrandi par facteur entier
- `draw_bitmap` — rendu bitmap 1 bit compressé MSB
- `set_orientation` — contrôle MADCTL (portrait, paysage, retourné)
- `set_invert` — inversion des couleurs (obligatoire sur dalles IPS)