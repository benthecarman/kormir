use crate::models::oracle_metadata::OracleMetadata;
use crate::models::{PostgresStorage, MIGRATIONS};
use crate::routes::*;
use axum::http::{StatusCode, Uri};
use axum::routing::{get, post};
use axum::{Extension, Router};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel_migrations::MigrationHarness;
use kormir::Oracle;
use nostr::Keys;
use nostr_sdk::Client;

mod models;
mod routes;

#[derive(Clone)]
pub struct State {
    oracle: Oracle<PostgresStorage>,
    client: Client,
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
    let kormir_key = &std::env::var("KORMIR_KEY").expect("KORMIR_KEY must be set");
    let secret_bytes = Keys::parse(kormir_key)?.secret_key()?.secret_bytes();
    let signing_key = SecretKey::from_slice(&secret_bytes)?;

    let pubkey = signing_key.x_only_public_key(&secp).0;

    // check oracle metadata, if it doesn't exist, create it
    let metadata = OracleMetadata::get(&mut conn)?;
    match metadata {
        Some(metadata) => {
            if metadata.pubkey() != pubkey {
                anyhow::bail!(
                    "Database's oracle pubkey ({}) does not match signing key ({})",
                    hex::encode(metadata.pubkey().serialize()),
                    hex::encode(pubkey.serialize()),
                );
            }
        }
        None => {
            OracleMetadata::upsert(&mut conn, pubkey)?;
        }
    }

    let oracle = Oracle::from_signing_key(
        PostgresStorage::new(db_pool, signing_key.x_only_public_key(&secp).0)?,
        signing_key,
    )?;

    let relays = std::env::var("KORMIR_RELAYS")
        .unwrap_or("wss://relay.damus.io".to_string())
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let client = Client::new(oracle.nostr_keys());
    client.add_relays(relays).await?;
    client.connect().await;

    let state = State { oracle, client };

    let addr: std::net::SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .expect("Failed to parse bind/port for webserver");

    let server_router = Router::new()
        .route("/health-check", get(health_check))
        .route("/pubkey", get(get_pubkey))
        .route("/list-events", get(list_events))
        .route("/announcement/:event_id", get(get_oracle_announcement))
        .route("/attestation/:event_id", get(get_oracle_attestation))
        .route("/create-enum", post(create_enum_event))
        .route("/sign-enum", post(sign_enum_event))
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
