use std::sync::Arc;

use anyhow::Result;
use kube::{runtime::controller::Action, Client, ResourceExt};
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
}
