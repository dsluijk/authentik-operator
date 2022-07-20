use std::sync::Arc;

use anyhow::Result;
use kube::{runtime::controller::Action, Client, ResourceExt};
use tokio::time::Duration;

use super::{crd, group};

pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn reconcile(&self, obj: Arc<crd::AuthentikGroup>) -> Result<Action> {
        debug!(
            "Starting reconcilidation of Authentik group `{}`.",
            obj.name_any()
        );

        // Reconcile all parts.
        group::reconcile(&obj, self.client.clone()).await?;

        debug!("Reconcilidation of Authentik group `{}` finished successfully, re-queued for 30 minutes.", obj.name_any());
        Ok(Action::requeue(Duration::from_secs(30 * 60)))
    }

    pub async fn cleanup(&self, obj: Arc<crd::AuthentikGroup>) -> Result<Action> {
        group::cleanup(obj.as_ref(), self.client.clone()).await?;

        Ok(Action::await_change())
    }
}
