use axum::http::{StatusCode, Uri};
use axum::routing::get;
use axum::{Extension, Router};
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel_migrations::MigrationHarness;
use kormir::Oracle;
use std::str::FromStr;

use crate::models::oracle_metadata::OracleMetadata;
use crate::models::{PostgresStorage, MIGRATIONS};
use crate::routes::*;

mod models;
mod routes;

#[derive(Clone)]
pub struct State {
    oracle: Oracle<PostgresStorage>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();
    pretty_env_logger::try_init()?;

    // get values key from env
    let pg_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port: u16 = std::env::var("KORMIR_PORT")
        .ok()
        .map(|p| p.parse::<u16>())
        .transpose()?
        .unwrap_or(8080);

    // DB management
    let manager = ConnectionManager::<PgConnection>::new(&pg_url);
    let db_pool = Pool::builder()
        .max_size(10)
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool");

    // run migrations
    let mut conn = db_pool.get()?;
    conn.run_pending_migrations(MIGRATIONS)
        .expect("migrations could not run");

    let secp = Secp256k1::new();
    let signing_key =
        SecretKey::from_str(&std::env::var("KORMIR_KEY").expect("KORMIR_KEY must be set"))?;

    let pubkey = signing_key.x_only_public_key(&secp).0;

    // check oracle metadata, if it doesn't exist, create it
    let metadata = OracleMetadata::get(&mut conn)?;
    match metadata {
        Some(metadata) => {
            if metadata.pubkey() != pubkey {
                anyhow::bail!(
                    "Database's oracle pubkey ({}) does not match signing key ({})",
                    metadata.pubkey().to_hex(),
                    pubkey.to_hex()
                );
            }
        }
        None => {
            OracleMetadata::upsert(&mut conn, pubkey)?;
        }
    }

    // for nonce_xpriv we just hash the key and use that as the seed
    let nonce_xpriv = {
        let bytes = sha256::Hash::hash(&signing_key.secret_bytes()).into_inner();
        ExtendedPrivKey::new_master(Network::Bitcoin, &bytes)?
    };
    let oracle = Oracle::new(
        PostgresStorage::new(db_pool, signing_key.x_only_public_key(&secp).0)?,
        signing_key,
        nonce_xpriv,
    );

    let state = State { oracle };

    let addr: std::net::SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .expect("Failed to parse bind/port for webserver");

    let server_router = Router::new()
        .route("/health-check", get(health_check))
        .route("/pubkey", get(get_pubkey))
        .fallback(fallback)
        .layer(Extension(state));

    let server = axum::Server::bind(&addr).serve(server_router.into_make_service());

    println!("Webserver running on http://{addr}");

    let graceful = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to create Ctrl+C shutdown signal");
    });

    // Await the server to receive the shutdown signal
    if let Err(e) = graceful.await {
        eprintln!("shutdown error: {e}");
    }

    Ok(())
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {uri}"))
}
