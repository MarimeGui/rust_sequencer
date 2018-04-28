//! This is a crate about Music Sequencing, give it a sequence and instruments to play with and it will output PCM.
//! 
//! In this library, everything related to time is in seconds and notes is in hertz, so please do any conversions beforehand.
//! 
//! Samples are all processed as double-precision floats.

extern crate pcm;

use pcm::{PCMParameters, PCM};
use std::collections::HashMap;

/// The sequencer itself
pub struct MusicSequencer {
    pub pcm_parameters: PCMParameters,
    pub sequence: Sequence,
    pub instruments: InstrumentList,
}

/// Contains notes to play in a sequence
pub struct Sequence {
    pub notes: Vec<Note>,
}

/// Information about a note in a sequence
pub struct Note {
    pub start_at: f64,
    pub end_at: f64,
    pub duration: f64,
    pub frequency: f64,
    pub on_velocity: f64,
    pub off_velocity: f64,
    pub instrument_id: u16,
}

/// List of instruments used by the sequencer
pub struct InstrumentList {
    pub instruments: HashMap<u16, Instrument>,
}

/// Defines how a note being played should sound
pub struct Instrument {
    pub keys: HashMap<f64, Key>,
    pub key_gen: Option<Box<KeyGenerator>>,
    pub loopable: bool,
    pub envelope: Option<Box<Envelope>>,
}

/// Sound for a particular frequency made by an instrument
pub struct Key {
    pub audio: PCM,
}

/// Used for generating a new key for a particular frequency
pub trait KeyGenerator {
    /// Takes a frequency in Hertz and outputs PCM with specified parameters for the new key.
    fn key_gen(&self, frequency: f64, parameters: PCMParameters) -> PCM;
}

/// Defines how the loudness for an instrument behaves with time
pub trait Envelope {
    /// Defines behavior from start to sustain included.
    /// Time is in seconds and output is between 0 and 1.
    fn before_during_sustain(&self, time: f64) -> f64;
    /// Defines behavior after sustain.
    /// Time starts when key is released and is in seconds.
    /// Output (representing amplitude) should be between 0 and 1.
    fn after_sustain(&self, time: f64) -> f64;
}
