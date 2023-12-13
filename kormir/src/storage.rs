use crate::error::Error;
use async_trait::async_trait;
use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::schnorr::Signature;
use dlc_messages::oracle_msgs::OracleAnnouncement;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Storage {
    /// Get the next `num` nonce indexes
    async fn get_next_nonce_indexes(&self, num: usize) -> Result<Vec<u32>, Error>;

    /// Save the announcement and return the identifier
    /// for the announcement
    async fn save_announcement(
        &self,
        announcement: OracleAnnouncement,
        indexes: Vec<u32>,
    ) -> Result<u32, Error>;

    /// Save signatures for a given event
    async fn save_signatures(
        &self,
        id: u32,
        sigs: Vec<Signature>,
    ) -> Result<OracleEventData, Error>;

    /// Get the announcement data for the given id
    async fn get_event(&self, id: u32) -> Result<Option<OracleEventData>, Error>;
}

/// Data saved for an oracle announcement
#[derive(Debug, Clone)]
pub struct OracleEventData {
    pub announcement: OracleAnnouncement,
    pub indexes: Vec<u32>,
    pub signatures: Vec<Signature>,
}

#[derive(Debug, Clone)]
pub struct MemoryStorage {
    current_index: Arc<AtomicU32>,
    data: Arc<RwLock<HashMap<u32, OracleEventData>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            current_index: Arc::new(AtomicU32::new(0)),
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Storage for MemoryStorage {
    async fn get_next_nonce_indexes(&self, num: usize) -> Result<Vec<u32>, Error> {
        let mut current_index = self.current_index.fetch_add(num as u32, Ordering::Relaxed);
        let mut indexes = Vec::with_capacity(num);
        for _ in 0..num {
            indexes.push(current_index);
            current_index += 1;
        }
        Ok(indexes)
    }

    async fn save_announcement(
        &self,
        announcement: OracleAnnouncement,
        indexes: Vec<u32>,
    ) -> Result<u32, Error> {
        // generate random id
        let id = rand::random::<u32>();
        let event = OracleEventData {
            announcement,
            indexes,
            signatures: Vec::new(),
        };

        let mut data = self.data.try_write().unwrap();
        data.insert(id, event);

        Ok(id)
    }

    async fn save_signatures(
        &self,
        id: u32,
        sigs: Vec<Signature>,
    ) -> Result<OracleEventData, Error> {
        let mut data = self.data.try_write().unwrap();
        let Some(mut event) = data.get(&id).cloned() else {
            return Err(Error::NotFound);
        };

        if !event.signatures.is_empty() {
            return Err(Error::EventAlreadySigned);
        }

        event.signatures = sigs;
        data.insert(id, event.clone());

        Ok(event)
    }

    async fn get_event(&self, id: u32) -> Result<Option<OracleEventData>, Error> {
        let data = self.data.try_read().unwrap();
        Ok(data.get(&id).cloned())
    }
}
