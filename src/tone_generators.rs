use pcm::{Frame, PCMParameters, Sample, PCM};
use {Key, KeyGenerator};
use std::f64::consts::PI;

/// Generates a square wave
pub struct SquareWaveGenerator {}

/// Generates a Sine Wave
pub struct SineWaveGenerator {}

impl KeyGenerator for SquareWaveGenerator {
    fn key_gen(&self, frequency: &f64, parameters: &PCMParameters, duration: &f64) -> Key {
        match parameters.sample_type {
            Sample::Float(_) => {
                let sample_rate = f64::from(parameters.sample_rate); // In Hertz
                let sample_rate_period = sample_rate.recip(); // In Seconds
                let nb_samples = sample_rate * duration; // In number of samples
                let note_period = frequency.recip(); // In seconds
                let half_note_period = note_period / 2f64; // In seconds
                let mut frames = Vec::new();
                let mut pos_sample = 0f64; // In number of samples
                let mut pos_seconds = 0f64; // In seconds
                while pos_sample < nb_samples {
                    let mut samples = Vec::new();
                    if (pos_seconds % note_period) <= half_note_period {
                        for _ in 0..parameters.nb_channels {
                            samples.push(Sample::Float(1f32));
                        }
                    } else {
                        for _ in 0..parameters.nb_channels {
                            samples.push(Sample::Float(-1f32));
                        }
                    }
                    pos_sample += 1f64;
                    pos_seconds += sample_rate_period;
                    frames.push(Frame { samples });
                }
                Key {
                    frequency: *frequency,
                    audio: PCM {
                        parameters: parameters.clone(),
                        loop_info: None,
                        frames,
                    },
                }
            }
            _ => unimplemented!("Cannot generate anything but f32 for now"),
        }
    }
}

impl KeyGenerator for SineWaveGenerator {
    fn key_gen(&self, frequency: &f64, parameters: &PCMParameters, _duration: &f64) -> Key {
        match parameters.sample_type {
            Sample::Float(_) => {
                let nb_samples = f64::from(parameters.sample_rate) / frequency;
                let mut frames = Vec::new();
                let mut sample = 0f64;
                while sample <= nb_samples {
                    let mut samples = Vec::new();
                    for _ in 0..parameters.nb_channels {
                        samples.push(Sample::Float(((sample / nb_samples) * 2f64 * PI).sin() as f32));
                    }
                    frames.push(Frame { samples });
                    sample += 1f64;
                }
                Key {
                    frequency: *frequency,
                    audio: PCM {
                        parameters: parameters.clone(),
                        loop_info: None,
                        frames,
                    },
                }
            }
            _ => unimplemented!("Cannot generate anything but f32 for now"),
        }
    }
}
