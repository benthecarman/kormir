use bitcoin::key::XOnlyPublicKey;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::oracle_metadata;

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
#[diesel(primary_key(pubkey))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = oracle_metadata)]
pub struct OracleMetadata {
    pubkey: Vec<u8>,
    pub name: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    singleton_constant: bool,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = oracle_metadata)]
struct NewOracleMetadata<'a> {
    pubkey: Vec<u8>,
    name: &'a str,
}

impl OracleMetadata {
    pub fn pubkey(&self) -> XOnlyPublicKey {
        XOnlyPublicKey::from_slice(&self.pubkey).expect("invalid pubkey")
    }

    pub fn get(conn: &mut PgConnection) -> anyhow::Result<Option<OracleMetadata>> {
        Ok(oracle_metadata::table
            .filter(oracle_metadata::singleton_constant.eq(true))
            .first::<Self>(conn)
            .optional()?)
    }

    pub fn upsert(conn: &mut PgConnection, pubkey: XOnlyPublicKey) -> anyhow::Result<()> {
        let pubkey = pubkey.serialize().to_vec();
        let name = "Kormir";
        let new = NewOracleMetadata { pubkey, name };
        diesel::insert_into(oracle_metadata::table)
            .values(&new)
            .on_conflict(oracle_metadata::pubkey)
            .do_update()
            .set(&new)
            .execute(conn)?;
        Ok(())
    }
}
