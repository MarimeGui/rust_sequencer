use std::collections::HashMap;
use std::f64::EPSILON;
use {FrequencyLookupTable, Note, Sequence};

/// Represents a Note missing some information
#[derive(Clone)]
pub struct PartialNote {
    pub start_at: f64,
    pub on_velocity: f64,
}

/// Helps creating a Sequence and a FrequencyLookupTable from another type of sequence
#[derive(Default)]
pub struct SequenceHelper {
    pub current_instruments: HashMap<usize, HashMap<usize, PartialNote>>,
    pub frequency_lut: Option<FrequencyLookupTable>,
    pub frequency_lut_builder: Option<Vec<f64>>,
    pub sequence: Sequence,
    pub at_time: f64,
}

impl SequenceHelper {
    /// Creates a new empty HardwareSequenceHelper
    pub fn new() -> SequenceHelper {
        SequenceHelper {
            current_instruments: HashMap::new(),
            frequency_lut: None,
            frequency_lut_builder: Some(Vec::new()),
            sequence: Sequence::new(),
            at_time: 0f64,
        }
    }
    /// Creates a new empty HardwareSequenceHelper with a already existing FLUT
    pub fn new_with_flut(frequency_lut: FrequencyLookupTable) -> SequenceHelper {
        SequenceHelper {
            current_instruments: HashMap::new(),
            frequency_lut: Some(frequency_lut),
            frequency_lut_builder: None,
            sequence: Sequence::new(),
            at_time: 0f64,
        }
    }
    /// Makes the time go forward in seconds
    pub fn time_forward(&mut self, time_passed: f64) {
        self.at_time += time_passed;
    }
    /// Resets the time to 0
    pub fn reset_time(&mut self) {
        self.at_time = 0f64;
    }
    /// When a new note starts in the sequence
    pub fn start_note(&mut self, frequency: f64, on_velocity: f64, instrument_id: usize) {
        let frequency_id = match &mut self.frequency_lut_builder {
            Some(c) => match c.iter().position(|&x| (x - frequency).abs() < EPSILON) {
                Some(i) => i,
                None => {
                    c.push(frequency);
                    c.len() - 1
                }
            },
            None => panic!("Deserved for not using the correct function !"),
        };
        self.start_note_with_flut(frequency_id, on_velocity, instrument_id);
    }
    /// When a new note starts in the sequence and the Frequency ID is already known
    pub fn start_note_with_flut(
        &mut self,
        frequency_id: usize,
        on_velocity: f64,
        instrument_id: usize,
    ) {
        let freq_hashmap = self.current_instruments
            .entry(instrument_id)
            .or_insert_with(HashMap::new);
        match freq_hashmap.get(&frequency_id) {
            // Or Insert
            None => {
                freq_hashmap.insert(
                    frequency_id,
                    PartialNote {
                        start_at: self.at_time,
                        on_velocity,
                    },
                );
            }
            Some(_) => {}
        }
    }
    /// Stops the note
    pub fn stop_note(&mut self, frequency: f64, off_velocity: f64, instrument_id: usize) {
        let frequency_id = match self.frequency_lut_builder {
            Some(ref c) => match c.iter().position(|&x| (x - frequency).abs() < EPSILON) {
                Some(i) => Some(i),
                None => None,
            },
            None => panic!("Deserved for not using the correct function !"),
        };
        if let Some(id) = frequency_id {
            self.stop_note_with_flut(id, off_velocity, instrument_id)
        }
    }
    /// Stops the note with a known Frequency ID
    pub fn stop_note_with_flut(
        &mut self,
        frequency_id: usize,
        off_velocity: f64,
        instrument_id: usize,
    ) {
        let mut to_remove = false;
        match self.current_instruments.get_mut(&instrument_id) {
            Some(i) => {
                match i.get(&frequency_id) {
                    Some(pn) => {
                        if (self.at_time - pn.start_at) > 0f64 {
                            self.sequence.add_note(Note {
                                start_at: pn.start_at,
                                end_at: self.at_time,
                                duration: self.at_time - pn.start_at,
                                frequency_id,
                                on_velocity: pn.on_velocity,
                                off_velocity,
                                instrument_id,
                            });
                        } else if (self.at_time - pn.start_at) < 0f64 {
                            panic!("A note has a negative duration");
                        }
                        to_remove = true;
                    }
                    None => {}
                }
                if to_remove {
                    i.remove(&frequency_id);
                }
            }
            None => panic!("No instrument for ID"),
        }
    }
    /// Adds a new note to the sequence
    pub fn new_note(
        &mut self,
        frequency: f64,
        duration: f64,
        on_velocity: f64,
        off_velocity: f64,
        instrument_id: usize,
    ) {
        let frequency_id = match &mut self.frequency_lut_builder {
            Some(c) => match c.iter().position(|&x| (x - frequency).abs() < EPSILON) {
                Some(i) => i,
                None => {
                    c.push(frequency);
                    c.len() - 1
                }
            },
            None => panic!("Deserved for not using the correct function !"),
        };
        self.new_note_with_flut(
            frequency_id,
            duration,
            on_velocity,
            off_velocity,
            instrument_id,
        );
    }
    /// Adds a new note to the sequence with known Frequency ID
    pub fn new_note_with_flut(
        &mut self,
        frequency_id: usize,
        duration: f64,
        on_velocity: f64,
        off_velocity: f64,
        instrument_id: usize,
    ) {
        self.sequence.add_note(Note {
            start_at: self.at_time,
            end_at: self.at_time + duration,
            duration,
            frequency_id,
            on_velocity,
            off_velocity,
            instrument_id,
        });
    }
    /// Returns the built sequence
    pub fn get_sequence(&self) -> Sequence {
        self.sequence.clone()
    }
    /// Returns the built FrequencyLookupTable
    pub fn get_frequency_lut(&self) -> FrequencyLookupTable {
        match self.frequency_lut {
            Some(ref f) => f.clone(),
            None => match self.frequency_lut_builder {
                Some(ref fc) => {
                    let mut lut = HashMap::new();
                    for (index, value) in fc.iter().enumerate() {
                        lut.insert(index, value.clone());
                    }
                    FrequencyLookupTable { lut }
                }
                None => panic!("Deserved for not using the correct function !"),
            },
        }
    }
}