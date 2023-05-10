use std::fs;
use std::path::Path;
use anyhow::Context;
use axum::{Json, Router, Server, ServiceExt};
use axum::routing::{get, IntoMakeService};
use hyperlocal::{SocketIncoming, UnixServerExt};

pub fn build_server(path: impl AsRef<Path>) -> anyhow::Result<Server<SocketIncoming, IntoMakeService<Router>>> {
    if path.as_ref().exists() {
        fs::remove_file(path.as_ref())
            .context("Could not remove old socket!")?;
    }

    let app = Router::new()
        .route("/list", get(list_endpoints));

    let srv = axum::Server::bind_unix(path)?
        .serve(app.into_make_service());

    Ok(srv)
}

#[derive(serde::Serialize)]
pub struct Endpoint {

}

async fn list_endpoints() -> Json<Vec<Endpoint>> {
    Json(vec![])
}