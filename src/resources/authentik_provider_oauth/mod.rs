use std::sync::Arc;

use anyhow::anyhow;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, ListParams, ResourceExt},
    runtime::{self, controller::Action, finalizer},
    Client,
};
use tokio::{sync::Mutex, time::Duration};

mod controller;
pub mod crd;
mod labels;

mod provider;
mod secret;

use controller::Controller;

use crate::ReconcileError;

use super::list_lp;

pub struct Manager;

impl Manager {
    pub fn new(client: Client) -> BoxFuture<'static, ()> {
        let ctrlr = Controller::new(client.clone());
        let users = Api::<crd::AuthentikOAuthProvider>::all(client.clone());

        let secrets = Api::<Secret>::all(client.clone());
        let lp = list_lp("ak-provider-oauth");

        let drainer = runtime::Controller::new(users, ListParams::default())
            .owns(secrets, lp.clone())
            .run(
                move |obj, controller| Self::reconcile(obj, controller, client.clone()),
                move |e, _| Self::error_policy(e),
                Arc::new(Mutex::new(ctrlr)),
            )
            .filter_map(|x| async move { Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .boxed();

        drainer
    }

    async fn reconcile(
        obj: Arc<crd::AuthentikOAuthProvider>,
        controller: Arc<Mutex<Controller>>,
        client: Client,
    ) -> Result<Action, ReconcileError> {
        let ns = obj.namespace().ok_or(anyhow!(
            "Authentik oauth provider resource should have a namespace."
        ))?;
        let servers: Api<crd::AuthentikOAuthProvider> = Api::namespaced(client, &ns);

        finalizer(
            &servers,
            "authentik-oauth/ak.dany.dev",
            obj,
            |event| async {
                // Make sure only one reconciliation can be run at the same time.
                let controller = controller.lock().await;

                match event {
                    finalizer::Event::Apply(server) => controller.reconcile(server).await,
                    finalizer::Event::Cleanup(server) => controller.cleanup(server).await,
                }
                .map_err(|e| e.into())
            },
        )
        .await
        .map_err(|e| e.into())
    }

    fn error_policy(error: &ReconcileError) -> Action {
        warn!("{}", error);
        Action::requeue(Duration::from_secs(60))
    }
}
