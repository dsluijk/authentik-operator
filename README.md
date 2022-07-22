# Authentik Operator

Controller for the Authentik identity provider.

In it's simplest form it's a direct replacement for the official [Helm chart](https://github.com/goauthentik/helm), with the bootstrapping functions removed.
This can then be extended with other CRD's to automatically create certain objects through the API.
This makes it ideal to be used in combination with GitOps tools like [FluxCD](https://fluxcd.io/).

## Installation

Installation is done with Helm, and very simple.
The values are in most cases fine, as most configuration is done in the CRD's.
It's ensured that the CRD's are installed when starting the controller, so no action is required here.
Scroll down to [Usage](#usage) after this to see how you can then deploy Authentik.

To install the operator, run the following commands:

```bash
helm repo add akoperator https://dsluijk.github.io/authentik-operator
helm repo update
helm install akoperator akoperator/authentik-operator
```

You can always uninstall the operator.
Do make sure to remove any related objects first.
The uninstallation does not delete the CRD's, this will have to be done manually.
When all required objects are deleted, then you can delete it like so:

```bash
helm uninstall akoperator
```

## Usage

### Quickstart

Authentik requires a Postgres and Redis database to function.
This controller does not provide one for you, so you'll have to install this yourself.
For testing purposes, a simple install of the Bitnami charts will suffice.
Make sure you have something more permanent when moving this to production, this operator does not make backups.

```bash
helm repo add bitnami https://charts.bitnami.com/bitnami

helm install postgres --set auth.username=authentik --set auth.password=authentik --set auth.database=authentik bitnami/postgresql
helm install redis --set auth.enabled=false bitnami/redis
```

Then to create a simple server, you can use the following one-liner:

```bash
kubectl apply -f https://raw.githubusercontent.com/dsluijk/authentik-operator/main/docs/complete.yaml
```

Give it a minute to initialize, and then you can visit it through your ingress controller at the host [login.example.com](https://login.example.com).
Here you can login with the user `admin` and the password `HelloController!`.
This user has full admin rights, and you can use the Authentik instance like normal.

For a more in-depth guide, scroll down a little bit.

### Custom Resources

This controller watches multiple custom resources, which all have their own page of documentation.
Click the link corresponding to the CRD you want to know more about to go there.

Is your custom resource not in here?
Open an issue!

| CRD                                                        | Description                                                    |
| ---------------------------------------------------------- | -------------------------------------------------------------- |
| [Authentik](docs/authentik.md)                             | An instance of Authentik. This is required for any deployment. |
| [AuthentikGroup](docs/authentik-group.md)                  | Group within Authentik. This can be a superuser group.         |
| [AuthentikOAuthProvider](docs/authentik-provider-oauth.md) | Creates a OAuth 2.0 / OpenID provider.                         |
| [AuthentikUser](docs/authentik-user.md)                    | Authentik user, as you are familiar with.                      |

## Differences

This operator changes some behavior compared to a "vanilla" installation of Authentik.
The most notable of these are, in no particulair order:

-   The initial setup flow is deleted, along with the associated stage. Normally this is used in the `/if/flow/initial-setup/` page.
-   The `akadmin` user is deleted, along with the `authentik Admins` group.
-   A group `akOperator authentik service group` is created. **Do not delete this group**.
-   A service account `ak-operator` is created. **Also do not delete this**.
-   An `ak-operator-authentik__operatortoken` api token is created. You get it now, **don't delete this**.
-   An hardcoded bootstrap token is added, but removed soon after.

## Development

This controller is created using Rust, so make sure you have the required tools for this.
To test your changes it's good to use a tool like [Telepresence](https://www.telepresence.io/docs/latest/quick-start/), as you can interact with the Kubernetes API with a simple `telepresence connect`.
After connecting to your cluster you can simply run the default binary, it should pick up the cluster from you local Kubefile.
Make sure you don't still have a controller installed in the cluster, as this can conflict with eachother depending on the change.
