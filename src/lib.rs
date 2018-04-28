//! This is a crate about Music Sequencing, give it a sequence and instruments to play with and it will output PCM.
//! 
//! In this library, everything related to time is in seconds and notes is in hertz, so please do any conversions beforehand.
//! 
//! Samples are all processed as double-precision floats.
//! 
//! # Layout
//! 
//! * The Sequencer uses a Sequence and Instruments to 'play'.
//! * A Sequence is composed of notes with specific parameters.
//! * Instruments are composed of Keys, each of these have a different pitch.

extern crate pcm;

use pcm::{PCMParameters, PCM};
use std::collections::HashMap;

/// The sequencer itself
pub struct MusicSequencer {
    /// PCM Parameters for controlling the project audio
    pub pcm_parameters: PCMParameters,
    /// The Sequence to play
    pub sequence: Sequence,
    /// The Instruments to use for playing
    pub instruments: InstrumentList,
}

/// Contains notes to play in a sequence
pub struct Sequence {
    /// Notes in the Sequence
    pub notes: Vec<Note>,
}

/// Information about a note in a sequence
pub struct Note {
    /// Time at which the note start
    pub start_at: f64,
    /// Time at which the note stop
    pub end_at: f64,
    /// How long this note plays for
    pub duration: f64,
    /// The height for this note
    pub frequency: f64,
    /// Velocity of the key being pressed down
    pub on_velocity: f64,
    /// Velocity when releasing the key
    pub off_velocity: f64,
    /// Instrument to use for this note
    pub instrument_id: u16,
}

/// List of instruments used by the sequencer
pub struct InstrumentList {
    /// Instruments contained in the list
    pub instruments: HashMap<u16, Instrument>,
}

/// Defines how a note being played should sound
pub struct Instrument {
    /// Keys of the instrument
    pub keys: HashMap<f64, Key>,
    /// The Key Generator for generating every needed key. If not specified, push at least one key to 'keys' for the pitch change.
    pub key_gen: Option<Box<KeyGenerator>>,
    /// Is this instrument loopable ? If there is an envelope, this should be set to true.
    pub loopable: bool,
    /// Envelope for the instrument. If not set, the Instrument will play at max loudness all the time.
    pub envelope: Option<Box<Envelope>>,
}

/// Sound for a particular frequency made by an instrument
pub struct Key {
    /// Audio for the key
    pub audio: PCM,
}

/// Used for generating a new key for a particular frequency
pub trait KeyGenerator {
    /// Generates a new key for an instrument.
    /// # Arguments
    /// * frequency - The height that this key should produce
    /// * parameters - PCM Parameters to respect for the output
    fn key_gen(&self, frequency: f64, parameters: PCMParameters) -> PCM;
}

/// Defines how the loudness for an instrument behaves with time
pub trait Envelope {
    /// Defines behavior from start to sustain included.
    /// # Arguments
    /// Time - In seconds, the position to get the amplitude for.
    /// # Output
    /// Output - Amplitude for given time, should be between 0 and 1 included.
    fn before_during_sustain(&self, time: f64) -> f64;
    /// Defines behavior after sustain in the same manner as before and during sustain.
    fn after_sustain(&self, time: f64) -> f64;
}
