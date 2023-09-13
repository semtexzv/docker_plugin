use std::ops::Deref;
use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::routing::post;
use serde::{Deserialize, Serialize};

pub mod volume;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    implements: Vec<String>,
}

async fn activate(s: State<Arc<Vec<String>>>) -> Json<Manifest> {
    Json(Manifest {
        implements: s.0.deref().clone()
    })
}

pub fn router(implements: Vec<String>) -> Router {
    Router::new()
        .route("/Plugin.Activate", post(activate))
        .with_state(Arc::new(implements))
}
