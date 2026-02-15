# Tenori-Synth — Spécification complète d'implémentation

> **Destinataire** : Claude Code (agent IA)
> **Langage** : Rust
> **Framework UI** : egui (via eframe)
> **Objectif** : Ajouter un module `tenori_synth` dans un projet Rust/egui existant. Ce module implémente une interface de type Tenori-on (séquenceur matriciel 16×16). **Aucune génération de son** — uniquement l'IHM. L'état est exposé pour branchement futur sur un moteur audio.

---

## TABLE DES MATIÈRES

1. [Règles d'intégration](#1-règles-dintégration)
2. [Dépendances](#2-dépendances)
3. [Architecture fichiers](#3-architecture-fichiers)
4. [Thème et constantes visuelles](#4-thème-et-constantes-visuelles)
5. [État global](#5-état-global)
6. [Module principal et layout](#6-module-principal-et-layout)
7. [Widget : Knob rotatif](#7-widget--knob-rotatif)
8. [Widget : Slider horizontal](#8-widget--slider-horizontal)
9. [Widget : Select buttons](#9-widget--select-buttons)
10. [Widget : Panel wrapper](#10-widget--panel-wrapper)
11. [Grille 16×16](#11-grille-16×16)
12. [Barre de transport](#12-barre-de-transport)
13. [Barre de densité](#13-barre-de-densité)
14. [Header](#14-header)
15. [Panneaux latéraux](#15-panneaux-latéraux)
16. [Raccourcis clavier](#16-raccourcis-clavier)
17. [Undo / Redo](#17-undo--redo)
18. [API publique](#18-api-publique)
19. [Checklist finale](#19-checklist-finale)

---

## 1. Règles d'intégration

- **Ne JAMAIS modifier les fichiers existants** sauf pour ajouter `mod tenori_synth;` et l'appel `tenori.show(ctx, ui);` dans le code appelant.
- **Ne pas casser les dépendances** — vérifier la version d'egui/eframe dans `Cargo.toml` existant et s'adapter.
- **Tout le code nouveau** va dans `src/tenori_synth/`.
- **Pas de dépendances tierces supplémentaires** — uniquement egui/eframe et la std.
- **Compiler sans warnings** — `#![deny(warnings)]` compatible.
- **Pas de `unsafe`**.

---

## 2. Dépendances

Le projet utilise déjà egui/eframe. Vérifier `Cargo.toml` pour la version. Le code ci-dessous cible **egui 0.28+** / **eframe 0.28+**. Si la version est antérieure, adapter les API (`Shadow`, `Margin`, etc.).

Aucune dépendance supplémentaire n'est requise.

---

## 3. Architecture fichiers

Créer l'arborescence suivante :

```
src/
└── tenori_synth/
    ├── mod.rs                  # Struct TenoriSynth, impl show(), API publique
    ├── state.rs                # TenoriState, enums, Default, constantes
    ├── theme.rs                # Struct Theme, toutes les couleurs et dimensions
    ├── grid.rs                 # Rendu et interaction de la matrice 16×16
    ├── transport.rs            # Barre play/pause/clear/BPM/swing
    ├── density_bar.rs          # Barre de densité sous la grille
    ├── header.rs               # Header avec titre, LED, notes actives
    ├── shortcuts.rs            # Gestion de tous les raccourcis clavier
    ├── history.rs              # Système undo/redo (historique des grilles)
    ├── widgets/
    │   ├── mod.rs              # Re-exports
    │   ├── knob.rs             # Knob rotatif custom (Painter)
    │   ├── hslider.rs          # Slider horizontal custom (Painter)
    │   ├── select_buttons.rs   # Groupe de boutons toggle
    │   └── panel.rs            # Frame wrapper pour les panels
    └── panels/
        ├── mod.rs              # Re-exports
        ├── oscillator.rs       # Panel oscillateur
        ├── envelope.rs         # Panel ADSR
        ├── filter.rs           # Panel filtre
        ├── lfo.rs              # Panel LFO
        ├── effects.rs          # Panel effets
        ├── scale.rs            # Panel sélection de gamme
        └── draw_mode.rs        # Panel mode de dessin
```

Ajouter dans le fichier `src/main.rs` (ou `src/lib.rs`, ou là où se trouve la boucle egui) :

```rust
mod tenori_synth;
use tenori_synth::TenoriSynth;
```

---

## 4. Thème et constantes visuelles

**Fichier : `theme.rs`**

```rust
use egui::{Color32, Rounding, Shadow, Stroke, Vec2};

pub struct Theme;

impl Theme {
    // ── Fonds ──
    pub const BG: Color32            = Color32::from_rgb(13, 13, 26);
    pub const BG_GRADIENT_TOP: Color32 = Color32::from_rgba_premultiplied(124, 58, 237, 20);
    pub const PANEL: Color32         = Color32::from_rgb(26, 26, 46);
    pub const PANEL_LIGHT: Color32   = Color32::from_rgb(34, 34, 58);
    pub const BORDER: Color32        = Color32::from_rgb(42, 42, 74);

    // ── Accents ──
    pub const ACCENT: Color32        = Color32::from_rgb(155, 89, 182);
    pub const ACCENT_LIGHT: Color32  = Color32::from_rgb(192, 132, 252);
    pub const ACCENT_DARK: Color32   = Color32::from_rgb(124, 58, 237);

    // ── Textes ──
    pub const TEXT: Color32          = Color32::from_rgb(224, 216, 240);
    pub const TEXT_DIM: Color32      = Color32::from_rgb(136, 120, 169);
    pub const TEXT_WHITE: Color32    = Color32::WHITE;

    // ── Grille ──
    pub const CELL_OFF: Color32          = Color32::from_rgb(20, 20, 40);
    pub const CELL_OFF_HOVER: Color32    = Color32::from_rgb(28, 28, 52);
    pub const CELL_ON: Color32           = Color32::from_rgb(124, 58, 237);
    pub const CELL_ON_HOVER: Color32     = Color32::from_rgb(140, 75, 245);
    pub const CELL_PLAYHEAD: Color32     = Color32::from_rgb(26, 26, 56);
    pub const CELL_PLAYHEAD_BORDER: Color32 = Color32::from_rgb(58, 58, 90);
    pub const CELL_HIT: Color32          = Color32::from_rgb(233, 213, 255);
    pub const CELL_HIT_GLOW: Color32     = Color32::from_rgba_premultiplied(192, 132, 252, 40);
    pub const PLAYHEAD_BG: Color32       = Color32::from_rgba_premultiplied(192, 132, 252, 38);

    // ── Widgets ──
    pub const KNOB_BG: Color32       = Color32::from_rgb(22, 22, 43);
    pub const LED_PLAYING: Color32   = Color32::from_rgb(74, 222, 128);
    pub const LED_PLAYING_GLOW: Color32 = Color32::from_rgba_premultiplied(74, 222, 128, 128);
    pub const LED_STOPPED: Color32   = Color32::from_rgb(192, 132, 252);

    // ── Dimensions ──
    pub const CELL_SIZE: f32         = 32.0;
    pub const CELL_GAP: f32          = 2.0;
    pub const CELL_ROUNDING: f32     = 4.0;
    pub const PANEL_ROUNDING: f32    = 10.0;
    pub const SIDEBAR_WIDTH: f32     = 210.0;
    pub const NOTE_LABEL_WIDTH: f32  = 38.0;
    pub const COL_HEADER_HEIGHT: f32 = 16.0;
    pub const TRANSPORT_HEIGHT: f32  = 48.0;
    pub const DENSITY_HEIGHT: f32    = 10.0;

    // ── Shadows ──
    pub fn panel_shadow() -> Shadow {
        Shadow {
            offset: Vec2::new(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 128),
        }
    }

    pub fn glow_shadow() -> Shadow {
        Shadow {
            offset: Vec2::ZERO,
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_rgba_premultiplied(124, 58, 237, 64),
        }
    }

    pub fn panel_stroke() -> Stroke {
        Stroke::new(1.0, Self::BORDER)
    }

    pub fn panel_rounding() -> Rounding {
        Rounding::same(Self::PANEL_ROUNDING)
    }
}
```

---

## 5. État global

**Fichier : `state.rs`**

```rust
pub const ROWS: usize = 16;
pub const COLS: usize = 16;

/// Notes affichées de haut (aigu) en bas (grave)
pub const NOTE_LABELS: [&str; ROWS] = [
    "C5", "B4", "A#4", "A4", "G#4", "G4", "F#4", "F4",
    "E4", "D#4", "D4", "C#4", "C4", "B3", "A#3", "A3",
];

/// Intervalles en demi-tons par gamme (depuis la note la plus grave)
pub const SCALE_CHROMATIC: [u8; 16]  = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
pub const SCALE_MAJOR: [u8; 16]      = [0,2,4,5,7,9,11,12,14,16,17,19,21,23,24,26];
pub const SCALE_MINOR: [u8; 16]      = [0,2,3,5,7,8,10,12,14,15,17,19,20,22,24,26];
pub const SCALE_PENTATONIC: [u8; 16] = [0,2,4,7,9,12,14,16,19,21,24,26,28,31,33,36];

// ── Enums ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform { Sine, Saw, Square, Tri }

impl Waveform {
    pub const ALL: [Waveform; 4] = [Self::Sine, Self::Saw, Self::Square, Self::Tri];
    pub fn label(&self) -> &'static str {
        match self { Self::Sine => "Sine", Self::Saw => "Saw", Self::Square => "Square", Self::Tri => "Tri" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType { LP, HP, BP }

impl FilterType {
    pub const ALL: [FilterType; 3] = [Self::LP, Self::HP, Self::BP];
    pub fn label(&self) -> &'static str {
        match self { Self::LP => "LP", Self::HP => "HP", Self::BP => "BP" }
    }
    pub fn next(&self) -> Self {
        match self { Self::LP => Self::HP, Self::HP => Self::BP, Self::BP => Self::LP }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoDest { Pitch, Filter, Amp }

impl LfoDest {
    pub const ALL: [LfoDest; 3] = [Self::Pitch, Self::Filter, Self::Amp];
    pub fn label(&self) -> &'static str {
        match self { Self::Pitch => "Pitch", Self::Filter => "Filter", Self::Amp => "Amp" }
    }
    pub fn next(&self) -> Self {
        match self { Self::Pitch => Self::Filter, Self::Filter => Self::Amp, Self::Amp => Self::Pitch }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode { Toggle, Draw, Erase }

impl DrawMode {
    pub const ALL: [DrawMode; 3] = [Self::Toggle, Self::Draw, Self::Erase];
    pub fn label(&self) -> &'static str {
        match self { Self::Toggle => "Toggle", Self::Draw => "Draw", Self::Erase => "Erase" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale { Chromatic, Major, Minor, Pentatonic }

impl Scale {
    pub const ALL: [Scale; 4] = [Self::Chromatic, Self::Major, Self::Minor, Self::Pentatonic];
    pub fn label(&self) -> &'static str {
        match self {
            Self::Chromatic => "Chromatic", Self::Major => "Major",
            Self::Minor => "Minor", Self::Pentatonic => "Pentatonic",
        }
    }
    pub fn intervals(&self) -> &'static [u8; 16] {
        match self {
            Self::Chromatic => &SCALE_CHROMATIC, Self::Major => &SCALE_MAJOR,
            Self::Minor => &SCALE_MINOR, Self::Pentatonic => &SCALE_PENTATONIC,
        }
    }
}

// ── État principal ──

#[derive(Clone)]
pub struct TenoriState {
    // Matrice
    pub grid: [[bool; COLS]; ROWS],

    // Transport
    pub is_playing: bool,
    pub play_col: i32,       // -1 = arrêté, 0..15 = colonne active
    pub bpm: f32,            // 40.0 ..= 240.0
    pub swing: f32,          // 0.0 ..= 100.0
    pub elapsed_secs: f64,   // accumulateur pour avancer le step

    // Draw mode
    pub draw_mode: DrawMode,

    // Scale
    pub scale: Scale,

    // Oscillateur
    pub osc_waveform: Waveform,
    pub osc_pitch: f32,      // 0.0 ..= 100.0
    pub osc_detune: f32,     // 0.0 ..= 100.0  (cents)

    // ADSR
    pub env_attack: f32,     // 0.0 ..= 100.0
    pub env_decay: f32,
    pub env_sustain: f32,
    pub env_release: f32,

    // Filtre
    pub filter_type: FilterType,
    pub filter_cutoff: f32,  // 0.0 ..= 100.0
    pub filter_reso: f32,

    // LFO
    pub lfo_rate: f32,
    pub lfo_depth: f32,
    pub lfo_dest: LfoDest,

    // Effets
    pub fx_reverb: f32,
    pub fx_delay: f32,
    pub fx_chorus: f32,
}

impl Default for TenoriState {
    fn default() -> Self {
        Self {
            grid: [[false; COLS]; ROWS],
            is_playing: false,
            play_col: -1,
            bpm: 120.0,
            swing: 0.0,
            elapsed_secs: 0.0,
            draw_mode: DrawMode::Toggle,
            scale: Scale::Chromatic,
            osc_waveform: Waveform::Saw,
            osc_pitch: 50.0,
            osc_detune: 0.0,
            env_attack: 10.0,
            env_decay: 30.0,
            env_sustain: 70.0,
            env_release: 40.0,
            filter_type: FilterType::LP,
            filter_cutoff: 75.0,
            filter_reso: 30.0,
            lfo_rate: 30.0,
            lfo_depth: 50.0,
            lfo_dest: LfoDest::Filter,
            fx_reverb: 35.0,
            fx_delay: 20.0,
            fx_chorus: 15.0,
        }
    }
}

impl TenoriState {
    pub fn clear_grid(&mut self) {
        self.grid = [[false; COLS]; ROWS];
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            self.play_col = 0;
            self.elapsed_secs = 0.0;
        } else {
            self.play_col = -1;
        }
    }

    pub fn toggle_row(&mut self, row: usize) {
        let all_on = self.grid[row].iter().all(|&v| v);
        for col in 0..COLS {
            self.grid[row][col] = !all_on;
        }
    }

    pub fn toggle_col(&mut self, col: usize) {
        let all_on = (0..ROWS).all(|r| self.grid[r][col]);
        for row in 0..ROWS {
            self.grid[row][col] = !all_on;
        }
    }

    /// Indices des lignes actives à la colonne courante
    pub fn active_rows(&self) -> Vec<usize> {
        if self.play_col < 0 { return vec![]; }
        let col = self.play_col as usize;
        (0..ROWS).filter(|&r| self.grid[r][col]).collect()
    }

    /// Noms des notes actives à la colonne courante
    pub fn active_note_names(&self) -> Vec<&'static str> {
        self.active_rows().iter().map(|&r| NOTE_LABELS[r]).collect()
    }

    /// Nombre de cellules actives dans une colonne
    pub fn col_density(&self, col: usize) -> usize {
        (0..ROWS).filter(|&r| self.grid[r][col]).count()
    }
}
```

---

## 6. Module principal et layout

**Fichier : `mod.rs`**

```rust
mod state;
mod theme;
mod grid;
mod transport;
mod density_bar;
mod header;
mod shortcuts;
mod history;
mod widgets;
mod panels;

pub use state::TenoriState;

use state::*;
use theme::Theme;
use history::History;

pub struct TenoriSynth {
    state: TenoriState,
    history: History,
}

impl TenoriSynth {
    pub fn new() -> Self {
        Self {
            state: TenoriState::default(),
            history: History::new(),
        }
    }

    /// Méthode principale — appeler depuis la boucle egui existante.
    /// `ctx` est nécessaire pour request_repaint() et le delta time.
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // 0. Fond global
        let screen = ui.max_rect();
        ui.painter().rect_filled(screen, 0.0, Theme::BG);
        // Dégradé radial violet subtil en haut (optionnel)
        ui.painter().rect_filled(
            egui::Rect::from_min_size(screen.min, egui::vec2(screen.width(), 300.0)),
            0.0,
            Theme::BG_GRADIENT_TOP,
        );

        // 1. Playhead
        self.update_playhead(ctx);

        // 2. Raccourcis clavier (avant le rendu pour réactivité)
        shortcuts::handle(ui, &mut self.state, &mut self.history);

        // 3. Layout
        ui.vertical(|ui| {
            // Header
            header::draw(ui, &self.state);
            ui.add_space(14.0);

            // Corps : sidebar gauche + zone principale droite
            ui.horizontal_top(|ui| {
                // ── Sidebar gauche (largeur fixe) ──
                ui.allocate_ui_with_layout(
                    egui::vec2(Theme::SIDEBAR_WIDTH, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(Theme::SIDEBAR_WIDTH);
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                panels::oscillator::draw(ui, &mut self.state);
                                panels::envelope::draw(ui, &mut self.state);
                                panels::filter::draw(ui, &mut self.state);
                                panels::lfo::draw(ui, &mut self.state);
                                panels::effects::draw(ui, &mut self.state);
                                panels::scale::draw(ui, &mut self.state);
                                panels::draw_mode::draw(ui, &mut self.state);
                            });
                    },
                );

                ui.add_space(12.0);

                // ── Zone principale droite ──
                ui.vertical(|ui| {
                    // Transport
                    transport::draw(ui, &mut self.state, &mut self.history);
                    ui.add_space(8.0);

                    // Grille
                    grid::draw(ui, &mut self.state, &mut self.history);
                    ui.add_space(8.0);

                    // Densité
                    density_bar::draw(ui, &self.state);
                });
            });

            ui.add_space(12.0);

            // Footer
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("CLICK CELLS TO ACTIVATE · SPACE PLAY/PAUSE · D DRAW · E ERASE · T TOGGLE · C CLEAR")
                        .size(9.0)
                        .color(Theme::TEXT_DIM)
                );
            });
        });

        // 4. Repaint continu si en lecture
        if self.state.is_playing {
            ctx.request_repaint();
        }
    }

    fn update_playhead(&mut self, ctx: &egui::Context) {
        if !self.state.is_playing { return; }

        let dt = ctx.input(|i| i.stable_dt) as f64;
        self.state.elapsed_secs += dt;

        // Durée d'un step = 1 noire / 4 = 1/16e note
        let step_secs = 60.0 / self.state.bpm as f64 / 4.0;

        // TODO: appliquer le swing ici (décaler les steps pairs)
        // Pour swing > 0 : steps pairs durent step_secs*(1+swing_ratio),
        //                   steps impairs durent step_secs*(1-swing_ratio)

        while self.state.elapsed_secs >= step_secs {
            self.state.elapsed_secs -= step_secs;
            self.state.play_col = (self.state.play_col + 1) % COLS as i32;
        }
    }

    // ── API publique ──

    pub fn state(&self) -> &TenoriState { &self.state }
    pub fn state_mut(&mut self) -> &mut TenoriState { &mut self.state }
    pub fn active_note_names(&self) -> Vec<&'static str> { self.state.active_note_names() }
    pub fn active_rows(&self) -> Vec<usize> { self.state.active_rows() }
    pub fn is_playing(&self) -> bool { self.state.is_playing }
    pub fn current_step(&self) -> i32 { self.state.play_col }
}
```

---

## 7. Widget : Knob rotatif

**Fichier : `widgets/knob.rs`**

### Comportement

- Cercle de fond avec arc de progression (270° de course)
- Drag vertical : monter = augmenter, descendre = diminuer
- Sensibilité : 150 pixels de drag = plage complète
- Double-clic : reset à la valeur par défaut (paramètre `default`)

### Dimensions

Le paramètre `size` contrôle le diamètre du cercle. L'espace total occupé = `size + 8` en largeur.

### Rendu (avec `egui::Painter`)

1. Cercle de fond : rayon `size/2 - 4`, couleur `KNOB_BG`, stroke `BORDER` 1.5px
2. Arc de progression : même rayon - 2px, couleur `ACCENT_DARK`, épaisseur 2.5px, strokeLinecap round.
   L'arc part de l'angle -225° (= 135° en bas à gauche) et couvre `pct * 270°` (pct = proportion 0..1).
   **Implémenter avec** : calculer N points sur l'arc (ex: 32 segments) et dessiner un `PathShape` avec stroke.
3. Ligne indicateur : du centre vers le bord, couleur `ACCENT_LIGHT`, épaisseur 1.5px, linecap round.
   L'angle de la ligne = `-135° + pct * 270°` (0 en bas-gauche, 100% en bas-droite).
4. Point central : cercle rayon 3px, couleur `ACCENT`.
5. Sous le knob (espacement 2px) : valeur arrondie + unité, 9px, `TEXT_DIM`.
6. Sous la valeur (espacement 0px) : label, 9px, bold, `TEXT`.

### Signature

```rust
pub struct KnobResponse {
    pub changed: bool,
}

pub fn knob(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    default: f32,
    unit: &str,
    size: f32,    // diamètre en pixels (ex: 36.0, 40.0, 44.0)
) -> KnobResponse {
    let total_height = size + 2.0 + 12.0 + 12.0; // knob + gap + value_text + label_text
    let total_width = size + 8.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(total_width, total_height),
        egui::Sense::click_and_drag(),
    );

    // Double-clic → reset
    if response.double_clicked() {
        *value = default;
    }

    // Drag vertical
    if response.dragged() {
        let delta = -response.drag_delta().y; // monter = positif
        let range = max - min;
        *value = (*value + delta * range / 150.0).clamp(min, max);
    }

    // Rendu avec ui.painter()
    let painter = ui.painter_at(rect);
    let cx = rect.center_top() + egui::vec2(0.0, size / 2.0);
    // ... dessiner cercle, arc, ligne, point, textes

    KnobResponse { changed: response.dragged() || response.double_clicked() }
}
```

---

## 8. Widget : Slider horizontal

**Fichier : `widgets/hslider.rs`**

### Rendu

```
[Label 42px] [═══════●══════] [Val 24px]
               ↑ track 6px height
```

- **Track** : rect hauteur 6px, fond `KNOB_BG`, bordure `BORDER` 1px, rounding 3px
- **Fill** : de gauche à la position courante, gradient `ACCENT_DARK` → `ACCENT_LIGHT` (si gradient non faisable simplement, utiliser `ACCENT_DARK` uni). Rounding 3px.
- **Thumb** : rect 10×12px, fond `PANEL_LIGHT`, bordure `ACCENT` 1px, rounding 2px, centré sur la position courante.
- **Label** : à gauche, largeur 42px, aligné droite, 9px, bold, `TEXT`.
- **Valeur** : à droite, largeur 24px, 9px, `TEXT_DIM`. Afficher l'entier arrondi.
- **Espacement vertical** entre sliders : 5px (margin bottom).

### Interaction

- Click : positionner le thumb directement à la position du clic
- Drag : déplacer le thumb horizontalement

### Signature

```rust
pub fn hslider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
) -> egui::Response {
    let desired_height = 14.0; // track + thumb overflow
    let available_width = ui.available_width();

    ui.horizontal(|ui| {
        // Label
        ui.allocate_ui(egui::vec2(42.0, desired_height), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(label).size(9.0).strong().color(Theme::TEXT));
            });
        });

        // Track (le reste de la largeur - 30px pour la valeur)
        let track_width = available_width - 42.0 - 30.0 - 16.0; // marges
        let (track_rect, response) = ui.allocate_exact_size(
            egui::vec2(track_width, desired_height),
            egui::Sense::click_and_drag(),
        );

        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let pct = ((pos.x - track_rect.left()) / track_rect.width()).clamp(0.0, 1.0);
                *value = min + pct * (max - min);
            }
        }

        // Dessiner track, fill, thumb avec painter

        // Valeur
        ui.allocate_ui(egui::vec2(24.0, desired_height), |ui| {
            ui.label(egui::RichText::new(format!("{}", *value as i32)).size(9.0).color(Theme::TEXT_DIM));
        });
    });

    // Retourner la response du track
}
```

---

## 9. Widget : Select buttons

**Fichier : `widgets/select_buttons.rs`**

Groupe horizontal de boutons toggle mutuellement exclusifs.

### Rendu par bouton

- **Actif** : fond `ACCENT_DARK`, texte `TEXT_WHITE`, bordure `ACCENT` 1px, shadow douce
- **Inactif** : fond `KNOB_BG`, texte `TEXT_DIM`, bordure `BORDER` 1px
- **Hover inactif** : fond légèrement éclairci
- Padding : 3px vertical, 7px horizontal
- Font : 9px, bold
- Rounding : 4px
- Gap entre boutons : 2px

### Signature

```rust
/// T doit implémenter PartialEq + Copy.
/// `options` est un slice de (valeur, label).
pub fn select_buttons<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    current: &mut T,
    options: &[(T, &str)],
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for &(val, label) in options {
            let is_active = *current == val;
            let btn = egui::Button::new(
                egui::RichText::new(label)
                    .size(9.0)
                    .strong()
                    .color(if is_active { Theme::TEXT_WHITE } else { Theme::TEXT_DIM })
            )
            .fill(if is_active { Theme::ACCENT_DARK } else { Theme::KNOB_BG })
            .stroke(Stroke::new(1.0, if is_active { Theme::ACCENT } else { Theme::BORDER }))
            .rounding(4.0)
            .min_size(egui::vec2(0.0, 20.0));

            if ui.add(btn).clicked() {
                *current = val;
            }
        }
    });
}
```

---

## 10. Widget : Panel wrapper

**Fichier : `widgets/panel.rs`**

```rust
pub fn synth_panel(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(Theme::PANEL)
        .stroke(Theme::panel_stroke())
        .rounding(Theme::panel_rounding())
        .shadow(Theme::panel_shadow())
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(10.0)
                    .strong()
                    .color(Theme::ACCENT_LIGHT)
            );
            ui.add_space(10.0);
            add_contents(ui);
        });
    ui.add_space(10.0); // espacement entre panels
}
```

---

## 11. Grille 16×16

**Fichier : `grid.rs`**

### Dimensions totales

```
Largeur  = NOTE_LABEL_WIDTH + COLS * (CELL_SIZE + CELL_GAP)
Hauteur  = COL_HEADER_HEIGHT + ROWS * (CELL_SIZE + CELL_GAP)
```

### Algorithme de rendu

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState, history: &mut History) {
    let cell = Theme::CELL_SIZE;
    let gap = Theme::CELL_GAP;
    let step = cell + gap;
    let grid_width = Theme::NOTE_LABEL_WIDTH + COLS as f32 * step;
    let grid_height = Theme::COL_HEADER_HEIGHT + ROWS as f32 * step;

    // Envelopper dans un Frame (panel)
    widgets::panel::synth_panel_no_title(ui, |ui| {
        let (total_rect, _) = ui.allocate_exact_size(
            egui::vec2(grid_width, grid_height),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(total_rect);
        let origin = total_rect.min;

        // ── Numéros de colonnes ──
        for col in 0..COLS {
            let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step + cell / 2.0;
            let y = origin.y + Theme::COL_HEADER_HEIGHT / 2.0;
            let is_active = state.is_playing && state.play_col == col as i32;
            painter.text(
                egui::pos2(x, y),
                egui::Align2::CENTER_CENTER,
                format!("{}", col + 1),
                egui::FontId::proportional(8.0),
                if is_active { Theme::ACCENT_LIGHT } else { Theme::TEXT_DIM },
            );
        }

        // ── Colonne playhead (fond) ──
        if state.is_playing && state.play_col >= 0 {
            let col = state.play_col as usize;
            let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step - 1.0;
            let y = origin.y + Theme::COL_HEADER_HEIGHT;
            let h = ROWS as f32 * step;
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell + 2.0, h)),
                4.0,
                Theme::PLAYHEAD_BG,
            );
        }

        // ── Labels des notes ──
        for row in 0..ROWS {
            let label = NOTE_LABELS[row];
            let y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step + cell / 2.0;
            let x = origin.x + Theme::NOTE_LABEL_WIDTH - 4.0;
            let color = if label.contains('#') { Theme::ACCENT } else { Theme::TEXT };
            painter.text(
                egui::pos2(x, y),
                egui::Align2::RIGHT_CENTER,
                label,
                egui::FontId::proportional(9.0),
                color,
            );
        }

        // ── Cellules ──
        // Sauvegarder la grille avant modifications pour l'historique
        let grid_before = state.grid;
        let mut grid_changed = false;

        for row in 0..ROWS {
            for col in 0..COLS {
                let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step;
                let y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step;
                let cell_rect = egui::Rect::from_min_size(
                    egui::pos2(x, y),
                    egui::vec2(cell, cell),
                );

                let cell_id = ui.id().with(("cell", row, col));
                let response = ui.interact(cell_rect, cell_id, egui::Sense::click_and_drag());

                // Interaction
                let should_apply = response.clicked()
                    || (response.dragged() && response.hovered());

                if should_apply {
                    match state.draw_mode {
                        DrawMode::Toggle => {
                            if response.clicked() {
                                state.grid[row][col] = !state.grid[row][col];
                                grid_changed = true;
                            }
                        }
                        DrawMode::Draw => {
                            if !state.grid[row][col] {
                                state.grid[row][col] = true;
                                grid_changed = true;
                            }
                        }
                        DrawMode::Erase => {
                            if state.grid[row][col] {
                                state.grid[row][col] = false;
                                grid_changed = true;
                            }
                        }
                    }
                }

                // Modifier clicks (Shift = toggle row, Ctrl = toggle col)
                if response.clicked() {
                    let modifiers = ui.input(|i| i.modifiers);
                    if modifiers.shift {
                        state.toggle_row(row);
                        grid_changed = true;
                    } else if modifiers.ctrl || modifiers.mac_cmd {
                        state.toggle_col(col);
                        grid_changed = true;
                    }
                }

                // Déterminer le style
                let is_on = state.grid[row][col];
                let is_playhead = state.is_playing && state.play_col == col as i32;
                let is_hit = is_on && is_playhead;
                let is_hovered = response.hovered();

                let (bg, border_color) = if is_hit {
                    (Theme::CELL_HIT, Theme::ACCENT_LIGHT)
                } else if is_on && is_hovered {
                    (Theme::CELL_ON_HOVER, Theme::ACCENT)
                } else if is_on {
                    (Theme::CELL_ON, Theme::ACCENT)
                } else if is_playhead {
                    (Theme::CELL_PLAYHEAD, Theme::CELL_PLAYHEAD_BORDER)
                } else if is_hovered {
                    (Theme::CELL_OFF_HOVER, Theme::BORDER)
                } else {
                    (Theme::CELL_OFF, Theme::BORDER)
                };

                // Glow pour les HIT (rect élargi, semi-transparent)
                if is_hit {
                    painter.rect_filled(
                        cell_rect.expand(3.0),
                        Theme::CELL_ROUNDING + 3.0,
                        Theme::CELL_HIT_GLOW,
                    );
                }

                // Cellule
                painter.rect_filled(cell_rect, Theme::CELL_ROUNDING, bg);
                painter.rect_stroke(cell_rect, Theme::CELL_ROUNDING, Stroke::new(1.0, border_color));

                // Shadow pour cellules ON
                if is_on && !is_hit {
                    painter.rect_filled(
                        cell_rect.expand(1.5),
                        Theme::CELL_ROUNDING + 1.5,
                        Color32::from_rgba_premultiplied(124, 58, 237, 20),
                    );
                    // Re-dessiner par dessus pour que la shadow soit derrière
                    painter.rect_filled(cell_rect, Theme::CELL_ROUNDING, bg);
                    painter.rect_stroke(cell_rect, Theme::CELL_ROUNDING, Stroke::new(1.0, border_color));
                }
            }
        }

        // Sauvegarder dans l'historique si la grille a changé
        if grid_changed {
            history.push(grid_before);
        }
    });
}
```

**Note sur le rendu des shadows** : pour que la glow soit derrière la cellule, dessiner d'abord le rect glow (expand), puis la cellule par dessus. Alternativement, dessiner les glows dans une première passe, puis toutes les cellules dans une seconde passe.

**Optimisation recommandée** : faire 2 passes :
1. Passe 1 : dessiner toutes les glow/shadows (cellules ON + HIT)
2. Passe 2 : dessiner toutes les cellules (fond + bordure)

---

## 12. Barre de transport

**Fichier : `transport.rs`**

### Layout horizontal

```
[● Play/Pause 40px] [CLEAR] | [BPM ═══════ 120] | [◎ Swing]
```

### Bouton Play/Pause

- Cercle 40×40px
- **Stopped** : fond `KNOB_BG`, bordure `BORDER` 2px, icône triangle blanc (play)
- **Playing** : fond gradient `ACCENT_DARK`→`ACCENT` (si pas de gradient, `ACCENT_DARK`), bordure `ACCENT_LIGHT` 2px, icône pause (2 barres blanches 4×14px gap 3px), glow

Icône play (triangle) : dessiner un `PathShape` avec 3 points.
Icône pause : 2 `rect_filled`.

### Bouton Clear

- `egui::Button` avec texte "CLEAR", fond `KNOB_BG`, texte `TEXT_DIM`, font 10px bold, rounding 6px, letter-spacing 1.

Action : `state.clear_grid()` + `history.push(grid_before)`.

### Slider BPM

- Label "BPM" en `TEXT` 10px bold
- `egui::Slider::new(&mut state.bpm, 40.0..=240.0)` avec style custom, ou un `hslider` custom.
- Valeur affichée en `ACCENT_LIGHT` 12px bold, largeur 30px.

### Knob Swing

- `knob("Swing", &mut state.swing, 0.0, 100.0, 0.0, "%", 36.0)`

### Séparateurs

- Trait vertical 1px hauteur 28px couleur `BORDER` entre les groupes.

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState, history: &mut History) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;

        // Play/Pause button
        let btn_rect = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::click()).0;
        // ... dessiner cercle + icône
        // Sur clic : state.toggle_play();

        // Clear button
        if ui.add(egui::Button::new(
            egui::RichText::new("CLEAR").size(10.0).strong().color(Theme::TEXT_DIM)
        ).fill(Theme::KNOB_BG).stroke(Theme::panel_stroke()).rounding(6.0)).clicked() {
            let before = state.grid;
            state.clear_grid();
            history.push(before);
        }

        // Séparateur
        let sep_rect = ui.allocate_exact_size(egui::vec2(1.0, 28.0), egui::Sense::hover()).0;
        ui.painter().rect_filled(sep_rect, 0.0, Theme::BORDER);

        // BPM
        ui.label(egui::RichText::new("BPM").size(10.0).strong().color(Theme::TEXT));
        ui.add(egui::Slider::new(&mut state.bpm, 40.0..=240.0).show_value(false));
        ui.label(egui::RichText::new(format!("{}", state.bpm as i32)).size(12.0).strong().color(Theme::ACCENT_LIGHT));

        // Séparateur
        // ...

        // Swing knob
        widgets::knob::knob(ui, "Swing", &mut state.swing, 0.0, 100.0, 0.0, "%", 36.0);
    });
}
```

---

## 13. Barre de densité

**Fichier : `density_bar.rs`**

16 petites barres horizontales sous la grille, alignées avec les colonnes.

```rust
pub fn draw(ui: &mut egui::Ui, state: &TenoriState) {
    ui.horizontal(|ui| {
        ui.add_space(Theme::NOTE_LABEL_WIDTH); // aligner avec la grille

        let step = Theme::CELL_SIZE + Theme::CELL_GAP;

        for col in 0..COLS {
            let density = state.col_density(col);
            let is_active = state.is_playing && state.play_col == col as i32;

            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(Theme::CELL_SIZE, Theme::DENSITY_HEIGHT),
                egui::Sense::hover(),
            );

            let alpha = if density > 0 {
                ((density as f32 / 6.0).min(1.0) * 255.0) as u8
            } else { 0 };

            let fill = if density > 0 {
                Color32::from_rgba_premultiplied(
                    124, 58, 237, alpha
                )
            } else {
                Theme::CELL_OFF
            };

            let border = if is_active { Theme::ACCENT_LIGHT } else { Theme::BORDER };

            let painter = ui.painter();
            painter.rect_filled(rect, 3.0, fill);
            painter.rect_stroke(rect, 3.0, Stroke::new(1.0, border));

            // Gap
            ui.add_space(Theme::CELL_GAP);
        }
    });
}
```

---

## 14. Header

**Fichier : `header.rs`**

```rust
pub fn draw(ui: &mut egui::Ui, state: &TenoriState) {
    ui.horizontal(|ui| {
        // LED
        let (led_rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        let led_color = if state.is_playing { Theme::LED_PLAYING } else { Theme::LED_STOPPED };
        let glow_color = if state.is_playing { Theme::LED_PLAYING_GLOW } else { Theme::ACCENT_LIGHT };

        ui.painter().circle_filled(led_rect.center(), 5.0, led_color);
        // Glow autour
        ui.painter().circle_filled(led_rect.center(), 8.0,
            Color32::from_rgba_premultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 40));

        ui.add_space(8.0);

        // Titre
        ui.label(
            egui::RichText::new("TENORI-SYNTH")
                .size(20.0)
                .strong()
                .color(Theme::ACCENT_LIGHT)
                // letter-spacing : pas natif dans egui, on accepte sans
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Version
            ui.label(egui::RichText::new("v2.0").size(9.0).color(Theme::TEXT_DIM));
            ui.add_space(12.0);

            // Notes actives (badges)
            let notes = state.active_note_names();
            for note in notes.iter().rev() {
                egui::Frame::none()
                    .fill(Theme::ACCENT_DARK)
                    .rounding(3.0)
                    .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(*note)
                                .size(10.0)
                                .strong()
                                .color(Theme::TEXT_WHITE)
                        );
                    });
            }

            if !notes.is_empty() {
                ui.label(egui::RichText::new("PLAYING:").size(9.0).color(Theme::TEXT_DIM));
            }
        });
    });
}
```

---

## 15. Panneaux latéraux

Chaque panel utilise `widgets::panel::synth_panel(ui, title, |ui| { ... })`.

### `panels/oscillator.rs`

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Oscillator", |ui| {
        let options: Vec<(Waveform, &str)> = Waveform::ALL.iter().map(|w| (*w, w.label())).collect();
        select_buttons(ui, &mut state.osc_waveform, &options);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            knob(ui, "Pitch", &mut state.osc_pitch, 0.0, 100.0, 50.0, "", 40.0);
            knob(ui, "Detune", &mut state.osc_detune, 0.0, 100.0, 0.0, "ct", 40.0);
        });
    });
}
```

### `panels/envelope.rs`

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Envelope", |ui| {
        ui.horizontal(|ui| {
            knob(ui, "A", &mut state.env_attack,  0.0, 100.0, 10.0, "", 36.0);
            knob(ui, "D", &mut state.env_decay,   0.0, 100.0, 30.0, "", 36.0);
            knob(ui, "S", &mut state.env_sustain,  0.0, 100.0, 70.0, "", 36.0);
            knob(ui, "R", &mut state.env_release,  0.0, 100.0, 40.0, "", 36.0);
        });
    });
}
```

### `panels/filter.rs`

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Filter", |ui| {
        let options: Vec<(FilterType, &str)> = FilterType::ALL.iter().map(|f| (*f, f.label())).collect();
        select_buttons(ui, &mut state.filter_type, &options);
        ui.add_space(8.0);
        hslider(ui, "Cutoff", &mut state.filter_cutoff, 0.0, 100.0);
        hslider(ui, "Reso", &mut state.filter_reso, 0.0, 100.0);
    });
}
```

### `panels/lfo.rs`

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "LFO", |ui| {
        let options: Vec<(LfoDest, &str)> = LfoDest::ALL.iter().map(|d| (*d, d.label())).collect();
        select_buttons(ui, &mut state.lfo_dest, &options);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            knob(ui, "Rate", &mut state.lfo_rate, 0.0, 100.0, 30.0, "Hz", 36.0);
            knob(ui, "Depth", &mut state.lfo_depth, 0.0, 100.0, 50.0, "", 36.0);
        });
    });
}
```

### `panels/effects.rs`

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Effects", |ui| {
        hslider(ui, "Reverb", &mut state.fx_reverb, 0.0, 100.0);
        hslider(ui, "Delay", &mut state.fx_delay, 0.0, 100.0);
        hslider(ui, "Chorus", &mut state.fx_chorus, 0.0, 100.0);
    });
}
```

### `panels/scale.rs`

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Scale", |ui| {
        let options: Vec<(Scale, &str)> = Scale::ALL.iter().map(|s| (*s, s.label())).collect();
        select_buttons(ui, &mut state.scale, &options);
    });
}
```

### `panels/draw_mode.rs`

```rust
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Draw Mode", |ui| {
        let options: Vec<(DrawMode, &str)> = DrawMode::ALL.iter().map(|d| (*d, d.label())).collect();
        select_buttons(ui, &mut state.draw_mode, &options);
    });
}
```

---

## 16. Raccourcis clavier

**Fichier : `shortcuts.rs`**

```rust
use egui::Key;
use super::state::*;
use super::history::History;

pub fn handle(ui: &mut egui::Ui, state: &mut TenoriState, history: &mut History) {
    let modifiers = ui.input(|i| i.modifiers);

    // ── Transport ──

    if ui.input(|i| i.key_pressed(Key::Space)) {
        state.toggle_play();
    }

    if !modifiers.ctrl && !modifiers.mac_cmd {
        if ui.input(|i| i.key_pressed(Key::C)) {
            let before = state.grid;
            state.clear_grid();
            history.push(before);
        }
    }

    // ── BPM ──

    if modifiers.ctrl || modifiers.mac_cmd {
        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
            state.bpm = (state.bpm + 5.0).min(240.0);
        }
        if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
            state.bpm = (state.bpm - 5.0).max(40.0);
        }
    }

    // ── Draw modes ──

    if !modifiers.ctrl && !modifiers.mac_cmd {
        if ui.input(|i| i.key_pressed(Key::D)) {
            state.draw_mode = DrawMode::Draw;
        }
        if ui.input(|i| i.key_pressed(Key::E)) {
            state.draw_mode = DrawMode::Erase;
        }
        if ui.input(|i| i.key_pressed(Key::T)) {
            state.draw_mode = DrawMode::Toggle;
        }
    }

    // ── Waveform ──

    if modifiers.ctrl || modifiers.mac_cmd {
        if ui.input(|i| i.key_pressed(Key::Num1)) {
            state.osc_waveform = Waveform::Sine;
        }
        if ui.input(|i| i.key_pressed(Key::Num2)) {
            state.osc_waveform = Waveform::Saw;
        }
        if ui.input(|i| i.key_pressed(Key::Num3)) {
            state.osc_waveform = Waveform::Square;
        }
        if ui.input(|i| i.key_pressed(Key::Num4)) {
            state.osc_waveform = Waveform::Tri;
        }
    }

    // ── Filter / LFO cycle ──

    if ui.input(|i| i.key_pressed(Key::Tab)) {
        if modifiers.shift {
            state.lfo_dest = state.lfo_dest.next();
        } else {
            state.filter_type = state.filter_type.next();
        }
    }

    // ── Undo / Redo ──

    if (modifiers.ctrl || modifiers.mac_cmd) && ui.input(|i| i.key_pressed(Key::Z)) {
        if modifiers.shift {
            // Redo
            if let Some(grid) = history.redo() {
                state.grid = grid;
            }
        } else {
            // Undo
            if let Some(grid) = history.undo(&state.grid) {
                state.grid = grid;
            }
        }
    }

    if (modifiers.ctrl || modifiers.mac_cmd) && ui.input(|i| i.key_pressed(Key::Y)) {
        if let Some(grid) = history.redo() {
            state.grid = grid;
        }
    }
}
```

### Tableau récapitulatif

| Raccourci         | Action                                   |
|-------------------|------------------------------------------|
| `Space`           | Toggle Play / Pause                      |
| `C`               | Clear la grille                          |
| `D`               | Mode Draw                                |
| `E`               | Mode Erase                               |
| `T`               | Mode Toggle                              |
| `Ctrl/Cmd + ↑`   | BPM + 5                                  |
| `Ctrl/Cmd + ↓`   | BPM - 5                                  |
| `Ctrl/Cmd + 1`   | Waveform → Sine                          |
| `Ctrl/Cmd + 2`   | Waveform → Saw                           |
| `Ctrl/Cmd + 3`   | Waveform → Square                        |
| `Ctrl/Cmd + 4`   | Waveform → Tri                           |
| `Tab`             | Cycle filter (LP → HP → BP)             |
| `Shift + Tab`     | Cycle LFO dest (Pitch → Filter → Amp)   |
| `Ctrl/Cmd + Z`   | Undo                                     |
| `Ctrl/Cmd + Shift + Z` ou `Ctrl/Cmd + Y` | Redo         |
| `Shift + clic`    | Toggle toute la ligne                    |
| `Ctrl/Cmd + clic` | Toggle toute la colonne                 |

---

## 17. Undo / Redo

**Fichier : `history.rs`**

Historique circulaire de 20 snapshots de la grille.

```rust
use super::state::{ROWS, COLS};

type Grid = [[bool; COLS]; ROWS];

const MAX_HISTORY: usize = 20;

pub struct History {
    undo_stack: Vec<Grid>,
    redo_stack: Vec<Grid>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::with_capacity(MAX_HISTORY),
            redo_stack: Vec::with_capacity(MAX_HISTORY),
        }
    }

    /// Sauvegarder l'état AVANT modification
    pub fn push(&mut self, grid_before: Grid) {
        if self.undo_stack.len() >= MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(grid_before);
        self.redo_stack.clear(); // nouveau branch, on efface le redo
    }

    /// Undo : restaure l'état précédent, retourne la grille à appliquer
    pub fn undo(&mut self, current_grid: &Grid) -> Option<Grid> {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(*current_grid);
            Some(previous)
        } else {
            None
        }
    }

    /// Redo : restaure l'état suivant
    pub fn redo(&mut self) -> Option<Grid> {
        self.redo_stack.pop()
    }

    pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }
}
```

---

## 18. API publique

Le module expose :

```rust
// src/tenori_synth/mod.rs

/// Interface publique
pub struct TenoriSynth { ... }

impl TenoriSynth {
    /// Créer une nouvelle instance
    pub fn new() -> Self;

    /// Afficher dans un egui::Ui (appeler chaque frame)
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);

    /// État en lecture seule (pour connecter un moteur audio)
    pub fn state(&self) -> &TenoriState;

    /// État mutable (pour charger un preset, etc.)
    pub fn state_mut(&mut self) -> &mut TenoriState;

    /// Notes actives au step courant (noms, ex: ["C5", "A4"])
    pub fn active_note_names(&self) -> Vec<&'static str>;

    /// Indices des lignes actives au step courant
    pub fn active_rows(&self) -> Vec<usize>;

    /// Est-ce que le séquenceur joue ?
    pub fn is_playing(&self) -> bool;

    /// Colonne courante (-1 si arrêté, 0..15 sinon)
    pub fn current_step(&self) -> i32;
}
```

### Utilisation dans l'app existante

```rust
// src/main.rs ou src/app.rs

mod tenori_synth;
use tenori_synth::TenoriSynth;

struct MyApp {
    tenori: TenoriSynth,
    // ... autres champs existants
}

impl MyApp {
    fn new() -> Self {
        Self {
            tenori: TenoriSynth::new(),
            // ...
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.tenori.show(ctx, ui);
        });
    }
}
```

---

## 19. Checklist finale

Avant de considérer l'implémentation terminée, vérifier que :

- [ ] `cargo build` compile sans erreur ni warning
- [ ] Tous les fichiers listés en section 3 existent
- [ ] La grille 16×16 s'affiche avec les bons labels de notes
- [ ] Cliquer sur une cellule la toggle (mode Toggle)
- [ ] Le drag fonctionne en mode Draw et Erase
- [ ] `Shift + clic` toggle une ligne entière
- [ ] `Ctrl/Cmd + clic` toggle une colonne entière
- [ ] Le bouton Play lance le playhead qui défile de gauche à droite
- [ ] Les cellules HIT (ON + playhead) ont un glow blanc-violet
- [ ] La colonne playhead a un fond semi-transparent
- [ ] Le BPM change la vitesse du playhead
- [ ] Le bouton Clear vide la grille
- [ ] Tous les knobs répondent au drag vertical
- [ ] Tous les hsliders répondent au click et drag
- [ ] Les select buttons changent la valeur de l'enum
- [ ] Les raccourcis clavier fonctionnent (Space, C, D, E, T, Tab, etc.)
- [ ] Undo (Ctrl+Z) et Redo (Ctrl+Shift+Z / Ctrl+Y) fonctionnent
- [ ] La barre de densité reflète le nombre de notes par colonne
- [ ] Le header affiche les notes actives en temps réel
- [ ] La LED est verte en lecture, violette à l'arrêt
- [ ] Le panneau latéral scroll si la fenêtre est trop petite
- [ ] Les couleurs correspondent au thème défini
- [ ] Aucun `unsafe`, aucune dépendance ajoutée
- [ ] Le code existant n'est pas cassé
