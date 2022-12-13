use std::sync::Arc;

use anyhow::{anyhow, Result};
use kube::{
    api::{Patch, PatchParams},
    runtime::controller::Action,
    Api, Client, ResourceExt,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::{json, Map};
use tokio::time::Duration;

use super::{crd, provider, secret};

pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn reconcile(&self, obj: Arc<crd::AuthentikOAuthProvider>) -> Result<Action> {
        info!(
            "Starting reconcilidation of Authentik oauth provider `{}`.",
            obj.name_any()
        );
        let name = obj.name_any();
        let ns = obj
            .namespace()
            .ok_or(anyhow!("Missing namespace `{}`.", obj.name_any()))?;
        let servers: Api<crd::AuthentikOAuthProvider> = Api::namespaced(self.client.clone(), &ns);

        // Generate any default values.
        let mut obj = obj.as_ref().clone();
        let changed = self.autofill(&mut obj, &servers, name.as_str()).await?;

        if changed {
            // CRD updated, requeue so the changes are actually reflected.
            return Ok(Action::requeue(Duration::from_secs(1)));
        }

        // Reconcile all parts.
        provider::reconcile(&obj, self.client.clone()).await?;
        secret::reconcile(&obj, self.client.clone()).await?;

        info!("Reconcilidation of Authentik oauth provider `{}` finished successfully, re-queued for 30 minutes.", obj.name_any());
        Ok(Action::requeue(Duration::from_secs(30 * 60)))
    }

    pub async fn cleanup(&self, obj: Arc<crd::AuthentikOAuthProvider>) -> Result<Action> {
        secret::cleanup(obj.as_ref(), self.client.clone()).await?;
        provider::cleanup(obj.as_ref(), self.client.clone()).await?;

        Ok(Action::await_change())
    }

    async fn autofill(
        &self,
        obj: &mut crd::AuthentikOAuthProvider,
        api: &Api<crd::AuthentikOAuthProvider>,
        name: &str,
    ) -> Result<bool> {
        let mut values = Map::new();

        if obj.spec.client_id.is_none() {
            // Create the secret if it does not yet exist
            let client_id: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(128)
                .map(char::from)
                .collect();

            values.insert("clientId".to_string(), json!(client_id));
        }

        if obj.spec.client_secret.is_none() {
            // Create the secret if it does not yet exist
            let secret: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(255)
                .map(char::from)
                .collect();

            values.insert("clientSecret".to_string(), json!(secret));
        }

        if values.is_empty() {
            return Ok(false);
        }

        let pp = PatchParams::apply("authentik.ak-operator").force();
        let patched_secret = Patch::Apply(json!({
            "apiVersion": "ak.dany.dev/v1",
            "kind": "AuthentikOAuthProvider",
            "spec": values
        }));

        api.patch(&name, &pp, &patched_secret).await?;
        Ok(true)
    }
}
