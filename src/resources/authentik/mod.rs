use std::sync::Arc;

use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube::{
    api::{Api, ListParams, ResourceExt},
    runtime::{self, controller::Action, events::Reporter, finalizer},
    Client,
};
use tokio::time::Duration;
use tracing::*;

use crate::ReconcileError;

mod controller;
pub mod crd;
mod deployment;

use controller::Controller;

pub struct Manager;

impl Manager {
    pub fn new(client: Client) -> BoxFuture<'static, ()> {
        let reporter: Reporter = "authentik/ak-operator".into();
        let ctrlr = Controller::new(client.clone(), reporter);

        let servers = Api::<crd::Authentik>::all(client.clone());
        let drainer = runtime::Controller::new(servers, ListParams::default())
            .run(
                move |obj, controller| Self::reconcile(obj, controller, client.clone()),
                move |e, _| Self::error_policy(e),
                Arc::new(ctrlr),
            )
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .boxed();

        drainer
    }

    async fn reconcile(
        obj: Arc<crd::Authentik>,
        controller: Arc<Controller>,
        client: Client,
    ) -> Result<Action, ReconcileError> {
        let ns = obj
            .namespace()
            .expect("Authentik resource should have a namespace.");
        let servers: Api<crd::Authentik> = Api::namespaced(client, &ns);

        finalizer(&servers, "authentik/ak.dany.dev", obj, |event| async {
            match event {
                finalizer::Event::Apply(server) => controller.reconcile(server).await,
                finalizer::Event::Cleanup(server) => controller.cleanup(server).await,
            }
        })
        .await
        .map_err(|e| e.into())
    }

    fn error_policy(error: &ReconcileError) -> Action {
        warn!("reconcile failed: {:?}", error);
        Action::requeue(Duration::from_secs(5 * 60))
    }
}
