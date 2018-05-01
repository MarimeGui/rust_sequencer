//! This is a crate about Music Sequencing, give it a sequence and instruments to play with and it will output PCM.
//!
//! In this library, everything related to time is in seconds and notes is in hertz, so please do any conversions beforehand.
//!
//! Samples are all processed as double-precision floats.
//!
//! # Architecture
//!
//! * The Sequencer uses a Sequence and Instruments to 'play'.
//! * A Sequence is composed of notes with specific parameters.
//! * Instruments are composed of Keys, each of these have a different frequency.
//! * A Note is something placed in a Sequence that describes when and how to make a sound.
//! * A Key is a sound for a particular frequency that an instrument makes.

extern crate pcm;

mod error;

use error::SequencerError;
use pcm::{PCMParameters, PCM};
use std::cmp::max;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, SequencerError>;

/// Makes sure that a value is a usable Time or Frequency
trait ValidTimeFrequency {
    /// Checks for validity of value for use as a Time or a Frequency
    fn is_valid_time_frequency(&self) -> bool;
    /// Returns nothing if valid, but returns an error if not valid
    fn check_valid_time_frequency(&self) -> Result<()>;
}

impl ValidTimeFrequency for f64 {
    fn is_valid_time_frequency(&self) -> bool {
        (self.is_normal()) & (self > &0f64)
    }
    fn check_valid_time_frequency(&self) -> Result<()> {
        if !self.is_valid_time_frequency() {
            return Err(SequencerError::ImpossibleTimeOrFrequency(*self));
        }
        Ok(())
    }
}

/// The sequencer itself
pub struct MusicSequencer {
    /// PCM Parameters for controlling the project audio
    pub pcm_parameters: PCMParameters,
    /// The Sequence to play
    pub sequence: Sequence,
    /// The Instruments to use for playing
    pub instruments: InstrumentTable,
    /// Table used for storing all possible note frequencies
    pub frequency_lut: FrequencyLookupTable,
}

/// Contains notes to play in a sequence
pub struct Sequence {
    /// Notes in the Sequence
    pub notes: Vec<Note>,
    /// Different loops in audio
    pub loop_info: Vec<LoopInfo>,
}

/// Information about a note in a sequence
#[derive(Clone)]
pub struct Note {
    /// Time at which the note start
    pub start_at: f64,
    /// Time at which the note stop
    pub end_at: f64,
    /// How long this note plays for
    pub duration: f64,
    /// The height for this note, key for the Frequency Lookup Table
    pub frequency: u32,
    /// Velocity of the key being pressed down
    pub on_velocity: f64,
    /// Velocity when releasing the key
    pub off_velocity: f64,
    /// Instrument to use for this note
    pub instrument_id: u16,
}

/// Used to provide indexes for float values, along with error checking and easy conversion between different formats
pub struct FrequencyLookupTable {
    pub lut: HashMap<u32, f64>,
}

/// Represents where a loop starts and ends
pub struct LoopInfo {
    /// Where the loop starts in seconds
    pub loop_start: f64,
    /// Where the loop ends in seconds
    pub loop_end: f64,
}

/// List of instruments used by the sequencer
pub struct InstrumentTable {
    /// Instruments contained in the list
    pub instruments: HashMap<u16, Instrument>,
}

/// Defines how a note being played should sound
pub struct Instrument {
    /// Keys of the instrument
    pub keys: HashMap<u32, Key>,
    /// The Key Generator for generating every needed key. If not specified, push at least one key to 'keys' for the pitch change.
    pub key_generator: Option<Box<KeyGenerator>>,
    /// Is this instrument loopable ? If there is an envelope, this should be set to true.
    pub loopable: bool,
    /// Envelope for the instrument. If not set, the Instrument will play at max loudness all the time.
    pub envelope: Option<Box<Envelope>>,
}

/// Sound for a particular frequency made by an instrument
#[derive(Clone)]
pub struct Key {
    /// Audio for the key
    pub audio: PCM,
    /// Frequency made by this key
    pub frequency: f64,
}

/// Used for generating a new key for a particular frequency
pub trait KeyGenerator {
    /// Generates a new key for an instrument.
    /// # Arguments
    /// * frequency - The height that this key should produce
    /// * parameters - PCM Parameters to respect for the output
    fn key_gen(&self, frequency: &f64, parameters: &PCMParameters) -> Key;
}

/// Changes the pitch of an already existing key for crating the others, fallback if there is nothing else to use.
pub struct KeyPitchChanger {
    /// The Original key to use for pitch change
    pub original_key: Key,
}

/// Defines how the loudness for an instrument behaves with time
pub trait Envelope {
    /// Defines behavior from start to sustain included.
    /// # Arguments
    /// Time - In seconds, the position to get the amplitude for.
    /// # Output
    /// Output - Amplitude for given time, should be between 0 and 1 included.
    fn before_during_sustain(&self, time: &f64) -> f64;
    /// Defines behavior after sustain in the same manner as before and during sustain.
    fn after_sustain(&self, time: &f64) -> f64;
}

impl MusicSequencer {
    pub fn render(&self) -> PCM {
        unimplemented!();
    }
    pub fn gen_instrument_keys(&mut self) -> Result<()> {
        for (instrument_id, frequencies) in &self.sequence.list_frequencies_for_instruments() {
            let instrument = self.instruments.get(instrument_id)?;
            instrument.gen_keys(frequencies, &self.frequency_lut, &self.pcm_parameters)?;
        }
        Ok(())
    }
}

impl Sequence {
    pub fn sort_by_time(&mut self) {
        self.notes
            .sort_by(|a, b| a.start_at.partial_cmp(&b.start_at).unwrap()); // Hopefully nobody decides to put NaNs in the data :)
    }
    pub fn calc_max_notes_at_once(&mut self) -> u32 {
        if self.notes.is_empty() {
            return 0;
        }
        self.sort_by_time();
        let mut max_notes_at_once = 1u32;
        let mut to_delete = Vec::new();
        let mut current_index: u32;
        let mut notes_to_compare: Vec<Note> = Vec::new();
        for current_note in &self.notes {
            to_delete.clear();
            current_index = 0;
            for comparing_note in &notes_to_compare {
                if current_note.start_at > comparing_note.end_at {
                    to_delete.push(current_index);
                }
                current_index += 1;
            }
            for index in &to_delete {
                notes_to_compare.remove(*index as usize);
            }
            notes_to_compare.push(current_note.clone());
            max_notes_at_once = max(max_notes_at_once, notes_to_compare.len() as u32);
        }
        max_notes_at_once
    }
    pub fn list_frequencies_for_instruments(&self) -> HashMap<u16, Vec<u32>> {
        let mut frequencies_used_by_instruments = HashMap::new();
        for note in &self.notes {
            let frequencies = frequencies_used_by_instruments
                .entry(note.instrument_id)
                .or_insert_with(Vec::new);
            if !(frequencies.contains(&note.frequency)) {
                frequencies.push(note.frequency);
            }
        }
        frequencies_used_by_instruments
    }
}

impl FrequencyLookupTable {
    pub fn get(&self, id: &u32) -> Result<&f64> {
        match self.lut.get(id) {
            Some(v) => {
                v.check_valid_time_frequency()?;
                Ok(v)
            }
            None => Err(SequencerError::NoFrequencyForID(*id)),
        }
    }
}

impl InstrumentTable {
    pub fn get(&mut self, id: &u16) -> Result<&mut Instrument> {
        match self.instruments.get_mut(id) {
            Some(i) => Ok(i),
            None => Err(SequencerError::NoInstrumentForID(*id)),
        }
    }
}

impl Instrument {
    pub fn gen_keys(
        &mut self,
        frequency_ids: &[u32],
        f_lut: &FrequencyLookupTable,
        parameters: &PCMParameters,
    ) -> Result<()> {
        match self.key_generator {
            Some(ref g) => {
                for frequency_id in frequency_ids {
                    self.keys.insert(
                        frequency_id.clone(),
                        g.key_gen(f_lut.get(frequency_id)?, parameters),
                    );
                }
            }
            None => {
                let pitch_changer = KeyPitchChanger {
                    original_key: self.get_any_key()?.clone(),
                };
                for frequency_id in frequency_ids {
                    self.keys.insert(
                        frequency_id.clone(),
                        pitch_changer.key_gen(f_lut.get(frequency_id)?, parameters),
                    );
                }
            }
        }
        Ok(())
    }
    pub fn get_any_key(&self) -> Result<&Key> {
        Ok(match self.keys.values().next() {
            Some(v) => v,
            None => return Err(SequencerError::NoDefaultKeyGiven),
        })
    }
}

impl KeyGenerator for KeyPitchChanger {
    fn key_gen(&self, _frequency: &f64, _parameters: &PCMParameters) -> Key {
        unimplemented!()
    }
}
