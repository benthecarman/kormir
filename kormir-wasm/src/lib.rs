use crate::error::JsError;
use crate::storage::{IndexedDb, MNEMONIC_KEY};
use bip39::Mnemonic;
use gloo_utils::format::JsValueSerdeExt;
use kormir::bitcoin::hashes::hex::ToHex;
use kormir::bitcoin::util::bip32::ExtendedPrivKey;
use kormir::bitcoin::Network;
use kormir::storage::{OracleEventData, Storage};
use kormir::{EventDescriptor, Oracle, OracleAttestation, Writeable};
use nostr::{EventId, JsonUtil};
use nostr_sdk::Client;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

mod error;
mod storage;
mod utils;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct Kormir {
    oracle: Oracle<IndexedDb>,
    storage: IndexedDb,
    client: Client,
    relays: Vec<String>,
}

#[wasm_bindgen]
impl Kormir {
    pub async fn new(relays: Vec<String>) -> Result<Kormir, JsError> {
        utils::set_panic_hook();
        let storage = IndexedDb::new().await?;

        let mnemonic: Option<Mnemonic> = storage.get_from_indexed_db(MNEMONIC_KEY).await?;
        let xpriv = match mnemonic {
            Some(mnemonic) => ExtendedPrivKey::new_master(Network::Bitcoin, &mnemonic.to_seed(""))?,
            None => {
                let mut entropy: [u8; 16] = [0; 16];
                getrandom::getrandom(&mut entropy).unwrap();

                let mnemonic = Mnemonic::from_entropy(&entropy)?;
                storage.save_to_indexed_db(MNEMONIC_KEY, &mnemonic).await?;
                ExtendedPrivKey::new_master(Network::Bitcoin, &mnemonic.to_seed(""))?
            }
        };

        let oracle = Oracle::from_xpriv(storage.clone(), xpriv)?;

        let client = Client::new(&oracle.nostr_keys());

        for relay in relays.iter() {
            #[cfg(target_arch = "wasm32")]
            client.add_relay(relay.as_str()).await?;

            #[cfg(not(target_arch = "wasm32"))]
            client.add_relay(relay.as_str(), None).await?;
        }

        client.connect().await;

        Ok(Kormir {
            oracle,
            storage,
            client,
            relays,
        })
    }

    pub fn get_public_key(&self) -> String {
        self.oracle.public_key().to_hex()
    }

    pub async fn create_enum_event(
        &self,
        event_id: String,
        outcomes: Vec<String>,
        event_maturity_epoch: u32,
    ) -> Result<String, JsError> {
        let (id, ann) = self
            .oracle
            .create_enum_event(event_id, outcomes, event_maturity_epoch)
            .await?;

        let hex = ann.encode().to_hex();

        log::info!("Created enum event: {hex}");

        let event = kormir::nostr_events::create_announcement_event(
            &self.oracle.nostr_keys(),
            &ann,
            &self.relays,
        )?;

        log::debug!("Created nostr event: {}", event.as_json());

        self.storage
            .add_announcement_event_id(id, event.id.to_hex())
            .await?;

        log::debug!(
            "Added announcement event id to storage: {}",
            event.id.to_hex()
        );

        self.client.send_event(event).await?;

        log::trace!("Sent event to nostr");

        Ok(hex)
    }

    pub async fn sign_enum_event(&self, id: u32, outcome: String) -> Result<String, JsError> {
        let attestation = self.oracle.sign_enum_event(id, outcome).await?;

        let event = self.storage.get_event(id).await?.ok_or(JsError::NotFound)?;
        let event_id = EventId::from_hex(event.announcement_event_id.unwrap()).unwrap();

        let event = kormir::nostr_events::create_attestation_event(
            &self.oracle.nostr_keys(),
            &attestation,
            event_id,
        )?;

        self.storage
            .add_attestation_event_id(id, event.id.to_hex())
            .await?;

        self.client.send_event(event).await?;

        Ok(attestation.encode().to_hex())
    }

    pub async fn list_events(&self) -> Result<JsValue /* Vec<EventData> */, JsError> {
        let data = self.storage.list_events().await?;
        let events = data.into_iter().map(EventData::from).collect::<Vec<_>>();

        Ok(JsValue::from_serde(&events)?)
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    announcement: String,
    attestation: Option<String>,
    pub event_maturity_epoch: u32,
    outcomes: Vec<String>,
    event_id: String,
    announcement_event_id: Option<String>,
    attestation_event_id: Option<String>,
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
    pub fn event_id(&self) -> String {
        self.event_id.clone()
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

impl From<OracleEventData> for EventData {
    fn from(value: OracleEventData) -> Self {
        let outcomes = match &value.announcement.oracle_event.event_descriptor {
            EventDescriptor::EnumEvent(e) => e.outcomes.clone(),
            EventDescriptor::DigitDecompositionEvent(_) => unimplemented!(),
        };

        let attestation = match value.signatures.len() {
            0 => None,
            _ => {
                // todo proper sorting for non-enum events
                let attestation = OracleAttestation {
                    oracle_public_key: value.announcement.oracle_public_key,
                    signatures: value.signatures.values().cloned().collect(),
                    outcomes: value.signatures.keys().cloned().collect(),
                };
                Some(attestation.encode().to_hex())
            }
        };

        EventData {
            announcement: value.announcement.encode().to_hex(),
            attestation,
            event_maturity_epoch: value.announcement.oracle_event.event_maturity_epoch,
            outcomes,
            event_id: value.announcement.oracle_event.event_id,
            announcement_event_id: value.announcement_event_id,
            attestation_event_id: value.attestation_event_id,
        }
    }
}
