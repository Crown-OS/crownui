const LANES: usize = 8;

/// How loud one frame was, as linear amplitudes in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Level {
    /// The largest absolute sample.
    pub peak: f32,
    /// The root mean square of the samples.
    pub rms: f32,
}

impl Level {
    /// Measures `samples`, eight lanes at a time so the loop vectorises.
    pub fn measure(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        let (lanes, rest) = samples.as_chunks::<LANES>();
        let mut peaks = [0.0_f32; LANES];
        let mut energies = [0.0_f32; LANES];
        for lane in lanes {
            for ((peak, energy), &sample) in peaks.iter_mut().zip(&mut energies).zip(lane) {
                *peak = peak.max(sample.abs());
                *energy += sample * sample;
            }
        }
        let peak = rest
            .iter()
            .fold(peaks.into_iter().fold(0.0, f32::max), |peak, sample| {
                peak.max(sample.abs())
            });
        let energy = rest
            .iter()
            .fold(energies.into_iter().sum::<f32>(), |energy, sample| {
                energy + sample * sample
            });
        Self {
            peak,
            rms: (energy / samples.len() as f32).sqrt(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_measures_zero() {
        assert_eq!(Level::measure(&[0.0; 480]), Level::default());
        assert_eq!(Level::measure(&[]), Level::default());
    }

    #[test]
    fn a_full_scale_square_wave_has_unit_peak_and_rms() {
        let square: Vec<f32> = (0..483)
            .map(|index| if index % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let level = Level::measure(&square);
        assert_eq!(level.peak, 1.0);
        assert!((level.rms - 1.0).abs() < 1e-6);
    }

    #[test]
    fn the_peak_is_found_in_the_remainder_too() {
        let mut samples = [0.25_f32; 11];
        samples[10] = -0.75;
        let level = Level::measure(&samples);
        assert_eq!(level.peak, 0.75);
        let expected = ((10.0 * 0.0625 + 0.5625) / 11.0_f32).sqrt();
        assert!((level.rms - expected).abs() < 1e-6);
    }
}
