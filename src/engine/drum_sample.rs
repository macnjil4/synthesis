use std::sync::Arc;

use fundsp::prelude32::*;

use super::filter::Mul2;

const SAMPLE_NAMES: [&str; 16] = [
    "00-crash",
    "01-ride",
    "02-hihat-open",
    "03-hihat-closed",
    "04-clap",
    "05-rimshot",
    "06-snare",
    "07-tom-hi",
    "08-tom-mid",
    "09-tom-low",
    "10-conga-hi",
    "11-conga-lo",
    "12-cowbell",
    "13-clave",
    "14-kick",
    "15-kick-hard",
];

// ── SamplePlayer AudioNode ──

/// Custom AudioNode that plays one-shot WAV samples from pre-loaded buffers.
/// Detects changes in a trigger counter to start playback, avoiding gate-edge
/// timing issues when note_off + note_on happen in the same UI frame.
#[derive(Clone)]
pub struct SamplePlayer {
    buffers: Arc<Vec<Vec<f32>>>,
    sample_index: Shared,
    trigger: Shared,
    pitch_ratio: Shared,
    play_pos: f64,
    prev_trigger: f32,
    current_buffer_idx: usize,
    playing: bool,
}

impl SamplePlayer {
    pub fn new(
        buffers: &Arc<Vec<Vec<f32>>>,
        sample_index: &Shared,
        trigger: &Shared,
        pitch_ratio: &Shared,
    ) -> Self {
        Self {
            buffers: Arc::clone(buffers),
            sample_index: sample_index.clone(),
            trigger: trigger.clone(),
            pitch_ratio: pitch_ratio.clone(),
            play_pos: 0.0,
            prev_trigger: 0.0,
            current_buffer_idx: 0,
            playing: false,
        }
    }
}

impl AudioNode for SamplePlayer {
    const ID: u64 = 900_020;
    type Inputs = U0;
    type Outputs = U1;

    fn reset(&mut self) {
        self.play_pos = 0.0;
        self.prev_trigger = 0.0;
        self.current_buffer_idx = 0;
        self.playing = false;
    }

    #[inline]
    fn tick(&mut self, _: &Frame<f32, U0>) -> Frame<f32, U1> {
        let trig = self.trigger.value();

        // Trigger when the counter changes (no gate-edge timing issues)
        if trig != self.prev_trigger {
            self.prev_trigger = trig;
            self.current_buffer_idx = Ord::min(self.sample_index.value() as usize, 15);
            self.play_pos = 0.0;
            self.playing = true;
        }

        if !self.playing {
            return [0.0].into();
        }

        let buf = &self.buffers[self.current_buffer_idx];
        let idx = self.play_pos as usize;
        let out = if idx < buf.len() {
            // Linear interpolation for smooth pitch shifting
            let frac = self.play_pos - idx as f64;
            let s0 = buf[idx];
            let s1 = if idx + 1 < buf.len() {
                buf[idx + 1]
            } else {
                0.0
            };
            s0 + (s1 - s0) * frac as f32
        } else {
            0.0
        };

        self.play_pos += self.pitch_ratio.value() as f64;
        [out].into()
    }
}

// ── Per-voice shared parameters ──

/// Atomic shared parameters for a single sample-based drum voice.
pub struct SampleDrumVoiceShared {
    pub sample_index: Shared,
    pub pitch_ratio: Shared,
    pub level: Shared,
    pub trigger: Shared,
}

impl SampleDrumVoiceShared {
    pub fn new() -> Self {
        Self {
            sample_index: Shared::new(14.0), // default to kick
            pitch_ratio: Shared::new(1.0),
            level: Shared::new(1.0),
            trigger: Shared::new(0.0),
        }
    }
}

// ── Voice graph construction ──

/// Build a single sample-based drum voice.
///
/// Signal chain:
///   SamplePlayer → × velocity → × level → × master_amp → stereo split
pub fn build_sample_drum_voice_unit(
    velocity: &Shared,
    master_amp: &Shared,
    shared: &SampleDrumVoiceShared,
    buffers: &Arc<Vec<Vec<f32>>>,
) -> Box<dyn AudioUnit> {
    let mut net = Net::new(0, 2);

    // SamplePlayer: 0 inputs, 1 output (triggered by shared.trigger counter)
    let player_id = net.push(Box::new(An(SamplePlayer::new(
        buffers,
        &shared.sample_index,
        &shared.trigger,
        &shared.pitch_ratio,
    ))));

    // × velocity
    let vel_id = net.push(Box::new(var(velocity)));
    let vel_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(player_id, 0, vel_mul_id, 0);
    net.connect(vel_id, 0, vel_mul_id, 1);

    // × per-voice level
    let lvl_id = net.push(Box::new(var(&shared.level) >> follow(0.01)));
    let lvl_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(vel_mul_id, 0, lvl_mul_id, 0);
    net.connect(lvl_id, 0, lvl_mul_id, 1);

    // × master amplitude
    let amp_id = net.push(Box::new(var(master_amp) >> follow(0.01)));
    let amp_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(lvl_mul_id, 0, amp_mul_id, 0);
    net.connect(amp_id, 0, amp_mul_id, 1);

    // Split to stereo
    let split_id = net.push(Box::new(split::<U2>()));
    net.connect(amp_mul_id, 0, split_id, 0);
    net.connect_output(split_id, 0, 0);
    net.connect_output(split_id, 1, 1);

    Box::new(net)
}

// ── Sample loading ──

/// Load a drum kit from WAV files in `samples/{dir_name}/`.
/// Returns 16 mono f32 buffers (resampled to output_sample_rate).
/// Missing files produce empty buffers (silence).
pub fn load_drum_kit(dir_name: &str, output_sample_rate: f64) -> Arc<Vec<Vec<f32>>> {
    let base_path = samples_dir(dir_name);
    let mut buffers = Vec::with_capacity(16);

    for name in &SAMPLE_NAMES {
        let path = base_path.join(format!("{name}.wav"));
        let buf = load_wav(&path, output_sample_rate);
        buffers.push(buf);
    }

    Arc::new(buffers)
}

fn samples_dir(dir_name: &str) -> std::path::PathBuf {
    let relative = std::path::PathBuf::from("samples").join(dir_name);
    if relative.is_dir() {
        return relative;
    }
    // Try relative to executable
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let alt = parent.join("samples").join(dir_name);
        if alt.is_dir() {
            return alt;
        }
    }
    relative
}

fn load_wav(path: &std::path::Path, output_sample_rate: f64) -> Vec<f32> {
    let mut reader = match hound::WavReader::open(path) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let spec = reader.spec();
    let file_sr = spec.sample_rate as f64;

    // Read samples to f32
    let raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().filter_map(|s| s.ok()).collect(),
        hound::SampleFormat::Int => {
            let max = (1u32 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max)
                .collect()
        }
    };

    if raw.is_empty() {
        return Vec::new();
    }

    // Mix to mono if multi-channel
    let mono: Vec<f32> = if spec.channels > 1 {
        let ch = spec.channels as usize;
        raw.chunks(ch)
            .map(|frame| frame.iter().sum::<f32>() / ch as f32)
            .collect()
    } else {
        raw
    };

    // Resample if file rate differs from output rate
    if (file_sr - output_sample_rate).abs() > 1.0 && !mono.is_empty() {
        let ratio = output_sample_rate / file_sr;
        let new_len = (mono.len() as f64 * ratio) as usize;
        let mut resampled = Vec::with_capacity(new_len);
        for i in 0..new_len {
            let src = i as f64 / ratio;
            let idx = src as usize;
            let frac = src - idx as f64;
            let s0 = mono.get(idx).copied().unwrap_or(0.0);
            let s1 = mono.get(idx + 1).copied().unwrap_or(s0);
            resampled.push(s0 + (s1 - s0) * frac as f32);
        }
        resampled
    } else {
        mono
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 44100.0;

    fn make_test_buffers() -> Arc<Vec<Vec<f32>>> {
        let mut buffers = Vec::new();
        for i in 0..16 {
            // Each buffer is a short sine burst at different frequency
            let freq = 100.0 + i as f32 * 50.0;
            let len = 4410; // 0.1s at 44100
            let buf: Vec<f32> = (0..len)
                .map(|s| (2.0 * std::f32::consts::PI * freq * s as f32 / 44100.0).sin() * 0.8)
                .collect();
            buffers.push(buf);
        }
        Arc::new(buffers)
    }

    #[test]
    fn sample_player_produces_sound_on_trigger() {
        let buffers = make_test_buffers();
        let sample_index = Shared::new(14.0); // kick
        let trigger = Shared::new(0.0);
        let pitch_ratio = Shared::new(1.0);

        let mut player = SamplePlayer::new(&buffers, &sample_index, &trigger, &pitch_ratio);

        // No sound without trigger
        let out = player.tick(&Frame::default());
        assert_eq!(out[0], 0.0);

        // Trigger by incrementing counter
        trigger.set_value(1.0);
        let mut has_sound = false;
        for _ in 0..1000 {
            let out = player.tick(&Frame::default());
            if out[0].abs() > 0.01 {
                has_sound = true;
                break;
            }
        }
        assert!(has_sound, "sample player should produce sound on trigger");
    }

    #[test]
    fn sample_player_silent_without_trigger() {
        let buffers = make_test_buffers();
        let sample_index = Shared::new(14.0);
        let trigger = Shared::new(0.0);
        let pitch_ratio = Shared::new(1.0);

        let mut player = SamplePlayer::new(&buffers, &sample_index, &trigger, &pitch_ratio);

        let mut max_val = 0.0f32;
        for _ in 0..1000 {
            let out = player.tick(&Frame::default());
            max_val = max_val.max(out[0].abs());
        }
        assert!(
            max_val < 0.001,
            "sample player should be silent without trigger, got {max_val}"
        );
    }

    #[test]
    fn sample_player_retrigger_resets_position() {
        let buffers = make_test_buffers();
        let sample_index = Shared::new(6.0); // snare
        let trigger = Shared::new(0.0);
        let pitch_ratio = Shared::new(1.0);

        let mut player = SamplePlayer::new(&buffers, &sample_index, &trigger, &pitch_ratio);

        // First trigger
        trigger.set_value(1.0);
        for _ in 0..100 {
            player.tick(&Frame::default());
        }

        // Retrigger by incrementing counter (no need for gate cycle)
        trigger.set_value(2.0);

        // Position should reset — sample plays from start again
        let out = player.tick(&Frame::default());
        assert!(player.play_pos < 5.0, "play position should have reset");
        let _ = out;
    }

    #[test]
    fn sample_player_pitch_ratio_changes_speed() {
        let buffers = make_test_buffers();
        let sample_index = Shared::new(14.0);
        let trigger = Shared::new(0.0);
        let pitch_ratio = Shared::new(2.0); // double speed

        let mut player = SamplePlayer::new(&buffers, &sample_index, &trigger, &pitch_ratio);

        trigger.set_value(1.0);
        for _ in 0..100 {
            player.tick(&Frame::default());
        }

        // At 2x speed, position should be ~200 after 100 ticks
        assert!(
            player.play_pos > 190.0,
            "at 2x pitch, position should advance faster, got {}",
            player.play_pos
        );
    }

    #[test]
    fn build_sample_drum_voice_unit_returns_stereo() {
        let buffers = make_test_buffers();
        let vel = Shared::new(1.0);
        let master = Shared::new(0.5);
        let shared = SampleDrumVoiceShared::new();
        let unit = build_sample_drum_voice_unit(&vel, &master, &shared, &buffers);
        assert_eq!(unit.inputs(), 0);
        assert_eq!(unit.outputs(), 2);
    }

    #[test]
    fn sample_drum_voice_produces_sound_on_trigger() {
        let buffers = make_test_buffers();
        let vel = Shared::new(1.0);
        let master = Shared::new(0.5);
        let shared = SampleDrumVoiceShared::new();
        let mut unit = build_sample_drum_voice_unit(&vel, &master, &shared, &buffers);
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();

        // Warm up
        for _ in 0..100 {
            unit.get_stereo();
        }

        // Trigger via counter
        shared.trigger.set_value(1.0);
        let mut has_sound = false;
        for _ in 0..4096 {
            let (l, r) = unit.get_stereo();
            if l.abs() > 0.001 || r.abs() > 0.001 {
                has_sound = true;
                break;
            }
        }
        assert!(
            has_sound,
            "sample drum voice should produce sound on trigger"
        );
    }

    #[test]
    fn load_drum_kit_returns_16_buffers() {
        let kit = load_drum_kit("lm2", SAMPLE_RATE);
        assert_eq!(kit.len(), 16);
    }

    #[test]
    fn load_drum_kit_missing_dir_returns_empty_buffers() {
        let kit = load_drum_kit("nonexistent_kit", SAMPLE_RATE);
        assert_eq!(kit.len(), 16);
        for buf in kit.iter() {
            assert!(buf.is_empty());
        }
    }
}
