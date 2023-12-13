use crate::error::JsError;
use crate::storage::{IndexedDb, MNEMONIC_KEY};
use bip39::Mnemonic;
use kormir::bitcoin::hashes::hex::ToHex;
use kormir::bitcoin::util::bip32::ExtendedPrivKey;
use kormir::bitcoin::Network;
use kormir::storage::Storage;
use kormir::{Oracle, Writeable};
use nostr::EventId;
use nostr_sdk::Client;
use wasm_bindgen::prelude::wasm_bindgen;

mod error;
mod storage;

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

        let event = kormir::nostr_events::create_announcement_event(
            &self.oracle.nostr_keys(),
            &ann,
            &self.relays,
        )?;

        self.storage
            .add_announcement_event_id(id, event.id.to_hex())
            .await?;

        self.client.send_event(event).await?;

        Ok(ann.encode().to_hex())
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
}
