use std::sync::Arc;

use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    runtime::controller::Action,
    Client,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::{json, Map};
use tokio::time::Duration;
use tracing::*;

use crate::ReconcileError;

use super::{crd, deployment, ingress, service};

pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn reconcile(&self, obj: Arc<crd::Authentik>) -> Result<Action, ReconcileError> {
        debug!("Starting reconcilidation of Authentik.");
        let name = obj.name_any();
        let ns = obj
            .namespace()
            .ok_or(ReconcileError::NoNamespace(obj.name_any()))?;
        let servers: Api<crd::Authentik> = Api::namespaced(self.client.clone(), &ns);

        // Generate any default values.
        let mut obj = obj.as_ref().clone();
        let changed = self.autofill(&mut obj, &servers, name.as_str()).await?;

        if changed {
            // CRD updated, requeue so the changes are actually reflected.
            return Ok(Action::requeue(Duration::from_secs(1)));
        }

        // Reconcile all parts.
        deployment::reconcile(&obj, self.client.clone()).await?;
        service::reconcile(&obj, self.client.clone()).await?;
        ingress::reconcile(&obj, self.client.clone()).await?;

        // Update status of the CRD about the reconcilidation.
        servers
            .patch_status(
                &name,
                &PatchParams::apply("authentik.ak-operator").force(),
                &Patch::Apply(json!({
                    "apiVersion": "ak.dany.dev/v1",
                    "kind": "Authentik",
                    "status": crd::AuthentikStatus {
                        hidden: true,
                    }
                })),
            )
            .await?;

        debug!("Reconcilidation of Authentik finished successfully, re-queued for 30 minutes.");
        Ok(Action::requeue(Duration::from_secs(30 * 60)))
    }

    pub async fn cleanup(&self, obj: Arc<crd::Authentik>) -> Result<Action, ReconcileError> {
        // Cleanup all parts.
        deployment::cleanup(obj.as_ref(), self.client.clone()).await?;
        service::cleanup(obj.as_ref(), self.client.clone()).await?;
        ingress::cleanup(obj.as_ref(), self.client.clone()).await?;

        Ok(Action::await_change())
    }

    async fn autofill(
        &self,
        obj: &mut crd::Authentik,
        api: &Api<crd::Authentik>,
        name: &str,
    ) -> Result<bool, ReconcileError> {
        let mut values = Map::new();

        if obj.spec.secret_key.is_none() {
            // Create the secret if it does not yet exist
            let secret: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(128)
                .map(char::from)
                .collect();

            values.insert("secretKey".to_string(), json!(secret));
        }

        if values.is_empty() {
            return Ok(false);
        }

        let pp = PatchParams::apply("authentik.ak-operator");
        let patched_secret = Patch::Apply(json!({
            "apiVersion": "ak.dany.dev/v1",
            "kind": "Authentik",
            "spec": values
        }));

        api.patch(&name, &pp, &patched_secret).await?;
        Ok(true)
    }
}
