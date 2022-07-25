# `AuthentikApplication`

`AuthentikApplication` defines an external application which uses Authentik as an identity provider.
An application is dependent on a provider, like `AuthentikOAuthProvider`.

To deploy a simple example:

```bash
kubectl apply -f https://raw.githubusercontent.com/dsluijk/authentik-operator/main/docs/authentik-provider-oauth.yaml
kubectl apply -f https://raw.githubusercontent.com/dsluijk/authentik-operator/main/docs/authentik-application.yaml
```

## Reference

A full example:

```yaml
apiVersion: ak.dany.dev/v1
kind: AuthentikApplication
metadata:
    name: example-application
spec:
    authentikInstance: authentik
    name: example
    slug: example
    provider: testing-provider
    group: example-group
    policyMode: any
    ui:
        newTab: false
        url: https://example.com
        icon: fa://fa-eye
        description: This is the example application from the Authentik Operator
        publisher: Authentik Operator
```

| Key               | Required | Default         | Description                                                                                   |
| ----------------- | -------- | --------------- | --------------------------------------------------------------------------------------------- |
| authentikInstance | True     |                 | The instance of Authentik. Must match metadata.name from an `Authentik` resource.             |
| name              | True     |                 | The name of the application.                                                                  |
| slug              | True     |                 | The slug used in internal urls. Do not change this afterwards.                                |
| provider          | True     |                 | The provider to use with this application. This is the _name_ of the application, not the ID. |
| group             | False    | `""`            | The name of the group of the application. Used to group applications together.                |
| policyMode        | False    | `"any"`         | Policy engine mode to use. Valid values: `any` and `all`.                                     |
| ui.newTab         | False    | `false`         | When true, the application will be launched in a new tab when launched from the library.      |
| ui.url            | False    |                 | The url to use when launching the application from the library.                               |
| ui.icon           | False    | `"fa://fa-eye"` | The url of the icon to display in the library.                                                |
| ui.description    | False    | `""`            | Description of the application, shown in the library                                          |
| ui.publisher      | False    | `""`            | Publisher of the application, shown in the library                                            |
