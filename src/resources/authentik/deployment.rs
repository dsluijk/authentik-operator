use anyhow::{anyhow, Result};
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{EnvVar, EnvVarSource, SecretKeySelector},
};
use kube::{
    api::{Patch, PatchParams},
    Api, Client, ResourceExt,
};
use serde_json::json;

use crate::akapi::auth::TEMP_AUTH_TOKEN;

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

    // Create the server deployment.
    let api: Api<Deployment> = Api::namespaced(client.clone(), &ns);
    api.patch(
        &format!("authentik-{}-server", instance),
        &PatchParams::apply("authentik.ak-operator"),
        &Patch::Strategic(&build_server(instance.clone(), obj)?),
    )
    .await?;

    // Create the worker deployment.
    let api: Api<Deployment> = Api::namespaced(client, &ns);
    api.patch(
        &format!("authentik-{}-worker", instance),
        &PatchParams::apply("authentik.ak-operator"),
        &Patch::Strategic(&build_worker(instance.clone(), obj)?),
    )
    .await?;

    Ok(())
}

pub async fn cleanup(_obj: &crd::Authentik, _client: Client) -> Result<()> {
    Ok(())
}

fn build_server(name: String, obj: &crd::Authentik) -> Result<Deployment> {
    let deployment: Deployment = serde_json::from_value(json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": format!("authentik-{}-server", name.clone()),
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "server".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true,
            }]
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": labels::get_matching_labels(name.clone(), "server".to_string())
            },
            "template": {
                "metadata": {
                    "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "server".to_string()),
                },
                "spec": {
                    "serviceAccountName": format!("ak-{}", name),
                    "enableServiceLinks": true,
                    "containers": [{
                        "name": format!("authentik-{}-server", name),
                        "image": format!("{}:{}", obj.spec.image.repository, obj.spec.image.tag),
                        "imagePullPolicy": obj.spec.image.pull_policy,
                        "args": ["server"],
                        "ports": [{
                            "name": "http",
                            "containerPort": 9000,
                            "protocol": "TCP"
                        }],
                        "startupProbe": {
                            "failureThreshold": 30,
                            "periodSeconds": 10,
                            "httpGet": {
                                "path": "/-/health/live/",
                                "port": "http"
                            }
                        },
                        "livenessProbe": {
                            "failureThreshold": 2,
                            "periodSeconds": 10,
                            "httpGet": {
                                "path": "/-/health/live/",
                                "port": "http"
                            }
                        },
                        "readinessProbe": {
                            "failureThreshold": 2,
                            "periodSeconds": 10,
                            "httpGet": {
                                "path": "/-/health/ready/",
                                "port": "http"
                            }
                        },
                        "env": build_env(&obj.spec)
                    }]
                }
            }
        }
    }))?;

    Ok(deployment)
}

fn build_worker(name: String, obj: &crd::Authentik) -> Result<Deployment> {
    let deployment: Deployment = serde_json::from_value(json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": format!("authentik-{}-worker", name.clone()),
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "worker".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true,
            }]
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": labels::get_matching_labels(name.clone(), "worker".to_string())
            },
            "template": {
                "metadata": {
                    "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "worker".to_string()),
                },
                "spec": {
                    "serviceAccountName": format!("ak-{}", name),
                    "enableServiceLinks": true,
                    "containers": [{
                        "name": format!("authentik-{}-worker", name),
                        "image": format!("{}:{}", obj.spec.image.repository, obj.spec.image.tag),
                        "imagePullPolicy": obj.spec.image.pull_policy,
                        "args": ["worker"],
                        "env": build_env(&obj.spec)
                    }]
                }
            }
        }
    }))?;

    Ok(deployment)
}

fn build_env(obj: &crd::AuthentikSpec) -> Vec<EnvVar> {
    let mut env = vec![
        EnvVar {
            name: "AUTHENTIK_SECRET_KEY".to_string(),
            value: obj.secret_key.clone(),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_BOOTSTRAP_TOKEN".to_string(),
            value: Some(TEMP_AUTH_TOKEN.to_string()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_FOOTER_LINKS".to_string(),
            value: Some(serde_json::to_string(&obj.footer_links).expect("Invalid footer")),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_DISABLE_STARTUP_ANALYTICS".to_string(),
            value: Some("true".to_string()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_ERROR_REPORTING__ENABLED".to_string(),
            value: Some("false".to_string()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_POSTGRESQL__HOST".to_string(),
            value: Some(obj.postgres.host.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_POSTGRESQL__PORT".to_string(),
            value: Some(obj.postgres.port.to_string()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_POSTGRESQL__NAME".to_string(),
            value: Some(obj.postgres.database.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_POSTGRESQL__USER".to_string(),
            value: Some(obj.postgres.username.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_REDIS__HOST".to_string(),
            value: Some(obj.redis.host.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_REDIS__PORT".to_string(),
            value: Some(obj.redis.port.to_string()),
            value_from: None,
        },
    ];

    if let Some(log_level) = &obj.log_level {
        env.push(EnvVar {
            name: "AUTHENTIK_LOG_LEVEL".to_string(),
            value: Some(log_level.clone()),
            value_from: None,
        });
    }

    if let Some((secret, key)) = obj
        .postgres
        .password_secret
        .clone()
        .zip(obj.postgres.password_secret_key.as_ref())
    {
        env.push(EnvVar {
            name: "AUTHENTIK_POSTGRESQL__PASSWORD".to_string(),
            value: None,
            value_from: Some(EnvVarSource {
                config_map_key_ref: None,
                field_ref: None,
                resource_field_ref: None,
                secret_key_ref: Some(SecretKeySelector {
                    key: key.clone(),
                    name: Some(secret),
                    optional: Some(false),
                }),
            }),
        });
    } else {
        env.push(EnvVar {
            name: "AUTHENTIK_POSTGRESQL__PASSWORD".to_string(),
            value: Some(obj.postgres.password.clone()),
            value_from: None,
        });
    }

    if let Some(password) = obj.redis.password.as_ref() {
        env.push(EnvVar {
            name: "AUTHENTIK_REDIS__PASSWORD".to_string(),
            value: Some(password.clone()),
            value_from: None,
        });
    }

    env.extend(build_env_smtp(obj.smtp.as_ref()));

    env
}

fn build_env_smtp(obj: Option<&crd::AuthentikSmtp>) -> Vec<EnvVar> {
    let obj = match obj {
        Some(obj) => obj,
        None => return vec![],
    };

    vec![
        EnvVar {
            name: "AUTHENTIK_EMAIL__HOST".to_string(),
            value: Some(obj.host.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_EMAIL__PORT".to_string(),
            value: Some(obj.port.to_string()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_EMAIL__FROM".to_string(),
            value: Some(obj.from.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_EMAIL__USERNAME".to_string(),
            value: Some(obj.username.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_EMAIL__PASSWORD".to_string(),
            value: Some(obj.password.clone()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_EMAIL__USE_TLS".to_string(),
            value: Some(obj.use_tls.to_string()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_EMAIL__USE_SSL".to_string(),
            value: Some(obj.use_ssl.to_string()),
            value_from: None,
        },
        EnvVar {
            name: "AUTHENTIK_EMAIL__TIMEOUT".to_string(),
            value: Some(obj.timeout.to_string()),
            value_from: None,
        },
    ]
}
