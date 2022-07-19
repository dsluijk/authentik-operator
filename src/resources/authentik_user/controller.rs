use std::sync::Arc;

use anyhow::Result;
use kube::{api::ResourceExt, runtime::controller::Action, Client};
use tokio::time::Duration;

use super::{crd, group, password, user};

pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn reconcile(&self, obj: Arc<crd::AuthentikUser>) -> Result<Action> {
        debug!(
            "Starting reconcilidation of Authentik user `{}`.",
            obj.name_any()
        );

        // Reconcile all parts.
        user::reconcile(&obj, self.client.clone()).await?;
        password::reconcile(&obj, self.client.clone()).await?;
        group::reconcile(&obj, self.client.clone()).await?;

        debug!("Reconcilidation of Authentik user `{}` finished successfully, re-queued for 30 minutes.", obj.name_any());
        Ok(Action::requeue(Duration::from_secs(30 * 60)))
    }

    pub async fn cleanup(&self, obj: Arc<crd::AuthentikUser>) -> Result<Action> {
        group::cleanup(obj.as_ref(), self.client.clone()).await?;
        password::cleanup(obj.as_ref(), self.client.clone()).await?;
        user::cleanup(obj.as_ref(), self.client.clone()).await?;

        Ok(Action::await_change())
    }
}
