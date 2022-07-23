#[macro_use]
extern crate tracing;

use actix_web::{get, middleware, App, HttpRequest, HttpResponse, HttpServer, Responder};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Patch, PatchParams},
    Api, Client, CustomResourceExt, ResourceExt,
};
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

    ensure_crds().await?;

    tokio::select! {
        _ = start_managers() => warn!("A manager exited"),
        _ = server.run() => warn!("Actix Web exited"),
    }
    Ok(())
}

async fn ensure_crds() -> Result<(), StartError> {
    let crds = [
        resources::authentik::crd::Authentik::crd(),
        resources::authentik_application::crd::AuthentikApplication::crd(),
        resources::authentik_group::crd::AuthentikGroup::crd(),
        resources::authentik_user::crd::AuthentikUser::crd(),
        resources::authentik_provider_oauth::crd::AuthentikOAuthProvider::crd(),
    ];

    let client = Client::try_default().await?;
    let api: Api<CustomResourceDefinition> = Api::all(client);

    for crd in crds {
        api.patch(
            &crd.name_any(),
            &PatchParams::apply("ak-operator"),
            &Patch::Apply(&crd),
        )
        .await?;
    }

    Ok(())
}

async fn start_managers() -> Result<(), StartError> {
    let authentik_mgr = resources::AuthentikManager::new(Client::try_default().await?);
    let authentik_app_mgr = resources::AuthentikAppManager::new(Client::try_default().await?);
    let authentik_user_mgr = resources::AuthentikUserManager::new(Client::try_default().await?);
    let authentik_group_mgr = resources::AuthentikGroupManager::new(Client::try_default().await?);
    let authentik_oauth_mgr = resources::AuthentikOAuthManager::new(Client::try_default().await?);

    tokio::select! {
        _ = authentik_mgr => warn!("Authentik controller exited"),
        _ = authentik_app_mgr => warn!("Authentik user controller exited"),
        _ = authentik_user_mgr => warn!("Authentik user controller exited"),
        _ = authentik_group_mgr => warn!("Authentik user controller exited"),
        _ = authentik_oauth_mgr => warn!("Authentik oauth provider controller exited"),
    }

    Ok(())
}
