use gloo_utils::format::JsValueSerdeExt;
use kormir::bitcoin::hashes::hex::ToHex;
use kormir::storage::OracleEventData;
use kormir::{EventDescriptor, OracleAnnouncement, OracleAttestation, Writeable};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    announcement_signature: String,
    oracle_public_key: String,
    oracle_nonces: Vec<String>,
    pub event_maturity_epoch: u32,
    outcomes: Vec<String>,
    event_id: String,
}

#[wasm_bindgen]
impl Announcement {
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> JsValue {
        JsValue::from_serde(&serde_json::to_value(self).unwrap()).unwrap()
    }

    #[wasm_bindgen(getter)]
    pub fn announcement_signature(&self) -> String {
        self.announcement_signature.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn oracle_public_key(&self) -> String {
        self.oracle_public_key.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn oracle_nonces(&self) -> Vec<String> {
        self.oracle_nonces.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn outcomes(&self) -> Vec<String> {
        self.outcomes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn event_id(&self) -> String {
        self.event_id.clone()
    }
}

impl From<OracleAnnouncement> for Announcement {
    fn from(value: OracleAnnouncement) -> Self {
        let outcomes = match value.oracle_event.event_descriptor {
            EventDescriptor::EnumEvent(e) => e.outcomes,
            EventDescriptor::DigitDecompositionEvent(_) => {
                unimplemented!("Numeric events not supported")
            }
        };

        Self {
            announcement_signature: value.announcement_signature.to_hex(),
            oracle_public_key: value.announcement_signature.to_hex(),
            oracle_nonces: value
                .oracle_event
                .oracle_nonces
                .iter()
                .map(|x| x.to_hex())
                .collect(),
            event_maturity_epoch: value.oracle_event.event_maturity_epoch,
            outcomes,
            event_id: value.oracle_event.event_id,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    oracle_public_key: String,
    outcomes: Vec<String>,
    signatures: Vec<String>,
}

#[wasm_bindgen]
impl Attestation {
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> JsValue {
        JsValue::from_serde(&serde_json::to_value(self).unwrap()).unwrap()
    }

    #[wasm_bindgen(getter)]
    pub fn oracle_public_key(&self) -> String {
        self.oracle_public_key.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn outcomes(&self) -> Vec<String> {
        self.outcomes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn signatures(&self) -> Vec<String> {
        self.signatures.clone()
    }
}

impl From<OracleAttestation> for Attestation {
    fn from(value: OracleAttestation) -> Self {
        Self {
            oracle_public_key: value.oracle_public_key.to_hex(),
            signatures: value.signatures.iter().map(|x| x.to_hex()).collect(),
            outcomes: value.outcomes,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    id: u32,
    announcement: String,
    attestation: Option<String>,
    pub event_maturity_epoch: u32,
    outcomes: Vec<String>,
    event_name: String,
    announcement_event_id: Option<String>,
    attestation_event_id: Option<String>,
    observed_outcome: Option<String>,
}

#[wasm_bindgen]
impl EventData {
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> JsValue {
        JsValue::from_serde(&serde_json::to_value(self).unwrap()).unwrap()
    }

    #[wasm_bindgen(getter)]
    pub fn announcement(&self) -> String {
        self.announcement.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn attestation(&self) -> Option<String> {
        self.attestation.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn outcomes(&self) -> Vec<String> {
        self.outcomes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn event_name(&self) -> String {
        self.event_name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn announcement_event_id(&self) -> Option<String> {
        self.announcement_event_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn attestation_event_id(&self) -> Option<String> {
        self.attestation_event_id.clone()
    }
}

impl From<(u32, OracleEventData)> for EventData {
    fn from((id, value): (u32, OracleEventData)) -> Self {
        let outcomes = match &value.announcement.oracle_event.event_descriptor {
            EventDescriptor::EnumEvent(e) => e.outcomes.clone(),
            EventDescriptor::DigitDecompositionEvent(_) => {
                unimplemented!("Numeric events not supported")
            }
        };

        let (attestation, observed_outcome) = match value.signatures.len() {
            0 => (None, None),
            _ => {
                // todo proper sorting for non-enum events
                let attestation = OracleAttestation {
                    oracle_public_key: value.announcement.oracle_public_key,
                    signatures: value.signatures.values().cloned().collect(),
                    outcomes: value.signatures.keys().cloned().collect(),
                };
                let attestation = attestation.encode().to_hex();
                let outcome = value.signatures.keys().next().cloned().unwrap();
                (Some(attestation), Some(outcome))
            }
        };

        EventData {
            id,
            announcement: value.announcement.encode().to_hex(),
            attestation,
            event_maturity_epoch: value.announcement.oracle_event.event_maturity_epoch,
            outcomes,
            event_name: value.announcement.oracle_event.event_id,
            announcement_event_id: value.announcement_event_id,
            attestation_event_id: value.attestation_event_id,
            observed_outcome,
        }
    }
}
