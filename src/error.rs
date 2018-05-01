use pcm::error::PCMError;
use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum SequencerError {
    PCMError(PCMError),
    NoDefaultKeyGiven,
    ImpossibleTimeOrFrequency(f64),
    NoFrequencyForID(u32),
    NoInstrumentForID(u16),
}

impl Error for SequencerError {
    fn description(&self) -> &str {
        match self {
            SequencerError::PCMError(e) => e.description(),
            SequencerError::NoDefaultKeyGiven => "No KeyGenerator and no default key to change the pitch of",
            SequencerError::ImpossibleTimeOrFrequency(_) => "An impossible value for a Frequency or a Time was tried to be used or put in a FrequencyLookupTable",
            SequencerError::NoFrequencyForID(_) => "There is no frequency in the FrequencyLookupTable associated with this ID",
            SequencerError::NoInstrumentForID(_) => "There is no instrument in the InstrumentLookingTable associated with this ID"
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
        }
    }
}

impl From<PCMError> for SequencerError {
    fn from(e: PCMError) -> SequencerError {
        SequencerError::PCMError(e)
    }
}
