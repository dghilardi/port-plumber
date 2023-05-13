use std::fs;
use std::net::IpAddr;
use std::path::Path;
use anyhow::Context;
use axum::{Json, Router, Server, ServiceExt};
use axum::extract::{State};
use axum::routing::{get, IntoMakeService};
use hyperlocal::{SocketIncoming, UnixServerExt};
use crate::plumber::Plumber;

pub fn build_server(path: impl AsRef<Path>, plumber: Plumber) -> anyhow::Result<Server<SocketIncoming, IntoMakeService<Router>>> {
    if path.as_ref().exists() {
        fs::remove_file(path.as_ref())
            .context("Could not remove old socket!")?;
    }

    let app = Router::new()
        .route("/list", get(list_endpoints))
        .route("/resolve/:name", get(resolve_endpoint))
        .with_state(plumber);

    let srv = axum::Server::bind_unix(path)?
        .serve(app.into_make_service());

    Ok(srv)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Endpoint {
    pub ip: IpAddr,
}

async fn list_endpoints() -> Json<Vec<Endpoint>> {
    Json(vec![])
}

async fn resolve_endpoint(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(state): State<Plumber>
) -> Json<Endpoint> {
    let ip = state.resolve(&name);
    Json(Endpoint {
        ip,
    })
}