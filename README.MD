# docker_plugin
This crate allows you to create plugins for docker swarm. 
It implements the scaffolding for Docker's [plugin api](https://docs.docker.com/engine/extend/plugin_api/).

## Supported drivers

### Example
```rust
use std::sync::Arc;
use docker_plugin::volume::{CapabilitiesResponse, CreateRequest, GetRequest, GetResponse, ListResponse, MountRequest, MountResponse, PathRequest, PathResponse, RemoveRequest, UnmountRequest, Volume};
use hyperlocal::UnixServerExt;

pub struct Movable {}

#[async_trait::async_trait]
impl docker_plugin::volume::Driver for Movable {
    type Opts = ();
    type Status = ();

    async fn create(&self, req: CreateRequest<Self::Opts>) -> anyhow::Result<()> {
        todo!()
    }

    async fn list(&self) -> anyhow::Result<ListResponse<Self::Status>> {
        todo!()
    }

    async fn get(&self, req: GetRequest) -> anyhow::Result<GetResponse<Self::Status>> {
        todo!()
    }

    async fn remove(&self, req: RemoveRequest) -> anyhow::Result<()> {
        todo!()
    }

    async fn path(&self, req: PathRequest) -> anyhow::Result<PathResponse> {
        todo!()
    }

    async fn mount(&self, req: MountRequest) -> anyhow::Result<MountResponse> {
        todo!()
    }

    async fn unmount(&self, req: UnmountRequest) -> anyhow::Result<()> {
        todo!()
    }

    async fn capabilities(&self) -> CapabilitiesResponse {
        todo!()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let driver = Movable {};
    let driver = Arc::new(driver);

    let app = docker_plugin::router(vec![
        docker_plugin::volume::IMPLEMENTS_VOLUME.to_string()
    ]);

    let app = app.merge(docker_plugin::volume::router(driver));

    let _ = std::fs::remove_file("/run/docker/plugins/movable.sock");

    axum::Server::bind_unix("/run/docker/plugins/movable.sock")
        .expect("Couldn't bind unix socket")
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Can't start the server");
    
    let _ = std::fs::remove_file("/run/docker/plugins/movable.sock");
}
```