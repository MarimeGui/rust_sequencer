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
//! * Instruments are composed of Keys, each of these have a different pitch.
//! * A Note is something placed in a Sequence that describes when to make a sound and at which pitch
//! * A Key is a sound for a particular pitch that an instrument makes.

// Todo: Implement Panning
//       Make a trait that replaces the FLUT
//       Process other types of data than f32
//       Move the ValidTimeFrequency error to it's own error type
//       Implement looping
//       Track volume in helper
//       Trait for calculating ticks to and from seconds in f64
//       Multi-threading for render() method
//       Apply envelope
//       Implement Pitch changer
//       Check and fix if necessary each key amplitude passing by the render() method
//       Check for overflows everywhere
//       Remove all unimplemented!()
//       Add errors for all panics!() and everything that should be checked in general
//       Make the user pass the Pitch changer rather than implying it if None
//       Integrate a Tempo Helper and a tick counter in helper
//       Allow for global volume control
//       Prevent clicking by multiplying last values of each note
//       New Tone Generators

extern crate pcm;

/// Contains all errors for this Library
pub mod error;
/// Helps the user to import a Sequence
pub mod helper;
/// Pre-made Tone Generators representing different Waveforms for use with the sequencer
pub mod tone_generators;

use error::SequencerError;
use pcm::{Frame, LoopInfo as PCMLoopInfo, PCMParameters, Sample, PCM};
use std::cmp::max;
use std::collections::HashMap;

/// Result type used everywhere in this crate
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
#[derive(Clone, Default)]
pub struct Sequence {
    /// Notes in the Sequence
    pub notes: Vec<Note>,
    /// Different loops in audio
    pub loop_info: Option<Vec<LoopInfo>>,
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
    pub frequency_id: usize,
    /// Velocity of the key being pressed down
    pub on_velocity: f64,
    /// Velocity when releasing the key
    pub off_velocity: f64,
    /// Instrument to use for this note
    pub instrument_id: usize,
}

/// Used to provide indexes for float values, along with error checking and easy conversion between different formats
#[derive(Clone, Default)]
pub struct FrequencyLookupTable {
    /// HashMap used to get a frequency from a float
    pub lut: HashMap<usize, f64>,
}

/// Represents where a loop starts and ends
#[derive(Clone)]
pub struct LoopInfo {
    /// Where the loop starts in seconds
    pub loop_start: f64,
    /// Where the loop ends in seconds
    pub loop_end: f64,
}

/// List of instruments used by the sequencer
pub struct InstrumentTable {
    /// Instruments contained in the list
    pub instruments: HashMap<usize, Instrument>,
}

/// Defines how a note being played should sound
pub struct Instrument {
    /// Keys of the instrument
    pub keys: HashMap<usize, Key>,
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
    /// Generates a new key for an instrument
    /// # Arguments
    /// * frequency - The height that this key should produce
    /// * parameters - PCM Parameters to respect for the output
    /// * duration - The longest time this key will be held for.
    /// This is useful if the generator needs to know how long it needs to run to create a good sound.
    /// Can be completely ignored.
    fn key_gen(&self, frequency: &f64, parameters: &PCMParameters, duration: &f64) -> Key;
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
    /// Runs everything and gives the final PCM
    pub fn render(&mut self) -> Result<PCM> {
        self.gen_instrument_keys()?;
        let max_notes_at_once = self.sequence.calc_max_notes_at_once();
        let amplitude_per_note = f32::from(max_notes_at_once as u16).recip();
        let duration = self.sequence.calc_music_duration();
        let nb_frames = (duration * f64::from(self.pcm_parameters.sample_rate)) as usize;
        let mut out_pcm_data = vec![
            Frame {
                samples: vec![Sample::Float(0f32); self.pcm_parameters.nb_channels as usize],
            };
            nb_frames
        ];
        for note in &self.sequence.notes {
            let to_add = self.instruments
                .get(&note.instrument_id)?
                .gen_sound(&note.frequency_id, &note.duration)?;
            let mut frame_id = 0usize;
            let mut frame_id_out =
                (note.start_at * f64::from(self.pcm_parameters.sample_rate)).round() as usize;
            while frame_id < to_add.frames.len() {
                for sample_id in 0..self.pcm_parameters.nb_channels as usize {
                    match out_pcm_data[frame_id_out].samples[sample_id] {
                        Sample::Float(s1) => match to_add.frames[frame_id].samples[sample_id] {
                            Sample::Float(s2) => {
                                out_pcm_data[frame_id_out].samples[sample_id] = Sample::Float(
                                    s1 + (s2 * amplitude_per_note * (note.on_velocity as f32)),
                                )
                            }
                            _ => unimplemented!(),
                        },
                        _ => unimplemented!(),
                    }
                }
                frame_id += 1;
                frame_id_out += 1;
            }
        }
        Ok(PCM {
            parameters: PCMParameters {
                nb_channels: self.pcm_parameters.nb_channels,
                sample_rate: self.pcm_parameters.sample_rate,
                sample_type: Sample::Float(0f32),
            },
            loop_info: None,
            frames: out_pcm_data,
        })
    }
    /// Generates all frequencies needed for processing
    pub fn gen_instrument_keys(&mut self) -> Result<()> {
        for (instrument_id, frequencies) in &self.sequence.list_frequencies_for_instruments() {
            let instrument = self.instruments.get(instrument_id)?;
            instrument.gen_keys(
                frequencies,
                &self.frequency_lut,
                &PCMParameters {
                    nb_channels: self.pcm_parameters.nb_channels,
                    sample_rate: self.pcm_parameters.sample_rate,
                    sample_type: Sample::Float(0f32),
                },
            )?;
        }
        Ok(())
    }
}

impl Sequence {
    /// Creates an empty new Sequence
    pub fn new() -> Sequence {
        Sequence {
            loop_info: None,
            notes: Vec::new(),
        }
    }
    /// Adds a new note to the sequence
    pub fn add_note(&mut self, new: Note) {
        self.notes.push(new);
    }
    /// Appends another Sequence to this one
    pub fn merge_other(&mut self, other: &mut Sequence) {
        self.notes.append(&mut other.notes);
    }
    /// Sorts all Notes in the sequence by time
    pub fn sort_by_time(&mut self) {
        self.notes
            .sort_by(|a, b| a.start_at.partial_cmp(&b.start_at).unwrap()); // Hopefully nobody decides to put NaNs in the data :)
    }
    /// Calculates the maximum amount of notes that will be played at once throughout the entire sequence
    pub fn calc_max_notes_at_once(&mut self) -> usize {
        if self.notes.is_empty() {
            return 0;
        }
        self.sort_by_time();
        let mut max_notes = 1usize;
        let mut previous_times: Vec<[f64; 2]> = Vec::new();
        for note in &self.notes {
            let mut passed = 0;
            let mut failed = Vec::new();
            let mut id = 0;
            for previous_time in &previous_times {
                if (previous_time[0] <= note.start_at) & (note.start_at < previous_time[1]) {
                    passed += 1
                } else {
                    failed.push(id)
                }
                id += 1;
            }
            max_notes = max(max_notes, passed + 1);
            let mut iter = 0;
            for id in failed {
                previous_times.remove(id - iter);
                iter += 1;
            }
            previous_times.push([note.start_at, note.end_at]);
        }
        max_notes
    }
    /// Generates a HashMap containing what frequencies each instrument will be playing and for how long
    pub fn list_frequencies_for_instruments(&self) -> HashMap<usize, Vec<(usize, f64)>> {
        let mut frequencies_used_by_instruments = HashMap::new();
        for note in &self.notes {
            let frequencies_times = frequencies_used_by_instruments
                .entry(note.instrument_id)
                .or_insert_with(Vec::new);
            match frequencies_times
                .iter()
                .position(|x: &(usize, f64)| x.0 == note.frequency_id)
            {
                None => frequencies_times.push((note.frequency_id, note.duration)),
                Some(id) => {
                    let ft = frequencies_times.get_mut(id).unwrap();
                    ft.1 = if ft.1 > note.duration {
                        ft.1
                    } else {
                        note.duration
                    }
                }
            }
        }
        frequencies_used_by_instruments
    }
    pub fn calc_music_duration(&self) -> f64 {
        let mut duration = 0f64;
        for note in &self.notes {
            if note.end_at > duration {
                duration = note.end_at
            }
        }
        duration
    }
}

impl FrequencyLookupTable {
    pub fn new() -> FrequencyLookupTable {
        FrequencyLookupTable {
            lut: HashMap::new(),
        }
    }
    /// Returns a Frequency for an ID if it exists, otherwise returns an error.
    pub fn get(&self, id: &usize) -> Result<&f64> {
        match self.lut.get(id) {
            Some(v) => {
                v.check_valid_time_frequency()?;
                Ok(v)
            }
            None => Err(SequencerError::NoFrequencyForID(*id)),
        }
    }
}

impl LoopInfo {
    pub fn to_pcm_loop_info(&self, sample_rate: u32) -> PCMLoopInfo {
        PCMLoopInfo {
            loop_start: (self.loop_start * f64::from(sample_rate)) as u64,
            loop_end: (self.loop_end * f64::from(sample_rate)) as u64,
        }
    }
}

impl InstrumentTable {
    /// Returns an Instrument from the list from an ID, returns an error if there is no instrument at specified ID
    pub fn get(&mut self, id: &usize) -> Result<&mut Instrument> {
        match self.instruments.get_mut(id) {
            Some(i) => Ok(i),
            None => Err(SequencerError::NoInstrumentForID(*id)),
        }
    }
}

impl Instrument {
    /// Generates keys with specified frequencies and adds the new keys to the Instrument.
    /// # Arguments
    /// * frequency_ids_durations: The frequency IDs to generate along with the amount of time needed
    /// * f_lut: The FrequencyLookupTable to use for getting an actual frequency from an ID
    /// * parameters: PCM parameters to use when generating new keys
    pub fn gen_keys(
        &mut self,
        frequency_ids_durations: &[(usize, f64)],
        f_lut: &FrequencyLookupTable,
        parameters: &PCMParameters,
    ) -> Result<()> {
        match self.key_generator {
            Some(ref g) => {
                for frequency_id in frequency_ids_durations {
                    self.keys.insert(
                        frequency_id.0,
                        g.key_gen(f_lut.get(&frequency_id.0)?, parameters, &frequency_id.1),
                    );
                }
            }
            None => {
                let pitch_changer = KeyPitchChanger {
                    original_key: self.get_any_key()?.clone(),
                };
                for frequency_id in frequency_ids_durations {
                    self.keys.insert(
                        frequency_id.0,
                        pitch_changer.key_gen(
                            f_lut.get(&frequency_id.0)?,
                            parameters,
                            &frequency_id.1,
                        ),
                    );
                }
            }
        }
        Ok(())
    }
    /// Returns any first key that is available, used for the pitch changer.
    pub fn get_any_key(&self) -> Result<&Key> {
        Ok(match self.keys.values().next() {
            Some(v) => v,
            None => return Err(SequencerError::NoDefaultKeyGiven),
        })
    }
    pub fn gen_sound(&self, frequency_id: &usize, duration: &f64) -> Result<PCM> {
        duration.check_valid_time_frequency()?;
        let key = match self.keys.get(frequency_id) {
            Some(k) => k,
            None => return Err(SequencerError::NoKeyForID(*frequency_id)),
        };
        let needed_frames = (duration * f64::from(key.audio.parameters.sample_rate)) as usize;
        let mut final_sound: Vec<Frame> = Vec::with_capacity(needed_frames);
        let mut frame_position = 0usize;
        if self.loopable {
            while frame_position < needed_frames {
                final_sound.push(
                    key.audio.frames[(frame_position % (key.audio.frames.len() - 1))].clone(),
                );
                frame_position += 1;
            }
        } else {
            let mut last_frame = &key.audio.frames[0];
            while frame_position < needed_frames {
                final_sound.push(match key.audio.frames.get(frame_position) {
                    Some(f) => {
                        last_frame = f;
                        f.clone()
                    }
                    None => last_frame.clone(),
                });
                frame_position += 1;
            }
        }
        Ok(PCM {
            parameters: key.audio.parameters.clone(),
            loop_info: key.audio.loop_info.clone(),
            frames: final_sound,
        })
    }
}

impl KeyGenerator for KeyPitchChanger {
    fn key_gen(&self, _frequency: &f64, _parameters: &PCMParameters, _duration: &f64) -> Key {
        unimplemented!("Cannot change the pitch of a Key for now")
    }
}
