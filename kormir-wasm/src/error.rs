use kormir::error::Error;
use thiserror::Error;
use wasm_bindgen::prelude::wasm_bindgen;

/// Kormir error type
#[derive(Error, Debug, Clone)]
#[wasm_bindgen]
pub enum JsError {
    /// Invalid argument given
    #[error("Invalid argument given")]
    InvalidArgument,
    /// Attempted to sign an event that was already signed
    #[error("Attempted to sign an event that was already signed")]
    EventAlreadySigned,
    /// Event data was not found
    #[error("Event data was not found")]
    NotFound,
    /// The storage failed to read/save the data
    #[error("Storage failed to read/save the data")]
    StorageFailure,
    /// User gave an invalid outcome
    #[error("User gave an invalid outcome")]
    InvalidOutcome,
    /// An error that should never happen, if it does it's a bug
    #[error("Internal Error")]
    Internal,
    /// An error with creating or sending Nostr events
    #[error("Error sending nostr events")]
    Nostr,
}

impl From<Error> for JsError {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidArgument => Self::InvalidArgument,
            Error::EventAlreadySigned => Self::EventAlreadySigned,
            Error::NotFound => Self::NotFound,
            Error::StorageFailure => Self::StorageFailure,
            Error::InvalidOutcome => Self::InvalidOutcome,
            Error::Internal => Self::Internal,
        }
    }
}

impl From<JsError> for Error {
    fn from(value: JsError) -> Self {
        match value {
            JsError::InvalidArgument => Self::InvalidArgument,
            JsError::EventAlreadySigned => Self::EventAlreadySigned,
            JsError::NotFound => Self::NotFound,
            JsError::StorageFailure => Self::StorageFailure,
            JsError::InvalidOutcome => Self::InvalidOutcome,
            JsError::Internal => Self::Internal,
            JsError::Nostr => Self::Internal,
        }
    }
}

impl From<rexie::Error> for JsError {
    fn from(_: rexie::Error) -> Self {
        JsError::StorageFailure
    }
}

impl From<hex::FromHexError> for JsError {
    fn from(_: hex::FromHexError) -> Self {
        JsError::InvalidArgument
    }
}

impl From<kormir::bitcoin::secp256k1::Error> for JsError {
    fn from(_: kormir::bitcoin::secp256k1::Error) -> Self {
        JsError::StorageFailure
    }
}

impl From<kormir::lightning::ln::msgs::DecodeError> for JsError {
    fn from(_: kormir::lightning::ln::msgs::DecodeError) -> Self {
        JsError::InvalidArgument
    }
}

impl From<serde_json::Error> for JsError {
    fn from(_: serde_json::Error) -> Self {
        JsError::StorageFailure
    }
}

impl From<nostr::event::builder::Error> for JsError {
    fn from(_: nostr::event::builder::Error) -> Self {
        JsError::NotFound
    }
}

impl From<nostr_sdk::client::Error> for JsError {
    fn from(_: nostr_sdk::client::Error) -> Self {
        JsError::Nostr
    }
}

impl From<kormir::bitcoin::util::bip32::Error> for JsError {
    fn from(_: kormir::bitcoin::util::bip32::Error) -> Self {
        JsError::Internal
    }
}

impl From<bip39::Error> for JsError {
    fn from(_: bip39::Error) -> Self {
        JsError::Internal
    }
}
