pub mod storage;

use crate::storage::Storage;
use bitcoin::hashes::sha256;
use bitcoin::secp256k1::{All, Message, Secp256k1, SecretKey};
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
use bitcoin::util::key::KeyPair;
use bitcoin::XOnlyPublicKey;
use dlc_messages::oracle_msgs::{
    EnumEventDescriptor, EventDescriptor, OracleAnnouncement, OracleEvent,
};
use lightning::util::ser::Writeable;
use std::str::FromStr;

// first key for taproot address
const SIGNING_KEY_PATH: &str = "m/86'/0'/0'/0/0";

const NONCE_KEY_PATH: &str = "m/585'/0'/0'";

#[derive(Debug, Clone)]
pub struct Oracle<S: Storage> {
    pub storage: S,
    signing_key: SecretKey,
    nonce_xpriv: ExtendedPrivKey,
    secp: Secp256k1<All>,
}

impl<S: Storage> Oracle<S> {
    pub fn new(storage: S, signing_key: SecretKey, nonce_xpriv: ExtendedPrivKey) -> Self {
        let secp = Secp256k1::new();
        Self {
            storage,
            signing_key,
            nonce_xpriv,
            secp,
        }
    }

    pub fn from_xpriv(storage: S, xpriv: ExtendedPrivKey) -> anyhow::Result<Self> {
        let secp = Secp256k1::new();

        let signing_key = xpriv
            .derive_priv(&secp, &DerivationPath::from_str(SIGNING_KEY_PATH)?)?
            .private_key;
        let nonce_xpriv = xpriv.derive_priv(&secp, &DerivationPath::from_str(NONCE_KEY_PATH)?)?;

        Ok(Self {
            storage,
            signing_key,
            nonce_xpriv,
            secp,
        })
    }

    pub fn public_key(&self) -> XOnlyPublicKey {
        self.signing_key.x_only_public_key(&self.secp).0
    }

    pub fn create_enum_event(
        &self,
        event_id: String,
        outcomes: Vec<String>,
        event_maturity_epoch: u32,
    ) -> anyhow::Result<OracleAnnouncement> {
        let indexes = self.storage.get_next_nonce_indexes(1)?;
        let oracle_nonces = indexes
            .iter()
            .map(|i| {
                let nonce_key = self
                    .nonce_xpriv
                    .derive_priv(&self.secp, &[ChildNumber::from_hardened_idx(*i).unwrap()])
                    .unwrap();
                nonce_key.private_key.x_only_public_key(&self.secp).0
            })
            .collect();
        let event_descriptor = EventDescriptor::EnumEvent(EnumEventDescriptor { outcomes });
        let oracle_event = OracleEvent {
            oracle_nonces,
            event_id,
            event_maturity_epoch,
            event_descriptor,
        };
        oracle_event
            .validate()
            .map_err(|_| anyhow::anyhow!("Created invalid event"))?;

        // create signature
        let mut data = Vec::new();
        oracle_event.write(&mut data)?;
        let msg = Message::from_hashed_data::<sha256::Hash>(&data);
        let announcement_signature = self.secp.sign_schnorr_no_aux_rand(
            &msg,
            &KeyPair::from_secret_key(&self.secp, &self.signing_key),
        );

        let ann = OracleAnnouncement {
            oracle_event,
            oracle_public_key: self.public_key(),
            announcement_signature,
        };
        ann.validate(&self.secp)
            .map_err(|_| anyhow::anyhow!("Created invalid announcement"))?;

        self.storage.save_announcement(&ann, indexes)?;

        Ok(ann)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bitcoin::Network;

    fn create_oracle() -> Oracle<()> {
        let xpriv = ExtendedPrivKey::new_master(Network::Regtest, &[1; 32]).unwrap();
        Oracle::from_xpriv((), xpriv).unwrap()
    }

    #[test]
    fn test_create_enum_event() {
        let oracle = create_oracle();

        let event_id = "test".to_string();
        let outcomes = vec!["a".to_string(), "b".to_string()];
        let event_maturity_epoch = 100;
        let ann = oracle
            .create_enum_event(event_id.clone(), outcomes.clone(), event_maturity_epoch)
            .unwrap();

        assert!(ann.validate(&oracle.secp).is_ok());
        assert_eq!(ann.oracle_event.event_id, event_id);
        assert_eq!(ann.oracle_event.event_maturity_epoch, event_maturity_epoch);
        assert_eq!(
            ann.oracle_event.event_descriptor,
            EventDescriptor::EnumEvent(EnumEventDescriptor { outcomes })
        );
    }
}
