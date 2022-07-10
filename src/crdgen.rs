use kube::CustomResourceExt;

use akcontroller::resources;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&resources::authentik::crd::Authentik::crd()).unwrap()
    )
}
