use std::sync::Arc;

use anyhow::anyhow;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{Secret, Service, ServiceAccount},
    networking::v1::Ingress,
    rbac::v1::{ClusterRole, ClusterRoleBinding},
};
use kube::{
    api::{Api, ListParams, ResourceExt},
    runtime::{self, controller::Action, finalizer},
    Client,
};
use tokio::{sync::Mutex, time::Duration};

mod controller;
pub mod crd;
pub mod labels;

mod clusteraccount;
mod deployment;
mod ingress;
mod secret;
mod service;
mod serviceaccount;
mod servicegroup;

use controller::Controller;

use crate::ReconcileError;

use super::list_lp;

pub struct Manager;

impl Manager {
    pub fn new(client: Client) -> BoxFuture<'static, ()> {
        let ctrlr = Controller::new(client.clone());

        let servers = Api::<crd::Authentik>::all(client.clone());
        let deploys = Api::<Deployment>::all(client.clone());
        let services = Api::<Service>::all(client.clone());
        let ingresses = Api::<Ingress>::all(client.clone());
        let secrets = Api::<Secret>::all(client.clone());
        let serviceaccounts = Api::<ServiceAccount>::all(client.clone());
        let clusterroles = Api::<ClusterRole>::all(client.clone());
        let clusterrolebindings = Api::<ClusterRoleBinding>::all(client.clone());
        let lp = list_lp("ak-ak");

        let drainer = runtime::Controller::new(servers, ListParams::default())
            .owns(deploys, lp.clone())
            .owns(services, lp.clone())
            .owns(ingresses, lp.clone())
            .owns(secrets, lp.clone())
            .owns(serviceaccounts, lp.clone())
            .owns(clusterroles, lp.clone())
            .owns(clusterrolebindings, lp.clone())
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
        obj: Arc<crd::Authentik>,
        controller: Arc<Mutex<Controller>>,
        client: Client,
    ) -> Result<Action, ReconcileError> {
        let ns = obj
            .namespace()
            .ok_or(anyhow!("Authentik resource should have a namespace."))?;
        let servers: Api<crd::Authentik> = Api::namespaced(client, &ns);

        finalizer(&servers, "authentik/ak.dany.dev", obj, |event| async {
            // Make sure only one reconciliation can be run at the same time.
            let controller = controller.lock().await;

            match event {
                finalizer::Event::Apply(server) => controller.reconcile(server).await,
                finalizer::Event::Cleanup(server) => controller.cleanup(server).await,
            }
            .map_err(|e| e.into())
        })
        .await
        .map_err(|e| e.into())
    }

    fn error_policy(error: &ReconcileError) -> Action {
        warn!("{}", error);
        Action::requeue(Duration::from_secs(60))
    }
}
