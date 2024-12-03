use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::XOnlyPublicKey;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::event_nonces;

#[derive(
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EventNonce {
    pub id: i32,
    pub index: i32,
    nonce: Vec<u8>,
    pub signature: Option<Vec<u8>>,
    pub outcome: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    pub event_id: Option<String>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = event_nonces)]
pub struct NewEventNonce {
    pub id: i32,
    pub event_id: String,
    pub index: i32,
    pub nonce: Vec<u8>,
}

impl EventNonce {
    pub fn nonce(&self) -> XOnlyPublicKey {
        XOnlyPublicKey::from_slice(&self.nonce).expect("invalid nonce")
    }

    pub fn signature(&self) -> Option<Signature> {
        self.signature
            .as_ref()
            .map(|sig| Signature::from_slice(sig).expect("invalid signature"))
    }

    pub fn outcome_and_sig(&self) -> Option<(String, Signature)> {
        match (self.outcome.clone(), self.signature()) {
            (Some(outcome), Some(sig)) => Some((outcome, sig)),
            _ => None,
        }
    }

    pub fn get_next_id(conn: &mut PgConnection) -> anyhow::Result<i32> {
        let num = event_nonces::table
            .select(diesel::dsl::max(event_nonces::id))
            .first::<Option<i32>>(conn)?
            .map(|id| id + 1)
            .unwrap_or(0);
        Ok(num)
    }

    pub fn get_by_id(conn: &mut PgConnection, id: i32) -> anyhow::Result<Option<Self>> {
        Ok(event_nonces::table
            .find(id)
            .first::<Self>(conn)
            .optional()?)
    }

    pub fn get_by_event_id(conn: &mut PgConnection, event_id: String) -> anyhow::Result<Vec<Self>> {
        Ok(event_nonces::table
            .filter(event_nonces::event_id.eq(event_id))
            .order_by(event_nonces::index.asc())
            .get_results(conn)?)
    }
}
