use std::time::Duration;

use anyhow::{anyhow, Result};
use hyper::{client::conn::SendRequest, Body, Method, Request, Response};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::ListParams,
    runtime::wait::{await_condition, Condition},
    Api, Client, ResourceExt,
};
use rand::prelude::SliceRandom;
use serde::Serialize;

use crate::{resources::authentik::labels, AKApiError};

#[derive(Debug)]
pub struct AkServer {
    sender: SendRequest<Body>,
    host: String,
}

impl AkServer {
    pub async fn connect(instance: &str, namespace: &str, client: Client) -> Result<Self> {
        let label_selectors: Vec<String> =
            labels::get_matching_labels(instance.to_string(), "server".to_string())
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();

        // Find a random pod with the selectors of the Deployment.
        let pods_api: Api<Pod> = Api::namespaced(client, namespace);
        let lp = ListParams::default().labels(&label_selectors.join(","));
        let pods = pods_api.list(&lp).await?;
        let pod = pods.items.choose(&mut rand::thread_rng()).ok_or(anyhow!(
            "failed to find pods with selectors: pod/{}",
            instance
        ))?;
        let pod_name = pod.name_any();

        // Wait for the pod to be ready.
        let running = await_condition(
            pods_api.clone(),
            &pod_name,
            Self::is_pod_ready(format!("authentik-{}-server", instance)),
        );
        let _ = tokio::time::timeout(Duration::from_secs(180), running).await?;
        debug!("Authentik API ready.");

        // Port forward to the pod api.
        let mut pf = pods_api.portforward(&pod_name, &[9000]).await?;
        let port = pf
            .take_stream(9000)
            .ok_or(anyhow!("Failed to get stream of port".to_string()))?;
        let (sender, connection) = hyper::client::conn::handshake(port).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                warn!("Error in connection: {}", e);
            }
        });

        Ok(Self {
            sender,
            host: format!("authentik-{}", instance),
        })
    }

    pub async fn send(
        &mut self,
        method: Method,
        uri: &str,
        api_key: &str,
        body: impl Serialize,
    ) -> Result<Response<Body>, AKApiError> {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header("Connection", "keep-alive")
            .header("Host", self.host.as_str())
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&body)?))?;

        self.sender.send_request(req).await.map_err(|e| e.into())
    }

    #[must_use]
    pub fn is_pod_ready(container_name: String) -> impl Condition<Pod> {
        move |obj: Option<&Pod>| {
            if let Some(pod) = &obj {
                if let Some(status) = &pod.status {
                    if let Some(statusses) = &status.container_statuses {
                        for status in statusses {
                            if status.name != container_name {
                                continue;
                            }

                            return status.ready;
                        }
                    }
                }
            }
            false
        }
    }
}
