use anyhow::{anyhow, Result};
use k8s_openapi::api::{
    core::v1::ServiceAccount,
    rbac::v1::{ClusterRole, ClusterRoleBinding},
};
use kube::{
    api::{Patch, PatchParams},
    Api, Client, ResourceExt,
};
use serde_json::json;

use super::{crd, labels};

pub async fn reconcile(obj: &crd::Authentik, client: Client) -> Result<()> {
    let instance = obj
        .metadata
        .name
        .clone()
        .ok_or(anyhow!("Missing instance name.".to_string()))?;
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Create the service account.
    let api: Api<ServiceAccount> = Api::namespaced(client.clone(), &ns);
    api.patch(
        &format!("ak-{}", &instance),
        &PatchParams::apply("authentik.ak-operator").force(),
        &Patch::Apply(&build_serviceaccount(instance.clone(), obj)?),
    )
    .await?;

    // Create the cluster role.
    let api: Api<ClusterRole> = Api::all(client.clone());
    api.patch(
        &format!("ak-{}", &instance),
        &PatchParams::apply("authentik.ak-operator").force(),
        &Patch::Apply(&build_clusterrole(instance.clone(), obj)?),
    )
    .await?;

    // Create the cluster role binding.
    let api: Api<ClusterRoleBinding> = Api::all(client.clone());
    api.patch(
        &format!("ak-{}", &instance),
        &PatchParams::apply("authentik.ak-operator").force(),
        &Patch::Apply(&build_binding(instance.clone(), obj, &ns)?),
    )
    .await?;

    Ok(())
}

pub async fn cleanup(_obj: &crd::Authentik, _client: Client) -> Result<()> {
    // Note: The account will automatically be cleaned up by Kubernetes.
    Ok(())
}

fn build_serviceaccount(name: String, obj: &crd::Authentik) -> Result<ServiceAccount> {
    let account: ServiceAccount = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": {
            "name": format!("ak-{}", &name),
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "clusteraccount".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik.")
            }]
        }
    }))?;

    Ok(account)
}

fn build_clusterrole(name: String, obj: &crd::Authentik) -> Result<ClusterRole> {
    let role: ClusterRole = serde_json::from_value(json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "ClusterRole",
        "metadata": {
            "name": format!("ak-{}", &name),
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "clusteraccount".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik.")
            }]
        },
        "rules": [
            {
                "apiGroups": [""],
                "resources": ["secrets", "services", "configmaps"],
                "verbs": ["get", "create", "delete", "list", "patch"]
            },
            {
                "apiGroups": ["extensions", "apps"],
                "resources": ["deployments"],
                "verbs": ["get", "create", "delete", "list", "patch"]
            },
            {
                "apiGroups": ["extensions", "networking.k8s.io"],
                "resources": ["ingresses"],
                "verbs": ["get", "create", "delete", "list", "patch"]
            },
            {
                "apiGroups": ["traefik.containo.us"],
                "resources": ["middlewares"],
                "verbs": ["get", "create", "delete", "list", "patch"]
            },
            {
                "apiGroups": ["monitoring.coreos.com"],
                "resources": ["servicemonitors"],
                "verbs": ["get", "create", "delete", "list", "patch"]
            },
            {
                "apiGroups": ["apiextensions.k8s.io"],
                "resources": ["customresourcedefinitions"],
                "verbs": ["list"]
            }
        ]
    }))?;

    Ok(role)
}

fn build_binding(name: String, obj: &crd::Authentik, ns: &str) -> Result<ClusterRoleBinding> {
    let binding: ClusterRoleBinding = serde_json::from_value(json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "ClusterRoleBinding",
        "metadata": {
            "name": format!("ak-{}", &name),
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "clusteraccount".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik.")
            }]
        },
        "roleRef": {
            "apiGroup": "rbac.authorization.k8s.io",
            "kind": "ClusterRole",
            "name": format!("ak-{}", &name)
        },
        "subjects": [{
            "kind": "ServiceAccount",
            "name": format!("ak-{}", &name),
            "namespace": ns
        }]
    }))?;

    Ok(binding)
}
