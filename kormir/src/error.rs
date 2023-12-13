/// Kormir error type
#[derive(Debug, Clone)]
pub enum Error {
    /// Attempted to sign an event that was already signed
    EventAlreadySigned,
    /// Event data was not found
    NotFound,
    /// The storage failed to read/save the data
    StorageFailure,
    /// User gave an invalid outcome
    InvalidOutcome,
    /// An error that should never happen, if it does it's a bug
    Internal,
}
