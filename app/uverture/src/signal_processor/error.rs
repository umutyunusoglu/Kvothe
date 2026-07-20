use std::fmt;

#[derive(Debug)]
pub enum SignalError {
    InvalidSampleRate,
    InvalidNumberOfSamples,
    ProcessorLockError,
    EmptySpectrum,
    EmptySignal,
}

impl std::error::Error for SignalError {}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SignalError::InvalidSampleRate => write!(f, "Sample rate must be positive"),
            SignalError::InvalidNumberOfSamples => write!(f, "FFT size must be positive"),
            SignalError::ProcessorLockError => write!(f, "Failed to acquire processor lock"),
            SignalError::EmptySpectrum => write!(f, "Spectrum is empty"),
            SignalError::EmptySignal => write!(f, "Signal is empty"),
        }
    }
}
