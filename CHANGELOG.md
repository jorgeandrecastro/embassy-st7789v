# Changelog

Toutes les modifications notables de ce projet sont documentées ici.
Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [0.4.0] — 2026-07-28

### Modifié
- Refactorisation du pilote en modules séparés : `cmd`, `font`, `nopin`, `color`, `driver` et `buffered`.
- Réexport des types principaux depuis la crate racine (`Color`, `NoPin`, `St7789v`, `St7789vBuffered`) pour simplifier l'utilisation.
- Amélioration de la structure interne du code sans changement fonctionnel sur l'API de dessin existante.

---

## [0.3.0] — 2026-06-06

### Ajouté
Texte mis à l'échelle (framebuffer)

**St7789vBuffered::draw_char_scaled_buf** — dessine un glyphe 5×7 agrandi d'un facteur entier scale: u8 dans le framebuffer. Chaque pixel source est rendu comme un carré de scale × scale pixels.
**St7789vBuffered::draw_str_scaled_buf** — affiche une chaîne ASCII mise à l'échelle dans le framebuffer. L'espacement entre caractères est proportionnel au facteur scale. L'affichage s'arrête automatiquement au bord droit de l'écran.

Ces deux méthodes complètent draw_char_scaled et draw_str_scaled qui existaient déjà sur St7789v (driver direct), en apportant le même comportement au mode framebuffer sans clignotement.
Exemple d'utilisation :
```rust Double taille (10×14 px par caractère)
ecran.draw_str_scaled_buf(8, 10, b"BONJOUR", Color::WHITE, Color::BLACK, 2);

// Triple taille (15×21 px par caractère)
ecran.draw_str_scaled_buf(8, 40, b"OK", Color::GREEN, Color::BLACK, 3);

// Taille normale — identique à draw_str_buf
ecran.draw_str_scaled_buf(8, 80, b"details", Color::CYAN, Color::BLACK, 1);
```

Note : scale est un entier (u8). Les valeurs décimales (ex. 1.5) ne sont pas
supportées car chaque pixel de la police bitmap doit correspondre à un carré entier
de pixels à l'écran. Pour un rendu intermédiaire, utilisez une police source plus grande.


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