use pcm::error::PCMError;
use std::error::Error;
use std::fmt::{Display, Formatter, Result};

/// The main error type. Everything in this library that returns an error will return this type.
#[derive(Debug)]
pub enum SequencerError {
    /// An error originating from the PCM Library
    PCMError(PCMError),
    /// If no key is available and no custom KeyGenerator is provided
    NoDefaultKeyGiven,
    /// If a float given to use as a TIme or a Frequency is not a normal number and strictly superior to zero
    ImpossibleTimeOrFrequency(f64),
    /// If there is no frequency associated with an ID in a FrequencyLookupTable
    NoFrequencyForID(usize),
    /// If there is no instrument associated with an ID in a InstrumentTable
    NoInstrumentForID(usize),
    /// IF there is no key associated with an ID for an Instrument
    NoKeyForID(usize),
}

impl Error for SequencerError {
    fn description(&self) -> &str {
        match self {
            SequencerError::PCMError(e) => e.description(),
            SequencerError::NoDefaultKeyGiven => "No KeyGenerator and no default key to change the pitch of",
            SequencerError::ImpossibleTimeOrFrequency(_) => "An impossible value for a Frequency or a Time was tried to be used or put in a FrequencyLookupTable",
            SequencerError::NoFrequencyForID(_) => "There is no frequency in the FrequencyLookupTable associated with this ID",
            SequencerError::NoInstrumentForID(_) => "There is no instrument in the InstrumentLookingTable associated with this ID",
            SequencerError::NoKeyForID(_) => "There is no Key in the Instrument associated with this ID"
        }
    }
}

impl Display for SequencerError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            SequencerError::PCMError(e) => e.fmt(f),
            SequencerError::NoDefaultKeyGiven => {
                write!(f, "No key in vec, impossible to crate new keys")
            }
            SequencerError::ImpossibleTimeOrFrequency(v) => write!(f, "Impossible value: {}", v),
            SequencerError::NoFrequencyForID(id) => write!(f, "Unassigned Frequency ID: {}", id),
            SequencerError::NoInstrumentForID(id) => write!(f, "Unassigned Instrument ID: {}", id),
            SequencerError::NoKeyForID(id) => write!(f, "Unassigned Key ID: {}", id),
        }
    }
}

impl From<PCMError> for SequencerError {
    fn from(e: PCMError) -> SequencerError {
        SequencerError::PCMError(e)
    }
}
