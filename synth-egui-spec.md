# Spécification : Interface Synthétiseur egui (Rust)

> **Document historique (Phase 6)** — Ce document est la spécification originale de l'UI Synthwave, implémentée en v0.6.0. Depuis la v0.7.0, l'architecture a évolué vers des **channel strips per-voice** (8 voix indépendantes) au lieu de contrôles globaux. Pour l'état actuel de l'implémentation, voir [`README.md`](README.md) et [`SPEC.md`](SPEC.md).

## Contexte

Implémenter une interface de synthétiseur dans une application Rust existante utilisant **egui**. Ce document décrit l'IHM à construire. **Pas de génération de son** — uniquement l'interface graphique avec des widgets interactifs.

---

## Dépendances requises

```toml
[dependencies]
eframe = "0.29"
egui = "0.29"
```

> S'adapter à la version d'egui déjà présente dans le projet. Ne pas casser les dépendances existantes.

---

## Architecture des fichiers

Créer les fichiers suivants dans le projet existant (adapter les chemins si nécessaire) :

```
src/
├── synth_ui/
│   ├── mod.rs          # Module principal, struct SynthUI + impl
│   ├── state.rs        # État du synthétiseur (toutes les valeurs)
│   ├── theme.rs        # Couleurs, constantes visuelles, shadows
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── knob.rs     # Widget knob rotatif custom
│   │   ├── vslider.rs  # Slider vertical custom
│   │   ├── level_meter.rs  # VU-mètre vertical
│   │   ├── keyboard.rs # Clavier piano 2 octaves
│   │   └── pads.rs     # Grille de drum pads 4x4
│   └── panels/
│       ├── mod.rs
│       ├── oscillator.rs
│       ├── envelope.rs
│       ├── filter.rs
│       ├── lfo.rs
│       ├── effects.rs
│       ├── master.rs
│       ├── keyboard_panel.rs
│       └── pads_panel.rs
```

---

## Thème & Couleurs

Définir dans `theme.rs` :

```rust
pub struct SynthTheme;

impl SynthTheme {
    pub const BG: Color32 = Color32::from_rgb(13, 13, 26);           // #0d0d1a
    pub const PANEL: Color32 = Color32::from_rgb(26, 26, 46);        // #1a1a2e
    pub const PANEL_LIGHT: Color32 = Color32::from_rgb(34, 34, 58);  // #22223a
    pub const BORDER: Color32 = Color32::from_rgb(42, 42, 74);       // #2a2a4a
    pub const ACCENT: Color32 = Color32::from_rgb(155, 89, 182);     // #9b59b6
    pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(192, 132, 252); // #c084fc
    pub const ACCENT_DARK: Color32 = Color32::from_rgb(124, 58, 237);   // #7c3aed
    pub const TEXT: Color32 = Color32::from_rgb(224, 216, 240);       // #e0d8f0
    pub const TEXT_DIM: Color32 = Color32::from_rgb(136, 120, 169);   // #8878a9
    pub const KNOB_BG: Color32 = Color32::from_rgb(22, 22, 43);      // #16162b
    pub const PAD_IDLE: Color32 = Color32::from_rgb(30, 30, 56);     // #1e1e38
    pub const WHITE_KEY: Color32 = Color32::from_rgb(232, 224, 240);  // #e8e0f0
    pub const BLACK_KEY: Color32 = Color32::from_rgb(26, 26, 46);    // #1a1a2e

    pub const SHADOW_COLOR: Color32 = Color32::from_rgba_premultiplied(124, 58, 237, 64);
    pub const PANEL_ROUNDING: f32 = 10.0;
    pub const PANEL_SHADOW: Shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 128),
    };
}
```

Appliquer un fond global `BG` avec un dégradé radial violet subtil en haut (utiliser `painter.rect_filled` avec un gradient si possible, sinon fond uni).

---

## État global — `state.rs`

```rust
pub struct SynthState {
    // Oscillateur
    pub osc_waveform: Waveform,  // enum { Sine, Saw, Square, Tri }
    pub osc_pitch: f32,          // 0.0..=100.0
    pub osc_detune: f32,         // 0.0..=100.0 (en cents)

    // Enveloppe ADSR
    pub env_attack: f32,         // 0.0..=100.0
    pub env_decay: f32,          // 0.0..=100.0
    pub env_sustain: f32,        // 0.0..=100.0
    pub env_release: f32,        // 0.0..=100.0

    // Filtre
    pub filter_type: FilterType, // enum { LP, HP, BP }
    pub filter_cutoff: f32,      // 0.0..=100.0
    pub filter_resonance: f32,   // 0.0..=100.0

    // LFO
    pub lfo_rate: f32,           // 0.0..=100.0
    pub lfo_depth: f32,          // 0.0..=100.0
    pub lfo_dest: LfoDest,       // enum { Pitch, Filter, Amp }

    // Effets
    pub fx_reverb: f32,          // 0.0..=100.0
    pub fx_delay: f32,           // 0.0..=100.0
    pub fx_chorus: f32,          // 0.0..=100.0

    // Master
    pub master_volume: f32,      // 0.0..=100.0

    // Keyboard & Pads (état d'interaction)
    pub pressed_key: Option<String>,  // ex: "C#4"
    pub pressed_pad: Option<usize>,   // index 0..15
}

#[derive(PartialEq, Clone, Copy)]
pub enum Waveform { Sine, Saw, Square, Tri }

#[derive(PartialEq, Clone, Copy)]
pub enum FilterType { LP, HP, BP }

#[derive(PartialEq, Clone, Copy)]
pub enum LfoDest { Pitch, Filter, Amp }
```

Implémenter `Default` avec des valeurs initiales raisonnables (pitch=50, detune=50, attack=10, decay=30, sustain=70, release=40, cutoff=75, reso=30, reverb=35, delay=20, chorus=15, volume=75).

---

## Widgets custom

### 1. Knob rotatif — `knob.rs`

**Comportement :**
- Cercle avec arc de progression (270° de course, de -135° à +135°)
- Drag vertical pour changer la valeur (monter = augmenter)
- Ligne indicateur du centre vers le bord, point central accent

**Rendu avec `egui::Painter` :**
1. Cercle de fond `KNOB_BG` avec stroke `BORDER`
2. Arc de progression couleur `ACCENT_DARK` (utiliser `painter.add(Shape::Path(...))`)
3. Ligne indicateur `ACCENT_LIGHT`, épaisseur 2px
4. Point central `ACCENT`, rayon 4px
5. Label valeur en dessous (dim), label nom en dessous (bold)

**Signature :**
```rust
pub fn knob(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32, unit: &str) -> Response
```

**Taille :** ~52x52px pour le knob, espace total ~58px large.

**Interaction :** `ui.allocate_rect()` + `sense.drag()`, delta_y mappé à la plage.

---

### 2. Slider vertical — `vslider.rs`

**Comportement :**
- Track vertical 8px large, 90px haut, fond `KNOB_BG`
- Remplissage de bas en haut avec gradient `ACCENT_DARK` → `ACCENT_LIGHT`
- Thumb rectangulaire 16x12px, couleur `PANEL_LIGHT`, bordure `ACCENT`
- Click/drag pour changer la valeur

**Signature :**
```rust
pub fn vslider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) -> Response
```

---

### 3. VU-mètre — `level_meter.rs`

**Comportement :**
- Barre verticale 12px × 90px
- 15 segments empilés
- Couleur selon position : vert/violet (bas) → jaune (65-85%) → rouge (>85%)
- Segments allumés si leur position % ≤ valeur

**Signature :**
```rust
pub fn level_meter(ui: &mut Ui, value: f32) // value 0.0..=100.0, lecture seule
```

---

### 4. Clavier piano — `keyboard.rs`

**Spécifications :**
- 2 octaves : C3 à B4
- 14 touches blanches (largeur 38px, hauteur 120px)
- 10 touches noires (largeur 24px, hauteur 72px, positionnées en overlay)
- Touches blanches : fond `WHITE_KEY` avec léger gradient, coins arrondis en bas (6px)
- Touches noires : fond `BLACK_KEY` avec gradient, coins arrondis en bas (4px)
- Au survol/clic : couleur `ACCENT_LIGHT` (blanches) ou `ACCENT` (noires) + shadow
- Afficher le nom de la note (ex: C3) en bas de chaque touche blanche, police 9px

**Notes par octave (dans l'ordre) :**
```
C, C#, D, D#, E, F, F#, G, G#, A, A#, B
```
Les touches noires : C#, D#, F#, G#, A#.

**Rendu :** Utiliser `painter` pour dessiner les touches. D'abord toutes les blanches, puis les noires par dessus. Gérer le clic avec `ui.allocate_rect()` par touche.

**Interaction :** `mousedown` → `pressed_key = Some("C#4")`, `mouseup` → `pressed_key = None`

---

### 5. Drum Pads — `pads.rs`

**Spécifications :**
- Grille 4×4 = 16 pads
- Labels : `["Kick", "Snare", "HiHat", "Clap", "Tom1", "Tom2", "Rim", "Cow", "Crash", "Ride", "Shaker", "Perc1", "Perc2", "Perc3", "FX1", "FX2"]`
- Chaque pad est carré (aspect ratio 1:1)
- État normal : fond `PAD_IDLE`, bordure `BORDER`
- État pressé : gradient `ACCENT_DARK` → `ACCENT`, bordure `ACCENT_LIGHT`, shadow violette, scale 0.95
- Police du label : 10px, bold

**Interaction :** `mousedown` → actif, `mouseup`/`mouseleave` → inactif

---

## Panels — Layout

### Panel wrapper

Chaque section est enveloppée dans un panel :

```rust
pub fn synth_panel(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::none()
        .fill(SynthTheme::PANEL)
        .stroke(Stroke::new(1.0, SynthTheme::BORDER))
        .rounding(SynthTheme::PANEL_ROUNDING)
        .shadow(SynthTheme::PANEL_SHADOW)
        .inner_margin(Margin::symmetric(16.0, 14.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .size(11.0)
                    .strong()
                    .color(SynthTheme::ACCENT_LIGHT)
                    .text_style(TextStyle::Small)
            );
            ui.add_space(12.0);
            add_contents(ui);
        });
}
```

### Widget sélection boutons (pour Waveform, FilterType, LfoDest)

Rangée horizontale de boutons toggle :

```rust
pub fn select_buttons<T: PartialEq + Copy>(
    ui: &mut Ui,
    current: &mut T,
    options: &[(T, &str)],
)
```

- Bouton sélectionné : fond `ACCENT_DARK`, texte blanc, bordure `ACCENT`, shadow
- Bouton non sélectionné : fond `KNOB_BG`, texte `TEXT_DIM`, bordure `BORDER`
- Padding 4×10px, border-radius 5px, font 10px bold

### Slider horizontal (pour Filter, Effects)

```rust
pub fn hslider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) -> Response
```

- Label à gauche (50px, aligné droite, 10px, bold, `TEXT`)
- Track : hauteur 8px, fond `KNOB_BG`, border `BORDER`, rounding 4px
- Fill : gradient horizontal `ACCENT_DARK` → `ACCENT_LIGHT`
- Thumb : 12×16px, fond `PANEL_LIGHT`, bordure `ACCENT`, rounding 3px
- Valeur à droite (30px, 10px, `TEXT_DIM`)

---

## Layout principal — `mod.rs`

```
┌─────────────────────────────────────────────────────────────────┐
│ ● SYNTHWAVE                                          v1.0      │
├──────────┬──────────┬──────────┬──────────┬────────────┬───────┤
│          │ Envelope │          │          │            │       │
│  Oscill. │  (ADSR)  │  Filter  │   LFO    │  Effects   │Master │
│          │          │          │          │            │       │
├────────────────────────────────────────────────┬──────────────┤
│                                                │              │
│               Keyboard (2 octaves)             │  Drum Pads   │
│                                                │   (4×4)      │
└────────────────────────────────────────────────┴──────────────┘
```

**Implémentation layout :**

```rust
impl SynthUI {
    pub fn show(&mut self, ui: &mut Ui) {
        // Header
        ui.horizontal(|ui| {
            // Indicateur LED (cercle 10px ACCENT_LIGHT avec glow)
            // Titre "SYNTHWAVE" taille 22, bold, ACCENT_LIGHT, letter-spacing 4
            // Spacer
            // "v1.0 — EGUI READY" taille 10, TEXT_DIM
        });
        ui.add_space(20.0);

        // Rangée du haut : 6 panels
        // Utiliser ui.columns() ou egui_extras::StripBuilder si disponible
        // Proportions colonnes : 1 | 1 | 1 | 1 | 1.2 | 0.6
        ui.horizontal(|ui| {
            // Oscillator panel
            // ADSR panel
            // Filter panel
            // LFO panel
            // Effects panel
            // Master panel
        });
        ui.add_space(12.0);

        // Rangée du bas : keyboard + pads
        // Proportions : ~70% keyboard, ~30% pads
        ui.horizontal(|ui| {
            // Keyboard panel (avec scroll horizontal si nécessaire)
            // Drum Pads panel
        });

        // Footer
        ui.add_space(16.0);
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("DRAG KNOBS VERTICALLY • CLICK KEYS & PADS TO PLAY")
                    .size(10.0).color(SynthTheme::TEXT_DIM)
            );
        });
    }
}
```

---

## Détails de chaque panel

### Oscillator
- `select_buttons` pour Waveform : Sine, Saw, Square, Tri
- 12px d'espace
- Rangée horizontale centrée : `knob("Pitch", &mut state.osc_pitch, 0, 100, "")` + `knob("Detune", &mut state.osc_detune, 0, 100, "ct")`

### Envelope (ADSR)
- Rangée horizontale centrée avec gap 12px
- 4 × `vslider` : A, D, S, R

### Filter
- `select_buttons` pour FilterType : LP, HP, BP
- 14px d'espace
- `hslider("Cutoff", ...)` + `hslider("Reso", ...)`

### LFO
- `select_buttons` pour LfoDest : Pitch, Filter, Amp
- 14px d'espace
- Rangée horizontale centrée : `knob("Rate", ..., "Hz")` + `knob("Depth", ...)`

### Effects
- 3 × `hslider` : Reverb, Delay, Chorus

### Master
- Rangée horizontale centrée
- `vslider("Vol", ...)` + `level_meter(state.master_volume * 0.9)` label "L" + `level_meter(state.master_volume * 0.82)` label "R"

---

## Intégration dans l'app existante

Exposer une méthode publique :

```rust
// Dans synth_ui/mod.rs
pub struct SynthUI {
    state: SynthState,
}

impl SynthUI {
    pub fn new() -> Self {
        Self { state: SynthState::default() }
    }

    /// Appeler depuis la boucle de rendu egui existante
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // ... tout le layout décrit ci-dessus
    }

    /// Accès en lecture à l'état (pour connecter au moteur audio plus tard)
    pub fn state(&self) -> &SynthState {
        &self.state
    }
}
```

L'application existante appelle :
```rust
// Quelque part dans le code existant
let mut synth_ui = SynthUI::new();
// Dans la boucle de rendu :
synth_ui.show(ui);
```

---

## Raccourcis clavier

### Mapping clavier → notes (style AZERTY)

Le clavier physique contrôle le piano. Deux rangées pour couvrir les 2 octaves :

**Octave 3 (rangée basse) :**

| Touche | Note |
|--------|------|
| `W`    | C3   |
| `S`    | C#3  |
| `X`    | D3   |
| `D`    | D#3  |
| `C`    | E3   |
| `V`    | F3   |
| `G`    | F#3  |
| `B`    | G3   |
| `H`    | G#3  |
| `N`    | A3   |
| `J`    | A#3  |
| `,`    | B3   |

**Octave 4 (rangée haute) :**

| Touche | Note |
|--------|------|
| `A`    | C4   |
| `É (2)`| C#4  |
| `Z`    | D4   |
| `" (3)`| D#4  |
| `E`    | E4   |
| `R`    | F4   |
| `( (5)`| F#4  |
| `T`    | G4   |
| `- (6)`| G#4  |
| `Y`    | A4   |
| `È (7)`| A#4  |
| `U`    | B4   |

**Implémentation :**

```rust
// Dans synth_ui/mod.rs ou un fichier dédié keyboard_shortcuts.rs

use egui::Key;

pub struct KeyMapping {
    pub key: Key,
    pub note: &'static str,
}

/// Retourne le mapping complet clavier physique → note
pub fn get_key_mappings() -> Vec<KeyMapping> {
    vec![
        // Octave 3
        KeyMapping { key: Key::W, note: "C3" },
        KeyMapping { key: Key::S, note: "C#3" },
        KeyMapping { key: Key::X, note: "D3" },
        KeyMapping { key: Key::D, note: "D#3" },
        KeyMapping { key: Key::C, note: "E3" },
        KeyMapping { key: Key::V, note: "F3" },
        KeyMapping { key: Key::G, note: "F#3" },
        KeyMapping { key: Key::B, note: "G3" },
        KeyMapping { key: Key::H, note: "G#3" },
        KeyMapping { key: Key::N, note: "A3" },
        KeyMapping { key: Key::J, note: "A#3" },
        KeyMapping { key: Key::Comma, note: "B3" },
        // Octave 4
        KeyMapping { key: Key::A, note: "C4" },
        KeyMapping { key: Key::Num2, note: "C#4" },
        KeyMapping { key: Key::Z, note: "D4" },
        KeyMapping { key: Key::Num3, note: "D#4" },
        KeyMapping { key: Key::E, note: "E4" },
        KeyMapping { key: Key::R, note: "F4" },
        KeyMapping { key: Key::Num5, note: "F#4" },
        KeyMapping { key: Key::T, note: "G4" },
        KeyMapping { key: Key::Num6, note: "G#4" },
        KeyMapping { key: Key::Y, note: "A4" },
        KeyMapping { key: Key::Num7, note: "A#4" },
        KeyMapping { key: Key::U, note: "B4" },
    ]
}
```

**Gestion dans la boucle de rendu :**

```rust
// Dans SynthUI::show()
// Parcourir les mappings et vérifier les événements clavier
for mapping in get_key_mappings() {
    if ui.input(|i| i.key_pressed(mapping.key)) {
        self.state.pressed_key = Some(mapping.note.to_string());
    }
    if ui.input(|i| i.key_released(mapping.key)) {
        if self.state.pressed_key.as_deref() == Some(mapping.note) {
            self.state.pressed_key = None;
        }
    }
}
```

Le widget `keyboard.rs` doit aussi refléter visuellement les touches pressées via le clavier physique (utiliser `state.pressed_key` pour le highlight).

### Raccourcis pads (pavé numérique + chiffres)

| Touche     | Pad    |
|------------|--------|
| `Num1`     | Kick   |
| `Num2`     | Snare  |
| `Num3`     | HiHat  |
| `Num4`     | Clap   |
| `Num5`     | Tom1   |
| `Num6`     | Tom2   |
| `Num7`     | Rim    |
| `Num8`     | Cow    |
| `Num9`     | Crash  |
| `Num0`     | Ride   |
| `F1`       | Shaker |
| `F2`       | Perc1  |
| `F3`       | Perc2  |
| `F4`       | Perc3  |
| `F5`       | FX1    |
| `F6`       | FX2    |

**Même logique :** `key_pressed` → `pressed_pad = Some(index)`, `key_released` → `None`. Le widget `pads.rs` reflète visuellement l'état.

### Raccourcis globaux

| Raccourci         | Action                              |
|-------------------|-------------------------------------|
| `Ctrl + ↑`        | Volume master +5                    |
| `Ctrl + ↓`        | Volume master -5                    |
| `Ctrl + 1`        | Waveform → Sine                     |
| `Ctrl + 2`        | Waveform → Saw                      |
| `Ctrl + 3`        | Waveform → Square                   |
| `Ctrl + 4`        | Waveform → Tri                      |
| `Tab`             | Cycle filter type (LP → HP → BP)    |
| `Shift + Tab`     | Cycle LFO destination               |
| `Space`           | Panic / all notes off (reset pressed_key & pressed_pad) |

**Implémentation des raccourcis globaux :**

```rust
// Dans SynthUI::show(), avant le rendu des panels
let modifiers = ui.input(|i| i.modifiers);

if modifiers.ctrl {
    if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
        self.state.master_volume = (self.state.master_volume + 5.0).min(100.0);
    }
    if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
        self.state.master_volume = (self.state.master_volume - 5.0).max(0.0);
    }
    if ui.input(|i| i.key_pressed(Key::Num1)) {
        self.state.osc_waveform = Waveform::Sine;
    }
    // ... etc
}

if ui.input(|i| i.key_pressed(Key::Tab)) {
    if modifiers.shift {
        self.state.lfo_dest = match self.state.lfo_dest {
            LfoDest::Pitch => LfoDest::Amp,
            LfoDest::Filter => LfoDest::Pitch,
            LfoDest::Amp => LfoDest::Filter,
        };
    } else {
        self.state.filter_type = match self.state.filter_type {
            FilterType::LP => FilterType::HP,
            FilterType::HP => FilterType::BP,
            FilterType::BP => FilterType::LP,
        };
    }
}

if ui.input(|i| i.key_pressed(Key::Space)) {
    self.state.pressed_key = None;
    self.state.pressed_pad = None;
}
```

### Support multi-touches

L'état `pressed_key` doit supporter plusieurs notes simultanées pour permettre les accords :

```rust
// Remplacer dans state.rs :
pub pressed_key: Option<String>,
// Par :
pub pressed_keys: HashSet<String>,

// Remplacer dans state.rs :
pub pressed_pad: Option<usize>,
// Par :
pub pressed_pads: HashSet<usize>,
```

Adapter la logique : `key_pressed` → `insert()`, `key_released` → `remove()`, `Space` → `clear()`.

---

## Résumé des contraintes

1. **Pas de son** — IHM uniquement, l'état est exposé en lecture
2. **Tous les widgets sont custom** via `egui::Painter` — pas de dépendances UI tierces
3. **Thème sombre violet** cohérent, shadows sur tous les panels
4. **Interactions fluides** — drag vertical pour knobs, click/drag pour sliders, click pour pads/clavier
5. **Code modulaire** — chaque widget et panel dans son propre fichier
6. **Ne pas casser l'existant** — ajouter le module `synth_ui` proprement, adapter les imports
7. **Compatible egui 0.28+** — utiliser les API stables
