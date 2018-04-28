extern crate pcm;

use pcm::{PCMParameters, PCM};
use std::collections::HashMap;

pub struct MusicSequencer {
    pub pcm_parameters: PCMParameters,
    pub sequence: Sequence,
    pub instruments: InstrumentList,
}

pub struct Sequence {
    pub notes: Vec<Note>,
}

pub struct Note {
    pub start_at: u32,
    pub end_at: u32,
    pub duration: u32,
    pub frequency: f64,
    pub on_velocity: u32,
    pub off_velocity: u32,
    pub instrument_id: u16,
}

pub struct InstrumentList {
    pub instruments: HashMap<u16, Instrument>,
}

pub struct Instrument {
    pub keys: HashMap<f64, Key>,
    pub key_gen: Box<KeyGenerator>,
    pub loopable: bool,
    pub envelope: Option<Box<Envelope>>,
}

pub struct Key {
    pub audio: PCM,
}

pub trait KeyGenerator {
    fn key_gen(&self, frequency: f64) -> PCM;
}

pub trait Envelope {
    fn before_sustain(&self, time: f64) -> f64;
    fn sustain(&self) -> f64;
    fn after_sustain(&self) -> f64;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
