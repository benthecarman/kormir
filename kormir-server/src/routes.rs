use crate::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use bitcoin::XOnlyPublicKey;
use kormir::storage::OracleEventData;

pub async fn health_check() -> Result<Json<()>, (StatusCode, String)> {
    Ok(Json(()))
}

pub async fn get_pubkey(
    Extension(state): Extension<State>,
) -> Result<Json<XOnlyPublicKey>, (StatusCode, String)> {
    Ok(Json(state.oracle.public_key()))
}

pub async fn list_events(
    Extension(state): Extension<State>,
) -> Result<Json<Vec<OracleEventData>>, (StatusCode, String)> {
    let events = state.oracle.storage.list_events().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to list events".to_string(),
        )
    })?;
    Ok(Json(events))
}
