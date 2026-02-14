use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::effects::{EffectSlot, EffectsConfig};
use crate::engine::filter::{FilterConfig, FilterType, LfoConfig, LfoTarget, LfoWaveform};
use crate::engine::oscillator::{AdsrParams, Waveform};

#[derive(Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub waveform: Waveform,
    pub amplitude: f32,
    pub adsr: AdsrParams,
    pub filter_cfg: FilterConfig,
    pub cutoff: f32,
    pub resonance: f32,
    pub lfo_cfg: LfoConfig,
    pub lfo_rate: f32,
    pub lfo_depth: f32,
    pub effects_cfg: EffectsConfig,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub reverb_mix: f32,
    pub chorus_mix: f32,
}

impl Preset {
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(io::Error::other)?;
        fs::write(path, json)
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn presets_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".synthesis")
            .join("presets")
    }

    pub fn list_user_presets() -> Vec<String> {
        let dir = Self::presets_dir();
        let mut names = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json")
                    && let Some(stem) = path.file_stem()
                {
                    names.push(stem.to_string_lossy().into_owned());
                }
            }
        }
        names.sort();
        names
    }

    pub fn factory_presets() -> Vec<Preset> {
        vec![
            // Init — clean slate
            Preset {
                name: "Init".to_string(),
                waveform: Waveform::Sine,
                amplitude: 0.5,
                adsr: AdsrParams {
                    attack: 0.01,
                    decay: 0.1,
                    sustain: 0.7,
                    release: 0.3,
                },
                filter_cfg: FilterConfig {
                    filter_type: FilterType::Lowpass,
                    enabled: false,
                },
                cutoff: 1000.0,
                resonance: 0.0,
                lfo_cfg: LfoConfig {
                    waveform: LfoWaveform::Sine,
                    target: LfoTarget::Cutoff,
                    enabled: false,
                },
                lfo_rate: 1.0,
                lfo_depth: 0.0,
                effects_cfg: EffectsConfig::default(),
                delay_time: 0.3,
                delay_feedback: 0.3,
                delay_mix: 0.0,
                reverb_mix: 0.0,
                chorus_mix: 0.0,
            },
            // Warm Pad
            Preset {
                name: "Warm Pad".to_string(),
                waveform: Waveform::Saw,
                amplitude: 0.5,
                adsr: AdsrParams {
                    attack: 0.5,
                    decay: 0.3,
                    sustain: 0.6,
                    release: 1.0,
                },
                filter_cfg: FilterConfig {
                    filter_type: FilterType::Lowpass,
                    enabled: true,
                },
                cutoff: 2000.0,
                resonance: 0.0,
                lfo_cfg: LfoConfig {
                    waveform: LfoWaveform::Sine,
                    target: LfoTarget::Cutoff,
                    enabled: true,
                },
                lfo_rate: 0.5,
                lfo_depth: 0.3,
                effects_cfg: EffectsConfig {
                    delay_enabled: false,
                    reverb_enabled: true,
                    chorus_enabled: true,
                    ..EffectsConfig::default()
                },
                delay_time: 0.3,
                delay_feedback: 0.3,
                delay_mix: 0.0,
                reverb_mix: 0.4,
                chorus_mix: 0.3,
            },
            // Sharp Lead
            Preset {
                name: "Sharp Lead".to_string(),
                waveform: Waveform::Square,
                amplitude: 0.5,
                adsr: AdsrParams {
                    attack: 0.01,
                    decay: 0.1,
                    sustain: 0.8,
                    release: 0.2,
                },
                filter_cfg: FilterConfig {
                    filter_type: FilterType::Lowpass,
                    enabled: true,
                },
                cutoff: 5000.0,
                resonance: 0.5,
                lfo_cfg: LfoConfig {
                    waveform: LfoWaveform::Sine,
                    target: LfoTarget::Frequency,
                    enabled: true,
                },
                lfo_rate: 5.0,
                lfo_depth: 0.1,
                effects_cfg: EffectsConfig {
                    delay_enabled: true,
                    reverb_enabled: false,
                    chorus_enabled: false,
                    ..EffectsConfig::default()
                },
                delay_time: 0.3,
                delay_feedback: 0.4,
                delay_mix: 0.3,
                reverb_mix: 0.0,
                chorus_mix: 0.0,
            },
            // Deep Bass
            Preset {
                name: "Deep Bass".to_string(),
                waveform: Waveform::Saw,
                amplitude: 0.5,
                adsr: AdsrParams {
                    attack: 0.01,
                    decay: 0.05,
                    sustain: 0.9,
                    release: 0.1,
                },
                filter_cfg: FilterConfig {
                    filter_type: FilterType::Lowpass,
                    enabled: true,
                },
                cutoff: 800.0,
                resonance: 0.3,
                lfo_cfg: LfoConfig {
                    waveform: LfoWaveform::Sine,
                    target: LfoTarget::Cutoff,
                    enabled: false,
                },
                lfo_rate: 1.0,
                lfo_depth: 0.0,
                effects_cfg: EffectsConfig::default(),
                delay_time: 0.3,
                delay_feedback: 0.3,
                delay_mix: 0.0,
                reverb_mix: 0.0,
                chorus_mix: 0.0,
            },
            // Space FX
            Preset {
                name: "Space FX".to_string(),
                waveform: Waveform::Triangle,
                amplitude: 0.5,
                adsr: AdsrParams {
                    attack: 0.8,
                    decay: 0.5,
                    sustain: 0.4,
                    release: 2.0,
                },
                filter_cfg: FilterConfig {
                    filter_type: FilterType::Highpass,
                    enabled: true,
                },
                cutoff: 500.0,
                resonance: 0.0,
                lfo_cfg: LfoConfig {
                    waveform: LfoWaveform::Triangle,
                    target: LfoTarget::Amplitude,
                    enabled: true,
                },
                lfo_rate: 3.0,
                lfo_depth: 0.6,
                effects_cfg: EffectsConfig {
                    delay_enabled: true,
                    reverb_enabled: true,
                    chorus_enabled: false,
                    order: [EffectSlot::Delay, EffectSlot::Reverb, EffectSlot::Chorus],
                    ..EffectsConfig::default()
                },
                delay_time: 0.5,
                delay_feedback: 0.6,
                delay_mix: 0.4,
                reverb_mix: 0.6,
                chorus_mix: 0.0,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn factory_presets_count() {
        let presets = Preset::factory_presets();
        assert_eq!(presets.len(), 5);
    }

    #[test]
    fn factory_preset_names() {
        let presets = Preset::factory_presets();
        let names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["Init", "Warm Pad", "Sharp Lead", "Deep Bass", "Space FX"]);
    }

    #[test]
    fn preset_save_and_load() {
        let preset = Preset::factory_presets().into_iter().next().unwrap();
        let dir = env::temp_dir().join("synthesis_test_presets");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_init.json");

        preset.save(&path).unwrap();
        assert!(path.exists());

        let loaded = Preset::load(&path).unwrap();
        assert_eq!(loaded.name, "Init");
        assert_eq!(loaded.waveform, Waveform::Sine);
        assert_eq!(loaded.adsr.attack, 0.01);

        // Cleanup
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn preset_serialization_roundtrip() {
        for preset in Preset::factory_presets() {
            let json = serde_json::to_string(&preset).unwrap();
            let loaded: Preset = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded.name, preset.name);
            assert_eq!(loaded.waveform, preset.waveform);
            assert_eq!(loaded.effects_cfg, preset.effects_cfg);
        }
    }

    #[test]
    fn presets_dir_exists_after_save() {
        let preset = Preset::factory_presets().into_iter().next().unwrap();
        let dir = env::temp_dir().join("synthesis_test_dir_check");
        let path = dir.join("check.json");
        preset.save(&path).unwrap();
        assert!(dir.exists());
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
