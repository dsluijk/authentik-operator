#[macro_use]
extern crate tracing;

use actix_web::{get, middleware, App, HttpRequest, HttpResponse, HttpServer, Responder};
use kube::Client;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

use akcontroller::resources;
use akcontroller::StartError;

#[get("/health")]
async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[tokio::main]
async fn main() -> Result<(), StartError> {
    let logger = tracing_subscriber::fmt::layer();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|_| StartError::TracingError)?;

    let collector = Registry::default().with(logger).with(env_filter);
    tracing::subscriber::set_global_default(collector).map_err(|_| StartError::TracingError)?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default().exclude("/health"))
            .service(health)
    })
    .bind("0.0.0.0:8080")?
    .shutdown_timeout(5);

    tokio::select! {
        _ = start_managers() => warn!("A manager exited"),
        _ = server.run() => info!("Actix Web exited"),
    }
    Ok(())
}

async fn start_managers() -> Result<(), StartError> {
    let client = Client::try_default().await?;

    let authentik_mgr = resources::AuthentikManager::new(client);

    tokio::select! {
        _ = authentik_mgr => warn!("Authentik controller exited"),
    }

    Ok(())
}
