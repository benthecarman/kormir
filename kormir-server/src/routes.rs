use crate::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use bitcoin::XOnlyPublicKey;

pub async fn health_check() -> Result<Json<()>, (StatusCode, String)> {
    Ok(Json(()))
}

pub async fn get_pubkey(
    Extension(state): Extension<State>,
) -> Result<Json<XOnlyPublicKey>, (StatusCode, String)> {
    Ok(Json(state.oracle.public_key()))
}
