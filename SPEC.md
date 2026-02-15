# Synthesis — Spécifications techniques

## Vue d'ensemble

**Synthesis** est un synthétiseur audio temps réel écrit en Rust, fonctionnant sous macOS,
avec une interface graphique permettant de modéliser et façonner des sons.

## Stack technique

| Couche | Crate | Version | Rôle |
|---|---|---|---|
| Audio I/O | `cpal` | 0.17 | Sortie audio temps réel via CoreAudio |
| DSP / Synthèse | `fundsp` | 0.23 | Oscillateurs, filtres, enveloppes, effets |
| MIDI | `midir` | 0.10 | Entrée MIDI (CoreMIDI natif sur macOS) |
| GUI | `egui` + `eframe` | 0.33 | Interface immédiate, rendu 60fps+, widgets custom |
| Communication inter-threads | `rtrb` | 0.3 | Ring buffer lock-free pour événements |
| Paramètres partagés | `fundsp::shared()` | — | Floats atomiques pour contrôle temps réel |

## Architecture

```
┌─────────────────┐     AtomicF32       ┌─────────────────┐
│   GUI Thread    │ ──────────────────> │  Audio Thread   │
│   (egui)        │   params: cutoff,   │  (cpal callback)│
│                 │   volume, etc.      │                 │
│   knobs/sliders │ <────────────────── │  fundsp graph   │
│   oscilloscope  │   rtrb: waveform    │                 │
└─────────────────┘   data, meters      └─────────────────┘
                                               ▲
┌─────────────────┐     rtrb: MIDI             │
│  MIDI Thread    │ ───────────────────────────┘
│  (midir callback)│
└─────────────────┘
```

### Contraintes temps réel

Le thread audio ne doit **jamais** :
- Allouer ou désallouer de la mémoire
- Locker un mutex
- Effectuer des I/O (fichier, réseau, console)
- Appeler des fonctions bloquantes

Toute communication entre threads passe par des atomiques (`AtomicF32`)
ou des ring buffers lock-free (`rtrb`).

### Organisation des modules

```
src/
├── main.rs              # Point d'entrée CLI (clap), --gui flag
├── midi.rs              # Entrée MIDI (midir), NoteEvent, TestOn/TestOff
├── preset.rs            # Presets : save/load JSON, presets d'usine
├── engine/              # Moteur audio
│   ├── mod.rs           # Sortie audio (cpal), init/start helpers
│   ├── oscillator.rs    # Oscillateurs (fundsp), ADSR, filtres, LFO, effets, poly graph
│   ├── filter.rs        # FilterConfig, LfoConfig, Mul2, Add2, resonance_to_q
│   ├── effects.rs       # FeedbackDelay, EffectsConfig, wire_delay/reverb/chorus
│   └── voice.rs         # Voice, VoiceAllocator, VoiceConfig, VoiceShared
├── gui/                 # Interface graphique
│   ├── mod.rs           # Point d'entrée GUI (eframe), fenêtre 1400×850
│   ├── app.rs           # SynthApp : sync per-voice, bridge moteur audio
│   └── oscilloscope.rs  # Visualiseur de forme d'onde (egui_plot)
└── synth_ui/            # Composants UI Synthwave
    ├── mod.rs           # SynthUI : layout 8 strips, raccourcis clavier, SynthParams
    ├── theme.rs         # SynthTheme : palette Synthwave, frames panels
    ├── widgets/
    │   ├── knob.rs      # Knob rotatif (58×78) + mini knob (36×52) pour strips
    │   ├── vslider.rs   # Slider vertical (volume master)
    │   ├── hslider.rs   # Slider horizontal (filtres, effets, level)
    │   ├── level_meter.rs # VU-mètre (15 segments)
    │   ├── select_buttons.rs # Groupe de boutons toggle
    │   ├── keyboard.rs  # Clavier piano (2 octaves, thème)
    │   └── pads.rs      # Drum pads (grille 4×4)
    └── panels/
        ├── voice_strip.rs   # Channel strip per-voice (waveform, ADSR, filtre, LFO, level, test)
        ├── effects.rs       # Contrôles delay/reverb/chorus
        ├── master.rs        # Volume + VU-mètres + oscilloscope
        ├── keyboard_panel.rs # Wrapper clavier piano
        └── pads_panel.rs    # Wrapper drum pads
```

## Phases de développement

### Phase 1 — Moteur sonore minimal

**Objectif** : produire un son continu et contrôlable sans GUI.

- 1 oscillateur avec 4 formes d'onde : sine, saw, square, triangle
- Sélection de la forme d'onde au lancement (argument CLI)
- Contrôle de la fréquence (défaut : 440 Hz)
- Contrôle de l'amplitude (défaut : 0.5)
- Sortie audio via `cpal` (sample rate 44100 Hz, buffer 256 samples)
- DSP via `fundsp`
- Durée de lecture : 5 secondes puis arrêt propre

**Critères d'acceptation** :
- `cargo run --release -- --waveform sine` produit un La 440 Hz pendant 5s
- Les 4 formes d'onde fonctionnent sans glitch
- Le programme se termine proprement (pas de thread zombie)

### Phase 2 — IHM de base

**Objectif** : interface graphique minimale pour contrôler le son en temps réel.

- Fenêtre `egui`/`eframe` avec :
  - Sélecteur de forme d'onde (sine, saw, square, triangle)
  - Slider de fréquence (20 Hz — 20 kHz, échelle logarithmique)
  - Slider de volume (0.0 — 1.0)
  - Bouton play/stop
- Visualisation de la forme d'onde (oscilloscope)
- Communication GUI ↔ Audio via `fundsp::shared()`

**Critères d'acceptation** :
- Les changements de paramètres se répercutent en temps réel sans glitch
- L'oscilloscope affiche la forme d'onde courante
- Latence perceptible < 10 ms

### Phase 3 — Enveloppe ADSR + polyphonie

**Objectif** : rendre le synthétiseur jouable musicalement.

- Enveloppe ADSR (Attack, Decay, Sustain, Release) par voix
- Polyphonie : 8 voix simultanées (allocation round-robin ou voice stealing)
- Entrée MIDI via `midir` (note on/off, velocity)
- Clavier virtuel cliquable dans la GUI
- Affichage de l'enveloppe ADSR avec sliders A/D/S/R

**Critères d'acceptation** :
- Un clavier MIDI externe déclenche des notes polyphoniques
- Le clavier virtuel fonctionne à la souris
- L'enveloppe ADSR module l'amplitude de chaque voix indépendamment

### Phase 4 — Filtres et modulation

**Objectif** : enrichir le son avec filtrage et modulation.

- Filtre résonant : lowpass, highpass, bandpass (sélectionnable)
- Paramètres : cutoff (20 Hz — 20 kHz, log), résonance (0.0 — 1.0)
- LFO (Low Frequency Oscillator) : sine, triangle, saw
- Routage de modulation : LFO → fréquence, LFO → cutoff, LFO → amplitude
- Profondeur de modulation configurable

**Critères d'acceptation** :
- Le filtre coupe les fréquences en temps réel sans artefacts
- Le LFO module le paramètre cible de manière audible et fluide
- Pas de glitch audio lors du changement de type de filtre

### Phase 5 — Effets et presets

**Objectif** : finaliser avec des effets et la persistance.

- Effets : delay (temps + feedback), reverb, chorus
- Chaîne d'effets configurable (ordre, bypass par effet)
- Système de presets :
  - Sérialisation/désérialisation via `serde` (format RON ou JSON)
  - Sauvegarde/chargement depuis le système de fichiers
  - Presets d'usine inclus dans le binaire
- Menu de presets dans la GUI

**Critères d'acceptation** :
- Les effets s'appliquent en temps réel sans latence perceptible
- Un preset sauvegardé se recharge à l'identique
- Les presets d'usine couvrent des sons variés (pad, lead, bass, fx)

### Phase 6 — Refonte IHM « Synthwave »

**Objectif** : remplacer l'interface actuelle (sliders egui standards) par une IHM professionnelle
avec widgets custom, thème sombre violet, et interactions avancées.

> Spécification détaillée : [`synth-egui-spec.md`](synth-egui-spec.md)

#### Thème & identité visuelle

- Palette « Synthwave » sombre : fond `#0d0d1a`, panels `#1a1a2e`, accent violet `#9b59b6` / `#c084fc`
- Shadows et rounding sur tous les panels
- Header avec LED indicateur, titre « SYNTHWAVE », version

#### Widgets custom (tous rendus via `egui::Painter`)

| Widget | Description | Fichier |
|---|---|---|
| Knob rotatif | Arc 270°, drag vertical, indicateur + label | `widgets/knob.rs` |
| Slider vertical | Track 8×90px, thumb 16×12px, gradient accent | `widgets/vslider.rs` |
| Slider horizontal | Label + track + thumb + valeur, pour filtres/effets | `widgets/hslider.rs` |
| VU-mètre | 15 segments, vert→jaune→rouge | `widgets/level_meter.rs` |
| Clavier piano | 2 octaves (C3–B4), 14 blanches + 10 noires, highlight au survol/clic | `widgets/keyboard.rs` |
| Drum pads | Grille 4×4, 16 pads labellisés (Kick, Snare, HiHat…), animation pressé | `widgets/pads.rs` |
| Boutons sélection | Toggle group horizontal pour Waveform, FilterType, LfoDest | `widgets/select_buttons.rs` |

#### Layout principal (v0.6.0, remplacé en v0.7.0)

> **Note** : Ce layout à 6 panels a été remplacé par l'architecture per-voice channel strips en v0.7.0. Voir Phase 7 ci-dessous.

#### Layout actuel (v0.7.0 — Channel Strips)

```
┌────────────────────────────────────────────────────────────────────────────┐
│ ● SYNTHWAVE   Preset [▾ Init]  Save [____] [Save]  MIDI [▾ port] v0.7.0  │
├────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┬───┤
│ VOX 1  │ VOX 2  │ VOX 3  │ VOX 4  │ VOX 5  │ VOX 6  │ VOX 7  │ VOX 8  │ M │
│ S Sw   │        │        │        │        │        │        │        │ A │
│ Sq T   │        │        │        │        │        │        │        │ S │
│ A D S R│        │        │        │        │        │        │        │ T │
│ F LP HP│        │        │        │        │        │        │        │ E │
│ Cut Res│        │        │        │        │        │        │        │ R │
│ L LFO..│        │        │        │        │        │        │        │   │
│ Level  │        │        │        │        │        │        │        │   │
│ [TEST] │        │        │        │        │        │        │        │   │
├────────┴────────┴────────┴────────┴────────┴────────┴────┬───────────┴───┤
│              Keyboard (2 octaves)                         │  Drum Pads    │
└──────────────────────────────────────────────────────────┴───────────────┘
```

- **Rangée haute** : 8 channel strips per-voice + master strip (volume, VU, oscilloscope, effets)
- **Rangée basse** : clavier piano (~70%) + drum pads (~30%)
- Chaque strip contient : waveform, ADSR (4 mini knobs), filtre (type + cutoff + résonance), LFO (target + rate + depth), level, bouton TEST
- Fenêtre : 1400×850 pixels

#### Raccourcis clavier

- **Clavier piano** : mapping AZERTY sur 2 rangées (W/X/C/V/B/N = C3–B3, A/Z/E/R/T/Y/U = C4–B4), touches noires sur S/D/G/H/J et 2/3/5/6/7
- **Drum pads** : pavé numérique 1–9, 0, F1–F6
- **Globaux** : Ctrl+↑/↓ volume, Ctrl+1–4 waveform, Tab cycle filtre, Shift+Tab cycle LFO dest, Space panic (all notes off)
- Support multi-touches (accords) via `HashSet<String>` pour les notes pressées

#### Architecture fichiers

```
src/synth_ui/
├── mod.rs              # SynthUI struct + show(), intégration
├── state.rs            # SynthState (tous les paramètres)
├── theme.rs            # SynthTheme (couleurs, shadows, constantes)
├── widgets/
│   ├── mod.rs
│   ├── knob.rs         # Knob rotatif custom
│   ├── vslider.rs      # Slider vertical
│   ├── hslider.rs      # Slider horizontal
│   ├── level_meter.rs  # VU-mètre segmenté
│   ├── keyboard.rs     # Clavier piano 2 octaves
│   ├── pads.rs         # Grille drum pads 4×4
│   └── select_buttons.rs
└── panels/
    ├── mod.rs
    ├── oscillator.rs
    ├── envelope.rs
    ├── filter.rs
    ├── lfo.rs
    ├── effects.rs
    ├── master.rs
    ├── keyboard_panel.rs
    └── pads_panel.rs
```

#### Intégration

- Module `synth_ui` ajouté proprement à côté de `gui/` existant
- `SynthUI::show(&mut self, ui: &mut egui::Ui)` appelable depuis la boucle de rendu
- `SynthUI::state()` expose l'état en lecture pour connexion au moteur audio
- Ne casse pas le mode GUI existant — transition progressive possible

**Critères d'acceptation** :
- Tous les widgets custom se rendent correctement et sont interactifs
- Le thème violet est cohérent sur l'ensemble de l'interface
- Les raccourcis clavier fonctionnent (piano, pads, globaux)
- Les accords (multi-touches) sont supportés
- Le clavier physique et le clavier virtuel sont synchronisés visuellement
- Pas de régression sur les fonctionnalités audio existantes
- `cargo clippy` zero warning, tous les tests passent

### Phase 7 — Per-Voice Channel Strips

**Objectif** : architecture per-voice avec 8 channel strips indépendants.

Chaque voix possède ses propres paramètres :
- Forme d'onde (sine, saw, square, triangle)
- Enveloppe ADSR (attack, decay, sustain, release)
- Filtre résonant (LP/HP/BP, cutoff, résonance, enable/disable)
- LFO (target freq/cutoff/amp, rate, depth, enable/disable)
- Niveau (volume per-voice)

#### Structures de données

- `VoiceConfig` : paramètres par voix (topologie + runtime), sérialisable
- `VoiceShared` : atomiques `Shared` par voix pour sync audio thread
- `VoiceConfig::topology_differs()` : détecte les changements nécessitant un rebuild du graphe

#### Moteur audio

- `build_voice_unit()` accepte les paramètres per-voice (+ `voice_level`)
- `build_poly_graph()` itère avec `&[VoiceConfig]` et `&[VoiceShared]`
- Effets (delay, reverb, chorus) restent globaux post-mix

#### UI Channel Strips

- 8 strips verticaux compacts côte à côte (style mixer)
- Mini knobs (36×52px) pour ADSR dans les strips
- Boutons toggle compacts "F" (filtre) et "L" (LFO) pour enable/disable
- LED d'activité par voix (vert = active, jaune = releasing, gris = idle)
- Bouton TEST par voix : joue C4 sur la voix spécifique (~500ms)

#### Événements Test

- `NoteEvent::TestOn { voice_idx, note, velocity }` : force l'allocation sur une voix
- `NoteEvent::TestOff { voice_idx }` : release de la voix testée
- `VoiceAllocator::force_note_on()` / `force_note_off()` pour cibler une voix spécifique

#### Presets

- Chargement : applique le preset à toutes les 8 voix (rétro-compatible)
- Sauvegarde : prend la configuration de la voix 1

**Critères d'acceptation** :
- Chaque voix a ses propres réglages indépendants
- Changer le waveform d'une voix ne change pas les autres
- Le bouton TEST joue un son C4 sur la voix correspondante
- Les presets chargent/sauvegardent correctement
- `cargo clippy --release` zero warning, tous les tests passent

## Configuration audio

| Paramètre | Valeur |
|---|---|
| Sample rate | 44 100 Hz |
| Buffer size | 256 samples (~5.8 ms) |
| Format | f32 |
| Canaux | Stéréo (2) |

## Bonnes pratiques

| Aspect | Outil / Méthode |
|---|---|
| Build audio | Toujours `--release` (debug = glitches DSP) |
| Tests | Tests unitaires par module, tests d'intégration pour le pipeline |
| Linting | `cargo clippy` |
| Formatage | `cargo fmt` |
| Erreurs | `thiserror` pour les erreurs typées |
| Versionnement | Semantic Versioning, Conventional Commits |
| Changelog | Keep a Changelog, mis à jour à chaque version |
