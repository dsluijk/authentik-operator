use std::sync::Arc;

use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder, Reporter},
    },
    Client, Resource,
};
use serde_json::json;
use tokio::time::Duration;
use tracing::*;

use crate::ReconcileError;

use super::{crd, deployment};

pub struct Controller {
    client: Client,
    reporter: Reporter,
}

impl Controller {
    pub fn new(client: Client, reporter: Reporter) -> Self {
        Self { client, reporter }
    }

    pub async fn reconcile(&self, obj: Arc<crd::Authentik>) -> Result<Action, ReconcileError> {
        debug!("Starting reconcilidation of Authentik.");
        let name = obj.name_any();
        let ns = obj
            .namespace()
            .ok_or(ReconcileError::NoNamespace(obj.name_any()))?;
        let servers: Api<crd::Authentik> = Api::namespaced(self.client.clone(), &ns);

        // Reconcile all parts.
        deployment::reconcile(obj, self.client.clone()).await?;

        // Todo: update status accurately.
        let pp = PatchParams::apply("authentik/ak-operator").force();
        let new_status = Patch::Apply(json!({
            "apiVersion": "ak.dany.dev/v1",
            "kind": "Authentik",
            "status": crd::AuthentikStatus {
                hidden: true,
            }
        }));
        let _o = servers.patch_status(&name, &pp, &new_status).await?;

        debug!("Reconcilidation of Authentik finished, re-queued for 30 minutes.");
        Ok(Action::requeue(Duration::from_secs(30 * 60)))
    }

    pub async fn cleanup(&self, obj: Arc<crd::Authentik>) -> Result<Action, ReconcileError> {
        let recorder = Recorder::new(
            self.client.clone(),
            self.reporter.clone(),
            obj.object_ref(&()),
        );

        // Cleanup all parts.
        deployment::cleanup(obj.clone(), self.client.clone()).await?;

        recorder
            .publish(Event {
                type_: EventType::Normal,
                reason: "Deleted".into(),
                note: Some(format!("Removed `{}`", obj.name_any())),
                action: "Reconciling".into(),
                secondary: None,
            })
            .await?;

        Ok(Action::await_change())
    }
}
