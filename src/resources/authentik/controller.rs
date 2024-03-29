use std::sync::Arc;

use anyhow::{anyhow, Result};
use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    runtime::controller::Action,
    Client,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::{json, Map};
use tokio::time::Duration;

use super::{
    clusteraccount, crd, deployment, ingress, secret, service, serviceaccount, servicegroup,
};

pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn reconcile(&self, obj: Arc<crd::Authentik>) -> Result<Action> {
        info!("Starting reconcilidation of Authentik.");
        let name = obj.name_any();
        let ns = obj
            .namespace()
            .ok_or(anyhow!("Missing namespace `{}`.", obj.name_any()))?;
        let servers: Api<crd::Authentik> = Api::namespaced(self.client.clone(), &ns);

        // Generate any default values.
        let mut obj = obj.as_ref().clone();
        let changed = self.autofill(&mut obj, &servers, name.as_str()).await?;

        if changed {
            // CRD updated, requeue so the changes are actually reflected.
            return Ok(Action::requeue(Duration::from_secs(1)));
        }

        // Reconcile all parts.
        clusteraccount::reconcile(&obj, self.client.clone()).await?;
        deployment::reconcile(&obj, self.client.clone()).await?;
        service::reconcile(&obj, self.client.clone()).await?;
        ingress::reconcile(&obj, self.client.clone()).await?;
        serviceaccount::reconcile(&obj, self.client.clone()).await?;
        servicegroup::reconcile(&obj, self.client.clone()).await?;
        secret::reconcile(&obj, self.client.clone()).await?;

        info!("Reconcilidation of Authentik finished successfully, re-queued for 30 minutes.");
        Ok(Action::requeue(Duration::from_secs(30 * 60)))
    }

    pub async fn cleanup(&self, obj: Arc<crd::Authentik>) -> Result<Action> {
        // Cleanup all parts.
        secret::cleanup(obj.as_ref(), self.client.clone()).await?;
        servicegroup::cleanup(obj.as_ref(), self.client.clone()).await?;
        serviceaccount::cleanup(obj.as_ref(), self.client.clone()).await?;
        ingress::cleanup(obj.as_ref(), self.client.clone()).await?;
        service::cleanup(obj.as_ref(), self.client.clone()).await?;
        deployment::cleanup(obj.as_ref(), self.client.clone()).await?;
        clusteraccount::cleanup(obj.as_ref(), self.client.clone()).await?;

        Ok(Action::await_change())
    }

    async fn autofill(
        &self,
        obj: &mut crd::Authentik,
        api: &Api<crd::Authentik>,
        name: &str,
    ) -> Result<bool> {
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

        let pp = PatchParams::apply("authentik.ak-operator").force();
        let patched_secret = Patch::Apply(json!({
            "apiVersion": "ak.dany.dev/v1",
            "kind": "Authentik",
            "spec": values
        }));

        api.patch(&name, &pp, &patched_secret).await?;
        Ok(true)
    }
}
