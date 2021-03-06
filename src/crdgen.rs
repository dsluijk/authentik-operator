use kube::CustomResourceExt;

use akcontroller::resources;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&resources::authentik::crd::Authentik::crd()).unwrap()
    );
    print!(
        "{}",
        serde_yaml::to_string(&resources::authentik_application::crd::AuthentikApplication::crd())
            .unwrap()
    );
    print!(
        "{}",
        serde_yaml::to_string(&resources::authentik_user::crd::AuthentikUser::crd()).unwrap()
    );
    print!(
        "{}",
        serde_yaml::to_string(&resources::authentik_group::crd::AuthentikGroup::crd()).unwrap()
    );
    print!(
        "{}",
        serde_yaml::to_string(
            &resources::authentik_provider_oauth::crd::AuthentikOAuthProvider::crd()
        )
        .unwrap()
    );
}
