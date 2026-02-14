use fundsp::prelude32::*;

/// Filter type for the resonant filter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
}

/// Filter configuration (type + enabled). Changes trigger a graph rebuild.
#[derive(Clone, Copy, PartialEq)]
pub struct FilterConfig {
    pub filter_type: FilterType,
    pub enabled: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Lowpass,
            enabled: false,
        }
    }
}

/// LFO waveform shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LfoWaveform {
    Sine,
    Triangle,
    Saw,
}

/// LFO modulation target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LfoTarget {
    Frequency,
    Cutoff,
    Amplitude,
}

/// LFO configuration (waveform + target + enabled). Changes trigger a graph rebuild.
#[derive(Clone, Copy, PartialEq)]
pub struct LfoConfig {
    pub waveform: LfoWaveform,
    pub target: LfoTarget,
    pub enabled: bool,
}

impl Default for LfoConfig {
    fn default() -> Self {
        Self {
            waveform: LfoWaveform::Sine,
            target: LfoTarget::Cutoff,
            enabled: false,
        }
    }
}

/// Convert a resonance value (0.0–1.0) to a Q factor.
/// 0.0 → 0.5 (gentle), 1.0 → 20.0 (sharp peak).
pub fn resonance_to_q(r: f32) -> f32 {
    0.5 + r * 19.5
}

/// Binary multiply node: 2 inputs → 1 output (input[0] * input[1]).
#[derive(Clone)]
pub struct Mul2;

impl Mul2 {
    pub fn new() -> Self {
        Self
    }
}

impl AudioNode for Mul2 {
    const ID: u64 = 900_001;
    type Inputs = U2;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        [input[0] * input[1]].into()
    }
}

/// Binary add node: 2 inputs → 1 output (input[0] + input[1]).
#[derive(Clone)]
pub struct Add2;

impl Add2 {
    pub fn new() -> Self {
        Self
    }
}

impl AudioNode for Add2 {
    const ID: u64 = 900_002;
    type Inputs = U2;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        [input[0] + input[1]].into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resonance_to_q_range() {
        let q_min = resonance_to_q(0.0);
        let q_max = resonance_to_q(1.0);
        assert!((q_min - 0.5).abs() < 0.001, "q at 0.0 should be 0.5, got {q_min}");
        assert!((q_max - 20.0).abs() < 0.001, "q at 1.0 should be 20.0, got {q_max}");
    }

    #[test]
    fn resonance_to_q_midpoint() {
        let q_mid = resonance_to_q(0.5);
        assert!((q_mid - 10.25).abs() < 0.001, "q at 0.5 should be 10.25, got {q_mid}");
    }

    #[test]
    fn mul2_multiplies_inputs() {
        let mut node = Mul2::new();
        let result = node.tick(&[3.0, 4.0].into());
        assert!((result[0] - 12.0).abs() < 0.001);
    }

    #[test]
    fn mul2_zero_input() {
        let mut node = Mul2::new();
        let result = node.tick(&[0.0, 5.0].into());
        assert!((result[0]).abs() < 0.001);
    }

    #[test]
    fn add2_adds_inputs() {
        let mut node = Add2::new();
        let result = node.tick(&[3.0, 4.0].into());
        assert!((result[0] - 7.0).abs() < 0.001);
    }

    #[test]
    fn add2_negative_inputs() {
        let mut node = Add2::new();
        let result = node.tick(&[-2.0, 5.0].into());
        assert!((result[0] - 3.0).abs() < 0.001);
    }

    #[test]
    fn filter_config_default() {
        let cfg = FilterConfig::default();
        assert_eq!(cfg.filter_type, FilterType::Lowpass);
        assert!(!cfg.enabled);
    }

    #[test]
    fn lfo_config_default() {
        let cfg = LfoConfig::default();
        assert_eq!(cfg.waveform, LfoWaveform::Sine);
        assert_eq!(cfg.target, LfoTarget::Cutoff);
        assert!(!cfg.enabled);
    }
}
