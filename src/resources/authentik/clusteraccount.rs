use anyhow::{anyhow, Result};
use k8s_openapi::api::{
    core::v1::ServiceAccount,
    rbac::v1::{ClusterRole, ClusterRoleBinding},
};
use kube::{
    api::{DeleteParams, Patch, PatchParams},
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
        &PatchParams::apply("authentik.ak-operator"),
        &Patch::Apply(&build_serviceaccount(instance.clone(), obj)?),
    )
    .await?;

    // Create the cluster role.
    let api: Api<ClusterRole> = Api::all(client.clone());
    api.patch(
        &format!("ak-{}", &instance),
        &PatchParams::apply("authentik.ak-operator"),
        &Patch::Apply(&build_clusterrole(instance.clone(), obj)?),
    )
    .await?;

    // Create the cluster role binding.
    let api: Api<ClusterRoleBinding> = Api::all(client.clone());
    api.patch(
        &format!("ak-{}", &instance),
        &PatchParams::apply("authentik.ak-operator"),
        &Patch::Apply(&build_binding(instance.clone(), obj, &ns)?),
    )
    .await?;

    Ok(())
}

pub async fn cleanup(obj: &crd::Authentik, client: Client) -> Result<()> {
    let instance = obj
        .metadata
        .name
        .clone()
        .ok_or(anyhow!("Missing instance name.".to_string()))?;

    // Clean up cluster resources as owner references don't work.
    let api: Api<ClusterRole> = Api::all(client.clone());
    api.delete(&format!("ak-{}", &instance), &DeleteParams::foreground())
        .await?;

    let api: Api<ClusterRoleBinding> = Api::all(client.clone());
    api.delete(&format!("ak-{}", &instance), &DeleteParams::foreground())
        .await?;

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
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true,
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
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "clusteraccount".to_string())
        },
        "rules": [
            {
                "apiGroups": [""],
                "resources": ["secrets", "services", "configmaps"],
                "verbs": ["*"]
            },
            {
                "apiGroups": ["extensions", "apps"],
                "resources": ["deployments"],
                "verbs": ["*"]
            },
            {
                "apiGroups": ["extensions", "networking.k8s.io"],
                "resources": ["ingresses"],
                "verbs": ["*"]
            },
            {
                "apiGroups": ["traefik.containo.us"],
                "resources": ["middlewares"],
                "verbs": ["*"]
            },
            {
                "apiGroups": ["monitoring.coreos.com"],
                "resources": ["servicemonitors"],
                "verbs": ["*"]
            },
            {
                "apiGroups": ["apiextensions.k8s.io"],
                "resources": ["customresourcedefinitions"],
                "verbs": ["*"]
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
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "clusteraccount".to_string())
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
