//! Stereo panning implementation for Rodio

use rodio::{Sample, Source};
use std::time::Duration;

/// A source that applies stereo panning to another source
pub struct PannedSource<I>
where
    I: Source,
    I::Item: Sample,
{
    input: I,
    #[allow(dead_code)]
    pan: f32,
    channel_idx: u16,
    channels: u16,
    left_gain: f32,
    right_gain: f32,
}

impl<I> PannedSource<I>
where
    I: Source,
    I::Item: Sample,
{
    /// Create a new panned source
    /// pan: -1.0 = full left, 0.0 = center, 1.0 = full right
    pub fn new(input: I, pan: f32) -> Self {
        let pan = pan.clamp(-1.0, 1.0);
        let (left_gain, right_gain) = calculate_pan_volumes(pan);
        let channels = input.channels();

        Self {
            input,
            pan,
            channel_idx: 0,
            channels,
            left_gain,
            right_gain,
        }
    }
}

impl<I> Iterator for PannedSource<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.input.next()?;

        match self.channels {
            1 => {
                // Mono input should use MonoToStereoPanned instead
                panic!("PannedSource expects stereo input, wrap mono with MonoToStereoPanned");
            }
            2 => {
                // Stereo input - apply panning based on channel
                let result = match self.channel_idx {
                    0 => sample.amplify(self.left_gain),  // Left channel
                    1 => sample.amplify(self.right_gain), // Right channel
                    _ => unreachable!(),
                };

                // Advance to next channel
                self.channel_idx = (self.channel_idx + 1) % 2;

                Some(result)
            }
            _ => {
                // Multi-channel: pass through unchanged
                Some(sample)
            }
        }
    }
}

impl<I> Source for PannedSource<I>
where
    I: Source,
    I::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}

/// A source that converts mono to stereo with panning applied
pub struct MonoToStereoPanned<I>
where
    I: Source,
    I::Item: Sample,
{
    input: I,
    left_gain: f32,
    right_gain: f32,
    current_sample: Option<I::Item>,
    channel_idx: u8,
}

impl<I> MonoToStereoPanned<I>
where
    I: Source,
    I::Item: Sample,
{
    /// Create a new mono to stereo panned source
    pub fn new(input: I, left_gain: f32, right_gain: f32) -> Self {
        debug_assert_eq!(input.channels(), 1, "MonoToStereoPanned expects mono input");
        Self {
            input,
            left_gain,
            right_gain,
            current_sample: None,
            channel_idx: 0,
        }
    }
}

impl<I> Iterator for MonoToStereoPanned<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.channel_idx {
            0 => {
                // Get new sample for left channel
                self.current_sample = self.input.next();
                self.channel_idx = 1;
                self.current_sample.map(|s| s.amplify(self.left_gain))
            }
            1 => {
                // Use same sample for right channel
                self.channel_idx = 0;
                self.current_sample.map(|s| s.amplify(self.right_gain))
            }
            _ => unreachable!(),
        }
    }
}

impl<I> Source for MonoToStereoPanned<I>
where
    I: Source,
    I::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        // Double the frame length since we're converting mono to stereo
        self.input.current_frame_len().map(|len| len * 2)
    }

    fn channels(&self) -> u16 {
        2 // Always outputs stereo
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}

/// Calculate left and right channel volumes from pan value
/// Returns (left_volume, right_volume)
pub fn calculate_pan_volumes(pan: f32) -> (f32, f32) {
    let pan = pan.clamp(-1.0, 1.0);

    // Convert pan (-1 to 1) to angle (0 to PI/2)
    let angle = ((pan + 1.0) / 2.0) * std::f32::consts::FRAC_PI_2;

    // Equal power panning
    let left = angle.cos();
    let right = angle.sin();

    (left, right)
}

/// Extension trait to add panning to any source
pub trait SourceExt: Source + Sized
where
    Self::Item: Sample,
{
    /// Apply stereo panning to this source
    fn panned(self, pan: f32) -> PannedSource<Self> {
        PannedSource::new(self, pan)
    }
}

// Implement the extension trait for all sources
impl<S> SourceExt for S
where
    S: Source,
    S::Item: Sample,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    #[test]
    fn test_equal_power_panning() {
        // Center pan
        let (left, right) = calculate_pan_volumes(0.0);
        assert!((left - 0.707).abs() < 0.01); // -3dB
        assert!((right - 0.707).abs() < 0.01);
        assert!((left * left + right * right - 1.0).abs() < 0.01);

        // Hard left
        let (left, right) = calculate_pan_volumes(-1.0);
        assert!((left - 1.0).abs() < 0.01);
        assert!(right.abs() < 0.01);

        // Hard right
        let (left, right) = calculate_pan_volumes(1.0);
        assert!(left.abs() < 0.01);
        assert!((right - 1.0).abs() < 0.01);

        // 45 degrees left
        let (left, right) = calculate_pan_volumes(-0.5);
        assert!((left - 0.924).abs() < 0.01); // cos(22.5°)
        assert!((right - 0.383).abs() < 0.01); // sin(22.5°)
        assert!((left * left + right * right - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_mono_to_stereo_conversion() {
        let mono_data = vec![0.5f32, -0.5, 0.25, -0.25];
        let source = SamplesBuffer::new(1, 44100, mono_data);

        let panned = MonoToStereoPanned::new(source, 1.0, 0.0);
        let output: Vec<f32> = panned.collect();

        // Should have double the samples (stereo)
        assert_eq!(output.len(), 8);

        // Check interleaving: L,R,L,R...
        assert_eq!(output[0], 0.5); // Left
        assert_eq!(output[1], 0.0); // Right (muted)
        assert_eq!(output[2], -0.5); // Left
        assert_eq!(output[3], 0.0); // Right
        assert_eq!(output[4], 0.25); // Left
        assert_eq!(output[5], 0.0); // Right
        assert_eq!(output[6], -0.25); // Left
        assert_eq!(output[7], 0.0); // Right
    }

    #[test]
    fn test_stereo_panning() {
        let stereo_data = vec![1.0f32, 1.0, 0.5, 0.5, -0.5, -0.5];
        let source = SamplesBuffer::new(2, 44100, stereo_data);

        // Pan hard left
        let panned = source.panned(-1.0);
        let output: Vec<f32> = panned.collect();

        assert_eq!(output.len(), 6);
        // Left channel should be at full volume
        assert_eq!(output[0], 1.0); // Left
        assert_eq!(output[1], 0.0); // Right (muted)
        assert_eq!(output[2], 0.5); // Left
        assert_eq!(output[3], 0.0); // Right
        assert_eq!(output[4], -0.5); // Left
        assert_eq!(output[5], 0.0); // Right
    }

    #[test]
    fn test_mono_to_stereo_metadata() {
        let mono_data = vec![0.5f32; 100];
        let source = SamplesBuffer::new(1, 44100, mono_data);

        let panned = MonoToStereoPanned::new(source, 0.707, 0.707);

        // Check metadata
        assert_eq!(panned.channels(), 2);
        assert_eq!(panned.sample_rate(), 44100);

        // Frame length should be doubled
        if let Some(len) = panned.current_frame_len() {
            assert_eq!(len, 200);
        }
    }

    #[test]
    fn test_panning_clamping() {
        // Test values outside range get clamped
        let (left, right) = calculate_pan_volumes(2.0);
        assert_eq!((left, right), calculate_pan_volumes(1.0));

        let (left, right) = calculate_pan_volumes(-2.0);
        assert_eq!((left, right), calculate_pan_volumes(-1.0));
    }
}
