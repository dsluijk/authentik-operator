use std::sync::Arc;

use anyhow::anyhow;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube::{
    api::{Api, ListParams, ResourceExt},
    runtime::{self, controller::Action, finalizer},
    Client,
};
use tokio::{sync::Mutex, time::Duration};

use crate::ReconcileError;

mod controller;
pub mod crd;

mod application;

use controller::Controller;

pub struct Manager;

impl Manager {
    pub fn new(client: Client) -> BoxFuture<'static, ()> {
        let ctrlr = Controller::new(client.clone());
        let apps = Api::<crd::AuthentikApplication>::all(client.clone());

        let drainer = runtime::Controller::new(apps, ListParams::default())
            .run(
                move |obj, controller| Self::reconcile(obj, controller, client.clone()),
                move |_, e, _| Self::error_policy(e),
                Arc::new(Mutex::new(ctrlr)),
            )
            .filter_map(|x| async move { Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .boxed();

        drainer
    }

    async fn reconcile(
        obj: Arc<crd::AuthentikApplication>,
        controller: Arc<Mutex<Controller>>,
        client: Client,
    ) -> Result<Action, ReconcileError> {
        let ns = obj.namespace().ok_or(anyhow!(
            "Authentik application resource should have a namespace."
        ))?;
        let servers: Api<crd::AuthentikApplication> = Api::namespaced(client, &ns);

        finalizer(
            &servers,
            "authentik-application/ak.dany.dev",
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
