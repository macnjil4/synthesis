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
├── main.rs          # Point d'entrée, initialisation
├── engine/          # Moteur audio
│   ├── mod.rs
│   ├── oscillator.rs
│   ├── envelope.rs
│   ├── filter.rs
│   ├── voice.rs
│   └── effects.rs
├── gui/             # Interface graphique
│   ├── mod.rs
│   ├── app.rs
│   ├── widgets/
│   └── visualizer.rs
└── midi/            # Gestion MIDI
    ├── mod.rs
    └── handler.rs
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
