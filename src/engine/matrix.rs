use std::sync::Arc;

use fundsp::prelude32::*;

use super::drum_sample::{build_sample_drum_voice_unit, SampleDrumVoiceShared};
use super::effects::{wire_chorus, wire_delay, wire_reverb, EffectSlot, EffectsConfig};
use super::filter::Add2;
use super::oscillator::build_voice_unit;
use super::voice::{Voice, VoiceConfig, VoiceShared};

/// Build a combined Matrix graph with 8 lead + 8 drum + 8 bass voices,
/// summed together through a single shared effects chain.
/// All voice sets play simultaneously; mode switching only affects the UI.
#[allow(clippy::too_many_arguments)]
pub fn build_matrix_graph(
    lead_voices: &[Voice],
    lead_configs: &[VoiceConfig],
    lead_shared: &[VoiceShared],
    drum_voices: &[Voice],
    drum_shared: &[SampleDrumVoiceShared],
    drum_buffers: &Arc<Vec<Vec<f32>>>,
    bass_voices: &[Voice],
    bass_configs: &[VoiceConfig],
    bass_shared: &[VoiceShared],
    master_amp: &Shared,
    effects_cfg: &EffectsConfig,
    delay_time: &Shared,
    delay_feedback: &Shared,
    delay_mix: &Shared,
    reverb_mix: &Shared,
    chorus_mix: &Shared,
) -> (Box<dyn AudioUnit>, Snoop, Snoop) {
    let mut net = Net::new(0, 2);

    let (snoop_l, snoop_backend_l) = snoop(32768);
    let (snoop_r, snoop_backend_r) = snoop(32768);

    let snoop_l_id = net.push(Box::new(snoop_backend_l));
    let snoop_r_id = net.push(Box::new(snoop_backend_r));

    net.connect_output(snoop_l_id, 0, 0);
    net.connect_output(snoop_r_id, 0, 1);

    // ── Build 8 lead voice units ──
    let mut all_voice_ids: Vec<NodeId> = Vec::new();
    for (i, voice) in lead_voices.iter().enumerate() {
        let cfg = &lead_configs[i];
        let shared = &lead_shared[i];
        let unit = build_voice_unit(
            cfg.waveform,
            &voice.freq,
            &voice.gate,
            &voice.velocity,
            master_amp,
            &shared.level,
            &cfg.adsr,
            &cfg.filter_cfg,
            &shared.cutoff,
            &shared.resonance,
            &cfg.lfo_cfg,
            &shared.lfo_rate,
            &shared.lfo_depth,
        );
        all_voice_ids.push(net.push(unit));
    }

    // ── Build 8 sample-based drum voice units ──
    for (i, voice) in drum_voices.iter().enumerate() {
        let shared = &drum_shared[i];
        let unit = build_sample_drum_voice_unit(
            &voice.velocity,
            master_amp,
            shared,
            drum_buffers,
        );
        all_voice_ids.push(net.push(unit));
    }

    // ── Build 8 bass voice units ──
    for (i, voice) in bass_voices.iter().enumerate() {
        let cfg = &bass_configs[i];
        let shared = &bass_shared[i];
        let unit = build_voice_unit(
            cfg.waveform,
            &voice.freq,
            &voice.gate,
            &voice.velocity,
            master_amp,
            &shared.level,
            &cfg.adsr,
            &cfg.filter_cfg,
            &shared.cutoff,
            &shared.resonance,
            &cfg.lfo_cfg,
            &shared.lfo_rate,
            &shared.lfo_depth,
        );
        all_voice_ids.push(net.push(unit));
    }

    // ── Sum all 24 voices (8 lead + 8 drum + 8 bass) ──

    // Sum left channels: chain of Add2 nodes
    let mut sum_l_id = {
        let p = net.push(Box::new(pass()));
        net.connect(all_voice_ids[0], 0, p, 0);
        p
    };
    for &vid in &all_voice_ids[1..] {
        let add = net.push(Box::new(An(Add2::new())));
        net.connect(sum_l_id, 0, add, 0);
        net.connect(vid, 0, add, 1);
        sum_l_id = add;
    }

    // Sum right channels: chain of Add2 nodes
    let mut sum_r_id = {
        let p = net.push(Box::new(pass()));
        net.connect(all_voice_ids[0], 1, p, 0);
        p
    };
    for &vid in &all_voice_ids[1..] {
        let add = net.push(Box::new(An(Add2::new())));
        net.connect(sum_r_id, 0, add, 0);
        net.connect(vid, 1, add, 1);
        sum_r_id = add;
    }

    // ── Shared effects chain ──
    let mut chain_l = sum_l_id;
    let mut chain_r = sum_r_id;

    for &slot in &effects_cfg.order {
        match slot {
            EffectSlot::Delay if effects_cfg.delay_enabled => {
                chain_l = wire_delay(&mut net, chain_l, delay_time, delay_feedback, delay_mix);
                chain_r = wire_delay(&mut net, chain_r, delay_time, delay_feedback, delay_mix);
            }
            EffectSlot::Reverb if effects_cfg.reverb_enabled => {
                let (rl, rr) = wire_reverb(&mut net, chain_l, chain_r, effects_cfg, reverb_mix);
                chain_l = rl;
                chain_r = rr;
            }
            EffectSlot::Chorus if effects_cfg.chorus_enabled => {
                chain_l = wire_chorus(&mut net, chain_l, effects_cfg, chorus_mix);
                chain_r = wire_chorus(&mut net, chain_r, effects_cfg, chorus_mix);
            }
            _ => {}
        }
    }

    // Connect effects output to snoops
    net.connect(chain_l, 0, snoop_l_id, 0);
    net.connect(chain_r, 0, snoop_r_id, 0);

    (Box::new(net), snoop_l, snoop_r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::voice::Voice;

    const SAMPLE_RATE: f64 = 44100.0;

    fn make_test_buffers() -> Arc<Vec<Vec<f32>>> {
        let mut buffers = Vec::new();
        for i in 0..16 {
            let freq = 100.0 + i as f32 * 50.0;
            let len = 4410;
            let buf: Vec<f32> = (0..len)
                .map(|s| (2.0 * std::f32::consts::PI * freq * s as f32 / 44100.0).sin() * 0.8)
                .collect();
            buffers.push(buf);
        }
        Arc::new(buffers)
    }

    #[test]
    fn build_matrix_graph_returns_stereo() {
        let lead_voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let lead_configs: Vec<VoiceConfig> = (0..8).map(|_| VoiceConfig::default()).collect();
        let lead_shared: Vec<VoiceShared> =
            lead_configs.iter().map(VoiceShared::new).collect();
        let drum_voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let drum_shared: Vec<SampleDrumVoiceShared> =
            (0..8).map(|_| SampleDrumVoiceShared::new()).collect();
        let drum_buffers = make_test_buffers();
        let bass_voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let bass_configs: Vec<VoiceConfig> = (0..8).map(|_| VoiceConfig::default()).collect();
        let bass_shared: Vec<VoiceShared> =
            bass_configs.iter().map(VoiceShared::new).collect();
        let master = Shared::new(0.5);
        let ecfg = EffectsConfig::default();
        let dt = Shared::new(0.3);
        let fb = Shared::new(0.3);
        let dm = Shared::new(0.0);
        let rm = Shared::new(0.0);
        let cm = Shared::new(0.0);

        let (graph, _, _) = build_matrix_graph(
            &lead_voices, &lead_configs, &lead_shared,
            &drum_voices, &drum_shared, &drum_buffers,
            &bass_voices, &bass_configs, &bass_shared,
            &master, &ecfg, &dt, &fb, &dm, &rm, &cm,
        );
        assert_eq!(graph.inputs(), 0);
        assert_eq!(graph.outputs(), 2);
    }

    #[test]
    fn build_matrix_graph_lead_and_drum_both_contribute() {
        let lead_voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let lead_configs: Vec<VoiceConfig> = (0..8).map(|_| VoiceConfig::default()).collect();
        let lead_shared: Vec<VoiceShared> =
            lead_configs.iter().map(VoiceShared::new).collect();
        let drum_voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let drum_shared: Vec<SampleDrumVoiceShared> =
            (0..8).map(|_| SampleDrumVoiceShared::new()).collect();
        let drum_buffers = make_test_buffers();
        let bass_voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let bass_configs: Vec<VoiceConfig> = (0..8).map(|_| VoiceConfig::default()).collect();
        let bass_shared: Vec<VoiceShared> =
            bass_configs.iter().map(VoiceShared::new).collect();
        let master = Shared::new(0.5);
        let ecfg = EffectsConfig::default();
        let dt = Shared::new(0.3);
        let fb = Shared::new(0.3);
        let dm = Shared::new(0.0);
        let rm = Shared::new(0.0);
        let cm = Shared::new(0.0);

        let (mut graph, _, _) = build_matrix_graph(
            &lead_voices, &lead_configs, &lead_shared,
            &drum_voices, &drum_shared, &drum_buffers,
            &bass_voices, &bass_configs, &bass_shared,
            &master, &ecfg, &dt, &fb, &dm, &rm, &cm,
        );
        graph.set_sample_rate(SAMPLE_RATE);
        graph.allocate();

        // Warm up
        for _ in 0..256 {
            graph.get_stereo();
        }

        // Trigger lead voice 0 (A4 = 440Hz)
        lead_voices[0].freq.set_value(440.0);
        lead_voices[0].gate.set_value(1.0);
        lead_voices[0].velocity.set_value(1.0);

        let mut lead_sound = false;
        for _ in 0..4096 {
            let (l, r) = graph.get_stereo();
            if l.abs() > 0.001 || r.abs() > 0.001 {
                lead_sound = true;
                break;
            }
        }
        assert!(lead_sound, "lead voice should produce sound");

        // Release lead, trigger drum voice 0
        lead_voices[0].gate.set_value(0.0);
        for _ in 0..22050 {
            graph.get_stereo();
        }

        // Set sample index to kick (14) and trigger via counter
        drum_shared[0].sample_index.set_value(14.0);
        drum_shared[0].trigger.set_value(1.0);
        drum_voices[0].velocity.set_value(1.0);

        let mut drum_sound = false;
        for _ in 0..4096 {
            let (l, r) = graph.get_stereo();
            if l.abs() > 0.001 || r.abs() > 0.001 {
                drum_sound = true;
                break;
            }
        }
        assert!(drum_sound, "drum voice should produce sound");
    }
}
