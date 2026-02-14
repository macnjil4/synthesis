use fundsp::prelude32::*;
use serde::{Deserialize, Serialize};

use super::filter::{Add2, Mul2};

/// Effect slot identifiers for ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSlot {
    Delay,
    Reverb,
    Chorus,
}

/// Effects configuration — changes trigger graph rebuild.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectsConfig {
    pub delay_enabled: bool,
    pub reverb_enabled: bool,
    pub chorus_enabled: bool,
    pub order: [EffectSlot; 3],
    // Reverb compile-time params (rebuild on change)
    pub reverb_room_size: f32,
    pub reverb_time: f32,
    // Chorus compile-time params
    pub chorus_separation: f32,
    pub chorus_variation: f32,
    pub chorus_mod_freq: f32,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            delay_enabled: false,
            reverb_enabled: false,
            chorus_enabled: false,
            order: [EffectSlot::Delay, EffectSlot::Reverb, EffectSlot::Chorus],
            reverb_room_size: 10.0,
            reverb_time: 2.0,
            chorus_separation: 0.5,
            chorus_variation: 0.5,
            chorus_mod_freq: 0.2,
        }
    }
}

/// Custom feedback delay AudioNode (1-in, 1-out) with ring buffer.
#[derive(Clone)]
pub struct FeedbackDelay {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_time: Shared,
    feedback: Shared,
    sample_rate: f32,
}

impl FeedbackDelay {
    /// Create a new feedback delay. Buffer sized for max 2 seconds at 48kHz.
    pub fn new(delay_time: &Shared, feedback: &Shared) -> Self {
        let max_samples = (48000.0 * 2.0) as usize + 1;
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            delay_time: delay_time.clone(),
            feedback: feedback.clone(),
            sample_rate: 44100.0,
        }
    }
}

impl AudioNode for FeedbackDelay {
    const ID: u64 = 900_010;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
        let max_samples = (self.sample_rate * 2.0) as usize + 1;
        if self.buffer.len() != max_samples {
            self.buffer = vec![0.0; max_samples];
            self.write_pos = 0;
        }
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.delay_time.value().clamp(0.01, 2.0);
        let fb = self.feedback.value().clamp(0.0, 0.99);
        let delay_samples = (dt * self.sample_rate) as usize;
        let buf_len = self.buffer.len();

        let delay_samples = Ord::min(delay_samples, buf_len - 1);
        let read_pos = (self.write_pos + buf_len - delay_samples) % buf_len;
        let delayed = self.buffer[read_pos];
        let out = input[0] + delayed * fb;
        self.buffer[self.write_pos] = input[0] + delayed * fb;
        self.write_pos = (self.write_pos + 1) % buf_len;
        [out].into()
    }
}

/// Wire a delay effect (dry/wet blend) into a Net. Returns the output NodeId (mono).
pub fn wire_delay(
    net: &mut Net,
    input_id: NodeId,
    delay_time: &Shared,
    feedback: &Shared,
    mix: &Shared,
) -> NodeId {
    // Dry path: signal * (1 - mix)
    let dry_coeff_id = net.push(Box::new(var_fn(mix, |x| 1.0 - x)));
    let dry_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(input_id, 0, dry_mul_id, 0);
    net.connect(dry_coeff_id, 0, dry_mul_id, 1);

    // Wet path: delay(signal) * mix
    let delay_id = net.push(Box::new(An(FeedbackDelay::new(delay_time, feedback))));
    net.connect(input_id, 0, delay_id, 0);

    let wet_coeff_id = net.push(Box::new(var(mix)));
    let wet_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(delay_id, 0, wet_mul_id, 0);
    net.connect(wet_coeff_id, 0, wet_mul_id, 1);

    // Mix dry + wet
    let add_id = net.push(Box::new(An(Add2::new())));
    net.connect(dry_mul_id, 0, add_id, 0);
    net.connect(wet_mul_id, 0, add_id, 1);

    add_id
}

/// Wire a reverb effect (dry/wet blend) into a Net. Stereo in/out.
/// Returns (left_out, right_out) NodeIds.
pub fn wire_reverb(
    net: &mut Net,
    left_id: NodeId,
    right_id: NodeId,
    cfg: &EffectsConfig,
    mix: &Shared,
) -> (NodeId, NodeId) {
    // Reverb node: 2 inputs → 2 outputs
    let reverb_id = net.push(Box::new(
        reverb_stereo(cfg.reverb_room_size, cfg.reverb_time, 0.5),
    ));
    net.connect(left_id, 0, reverb_id, 0);
    net.connect(right_id, 0, reverb_id, 1);

    // For each channel: dry * (1-mix) + wet * mix
    let mut out_ids = [NodeId::default(); 2];
    let input_ids = [left_id, right_id];

    for (ch, &in_id) in input_ids.iter().enumerate() {
        // Dry path
        let dry_coeff_id = net.push(Box::new(var_fn(mix, |x| 1.0 - x)));
        let dry_mul_id = net.push(Box::new(An(Mul2::new())));
        net.connect(in_id, 0, dry_mul_id, 0);
        net.connect(dry_coeff_id, 0, dry_mul_id, 1);

        // Wet path
        let wet_coeff_id = net.push(Box::new(var(mix)));
        let wet_mul_id = net.push(Box::new(An(Mul2::new())));
        net.connect(reverb_id, ch, wet_mul_id, 0);
        net.connect(wet_coeff_id, 0, wet_mul_id, 1);

        // Mix
        let add_id = net.push(Box::new(An(Add2::new())));
        net.connect(dry_mul_id, 0, add_id, 0);
        net.connect(wet_mul_id, 0, add_id, 1);

        out_ids[ch] = add_id;
    }

    (out_ids[0], out_ids[1])
}

/// Wire a chorus effect (dry/wet blend) into a Net. Returns the output NodeId (mono).
pub fn wire_chorus(
    net: &mut Net,
    input_id: NodeId,
    cfg: &EffectsConfig,
    mix: &Shared,
) -> NodeId {
    // Dry path: signal * (1 - mix)
    let dry_coeff_id = net.push(Box::new(var_fn(mix, |x| 1.0 - x)));
    let dry_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(input_id, 0, dry_mul_id, 0);
    net.connect(dry_coeff_id, 0, dry_mul_id, 1);

    // Wet path: chorus(signal) * mix
    let chorus_id = net.push(Box::new(chorus(
        0,
        cfg.chorus_separation,
        cfg.chorus_variation,
        cfg.chorus_mod_freq,
    )));
    net.connect(input_id, 0, chorus_id, 0);

    let wet_coeff_id = net.push(Box::new(var(mix)));
    let wet_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(chorus_id, 0, wet_mul_id, 0);
    net.connect(wet_coeff_id, 0, wet_mul_id, 1);

    // Mix dry + wet
    let add_id = net.push(Box::new(An(Add2::new())));
    net.connect(dry_mul_id, 0, add_id, 0);
    net.connect(wet_mul_id, 0, add_id, 1);

    add_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_config_default() {
        let cfg = EffectsConfig::default();
        assert!(!cfg.delay_enabled);
        assert!(!cfg.reverb_enabled);
        assert!(!cfg.chorus_enabled);
        assert_eq!(
            cfg.order,
            [EffectSlot::Delay, EffectSlot::Reverb, EffectSlot::Chorus]
        );
    }

    #[test]
    fn feedback_delay_produces_output() {
        let dt = Shared::new(0.01);
        let fb = Shared::new(0.0);
        let mut delay = FeedbackDelay::new(&dt, &fb);
        delay.set_sample_rate(44100.0);
        delay.reset();

        // Feed an impulse
        let out = delay.tick(&[1.0].into());
        assert!((out[0] - 1.0).abs() < 0.001, "impulse should pass through");

        // After delay_time seconds, should hear echo with feedback
        let fb_val = 0.5;
        fb.set_value(fb_val);
        let delay_samples = (0.01 * 44100.0) as usize;
        for _ in 0..delay_samples - 1 {
            delay.tick(&[0.0].into());
        }
        let echo = delay.tick(&[0.0].into());
        assert!(
            echo[0].abs() > 0.01,
            "should hear echo after delay time, got {}",
            echo[0]
        );
    }

    #[test]
    fn feedback_delay_no_feedback_decays() {
        let dt = Shared::new(0.05);
        let fb = Shared::new(0.0);
        let mut delay = FeedbackDelay::new(&dt, &fb);
        delay.set_sample_rate(44100.0);
        delay.reset();

        // Feed an impulse
        delay.tick(&[1.0].into());

        // With zero feedback, echo should be silent
        let delay_samples = (0.05 * 44100.0) as usize;
        let mut last = 0.0f32;
        for _ in 0..delay_samples + 10 {
            let out = delay.tick(&[0.0].into());
            last = out[0];
        }
        assert!(
            last.abs() < 0.001,
            "with zero feedback, echo should be silent, got {last}"
        );
    }

    #[test]
    fn wire_delay_produces_mono_output() {
        let dt = Shared::new(0.1);
        let fb = Shared::new(0.3);
        let mix = Shared::new(0.5);

        let mut net = Net::new(0, 1);
        let dc_id = net.push(Box::new(dc(0.5)));
        let out_id = wire_delay(&mut net, dc_id, &dt, &fb, &mix);
        net.connect_output(out_id, 0, 0);

        let mut graph: Box<dyn AudioUnit> = Box::new(net);
        graph.set_sample_rate(44100.0);
        graph.allocate();

        // Run some samples
        let mut has_nonzero = false;
        for _ in 0..1024 {
            let (l, _) = graph.get_stereo();
            if l.abs() > 0.01 {
                has_nonzero = true;
            }
        }
        assert!(has_nonzero, "delay should produce non-zero output");
    }

    #[test]
    fn wire_chorus_produces_mono_output() {
        let mix = Shared::new(0.5);
        let cfg = EffectsConfig::default();

        let mut net = Net::new(0, 1);
        let dc_id = net.push(Box::new(dc(0.5)));
        let out_id = wire_chorus(&mut net, dc_id, &cfg, &mix);
        net.connect_output(out_id, 0, 0);

        let mut graph: Box<dyn AudioUnit> = Box::new(net);
        graph.set_sample_rate(44100.0);
        graph.allocate();

        let mut has_nonzero = false;
        for _ in 0..1024 {
            let (l, _) = graph.get_stereo();
            if l.abs() > 0.01 {
                has_nonzero = true;
            }
        }
        assert!(has_nonzero, "chorus should produce non-zero output");
    }

    #[test]
    fn wire_reverb_produces_stereo_output() {
        let mix = Shared::new(0.5);
        let cfg = EffectsConfig::default();

        let mut net = Net::new(0, 2);
        let dc_l = net.push(Box::new(dc(0.5)));
        let dc_r = net.push(Box::new(dc(0.5)));
        let (out_l, out_r) = wire_reverb(&mut net, dc_l, dc_r, &cfg, &mix);
        net.connect_output(out_l, 0, 0);
        net.connect_output(out_r, 0, 1);

        let mut graph: Box<dyn AudioUnit> = Box::new(net);
        graph.set_sample_rate(44100.0);
        graph.allocate();

        let mut has_nonzero = false;
        for _ in 0..1024 {
            let (l, r) = graph.get_stereo();
            if l.abs() > 0.01 || r.abs() > 0.01 {
                has_nonzero = true;
            }
        }
        assert!(has_nonzero, "reverb should produce non-zero output");
    }

    #[test]
    fn dry_wet_mix_zero_is_dry_only() {
        let dt = Shared::new(0.1);
        let fb = Shared::new(0.5);
        let mix = Shared::new(0.0); // fully dry

        let mut net = Net::new(0, 1);
        let dc_id = net.push(Box::new(dc(0.5)));
        let out_id = wire_delay(&mut net, dc_id, &dt, &fb, &mix);
        net.connect_output(out_id, 0, 0);

        let mut graph: Box<dyn AudioUnit> = Box::new(net);
        graph.set_sample_rate(44100.0);
        graph.allocate();

        // With mix=0, output should equal input (0.5)
        // Run enough samples for dc to settle
        for _ in 0..512 {
            graph.get_stereo();
        }
        let (l, _) = graph.get_stereo();
        assert!(
            (l - 0.5).abs() < 0.1,
            "with mix=0, output should be ~0.5 (dry), got {l}"
        );
    }
}
