use std::collections::BTreeMap;

pub fn get_labels(
    instance: String,
    version: String,
    component: String,
) -> BTreeMap<String, String> {
    let mut labels = get_matching_labels(instance, component);
    labels.insert(
        "app.kubernetes.io/created-by".to_string(),
        "authentik-operator".to_string(),
    );
    labels.insert("app.kubernetes.io/version".to_string(), version);

    labels
}

pub fn get_matching_labels(instance: String, component: String) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.kubernetes.io/name".to_string(),
            "authentik".to_string(),
        ),
        ("app.kubernetes.io/part-of".to_string(), "ak-ak".to_string()),
        ("app.kubernetes.io/instance".to_string(), instance),
        ("app.kubernetes.io/component".to_string(), component),
    ])
}
