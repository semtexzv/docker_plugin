use anyhow::Result;
use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::json;

pub const IMPLEMENTS_VOLUME: &str = "VolumeDriver";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateRequest<Opts> {
    pub name: String,
    pub options: Opts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RemoveRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MountRequest {
    pub name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MountResponse {
    pub mountpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnmountRequest {
    pub name: String,
    #[serde(rename = "ID")]
    pub id: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PathRequest {
    pub name: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PathResponse {
    pub mountpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetResponse<Status> {
    pub volume: Option<Volume<Status>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CapabilitiesResponse {
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Capabilities {
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListResponse<Status> {
    pub volumes: Vec<Volume<Status>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Volume<Status> {
    pub name: String,
    pub mountpoint: String,
    pub created_at: String,
    pub status: Status,
}

#[async_trait::async_trait]
pub trait Driver: Send + Sync + 'static {
    type Opts: Send + Serialize + DeserializeOwned;
    type Status: Serialize + DeserializeOwned;

    async fn create(&self, req: CreateRequest<Self::Opts>) -> Result<()>;
    async fn list(&self) -> Result<ListResponse<Self::Status>>;
    async fn get(&self, req: GetRequest) -> Result<GetResponse<Self::Status>>;
    async fn remove(&self, req: RemoveRequest) -> Result<()>;
    async fn path(&self, req: PathRequest) -> Result<PathResponse>;
    async fn mount(&self, req: MountRequest) -> Result<MountResponse>;
    async fn unmount(&self, req: UnmountRequest) -> Result<()>;

    async fn capabilities(&self) -> CapabilitiesResponse;
}

// Make our own error that wraps `anyhow::Error`.
pub struct DriverError(anyhow::Error);

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for DriverError
    where
        E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}


// Tell axum how to convert `DriverError` into a response.
impl IntoResponse for DriverError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"Err":format!("Something went wrong: {}", self.0)}))
        )
            .into_response()
    }
}

async fn create<D: Driver>(d: State<Arc<D>>, req: Json<CreateRequest<D::Opts>>) -> Result<(), DriverError> {
    Ok(d.create(req.0).await?)
}

async fn get<D: Driver>(d: State<Arc<D>>, req: Json<GetRequest>) -> Result<Json<GetResponse<D::Status>>, DriverError> {
    Ok(Json(d.get(req.0).await?))
}

async fn list<D: Driver>(d: State<Arc<D>>) -> Result<Json<ListResponse<D::Status>>, DriverError> {
    Ok(Json(d.list().await?))
}

async fn remove<D: Driver>(d: State<Arc<D>>, req: Json<RemoveRequest>) -> Result<(), DriverError> {
    Ok(d.remove(req.0).await?)
}


async fn path<D: Driver>(d: State<Arc<D>>, req: Json<PathRequest>) -> Result<Json<PathResponse>, DriverError> {
    Ok(Json(d.path(req.0).await?))
}

async fn mount<D: Driver>(d: State<Arc<D>>, req: Json<MountRequest>) -> Result<Json<MountResponse>, DriverError> {
    Ok(Json(d.mount(req.0).await?))
}

async fn unmount<D: Driver>(d: State<Arc<D>>, req: Json<UnmountRequest>) -> Result<(), DriverError> {
    Ok(d.unmount(req.0).await?)
}

async fn capabilities<D: Driver>(d: State<Arc<D>>) -> Result<Json<CapabilitiesResponse>, DriverError> {
    Ok(Json(d.capabilities().await))
}

pub fn router<D: Driver>(d: Arc<D>) -> Router {
    Router::new()
        .route("/VolumeDriver.Create", post(create))
        .route("/VolumeDriver.Get", post(get))
        .route("/VolumeDriver.List", post(list))
        .route("/VolumeDriver.Remove", post(remove))
        .route("/VolumeDriver.Path", post(path))
        .route("/VolumeDriver.Mount", post(mount))
        .route("/VolumeDriver.Unmount", post(unmount))
        .route("/VolumeDriver.Capabilities", post(capabilities))
        .with_state(d)
}