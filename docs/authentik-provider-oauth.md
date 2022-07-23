# `AuthentikOAuthProvider`

`AuthentikOAuthProvider` creates a provider of the type OAuth 2.0 / OpenID.
This does not create a matching application, this will have to be created separately.
Note that the provider is not updated once created, althought this might change in the future.
If you need to change the provider, re-create it.

To deploy a simple example:

```bash
kubectl apply -f https://raw.githubusercontent.com/dsluijk/authentik-operator/main/docs/authentik-provider-oauth.yaml
```

## Created Secret

A secret is created with the data required for the client.
This can be mounted to the external application to connect to the server.
The secret is named by `ak-{AUTHENTIK_INSTANCE}-oauth-{PROVIDER_NAME}`.
Note that there are no URIs in the secret, as the domain and slug is not directly known.

| Key          | Description                                            |
| ------------ | ------------------------------------------------------ |
| clientType   | The type of client, can be `confidential` or `public`. |
| clientId     | The client ID of the provider.                         |
| clientSecret | The secret key of the client.                          |
| redirectUris | A list of valid redirect URL's.                        |

## Reference

A full example:

```yaml
apiVersion: ak.dany.dev/v1
kind: AuthentikOAuthProvider
metadata:
    name: oauth-provider
spec:
    authentikInstance: authentik
    name: oauth-provider
    flow: default-provider-authorization-implicit-consent
    clientType: confidential
    scopes:
        - "authentik default OAuth Mapping: OpenID 'email'"
        - "authentik default OAuth Mapping: OpenID 'openid'"
        - "authentik default OAuth Mapping: OpenID 'profile'"
    redirectUris:
        - example.com
    accessCodeValidity: minutes=1
    tokenValidity: days=30
    claimsInToken: true
    signingKey: authentik Self-signed Certificate
    subjectMode: hashed_user_id
    issuerMode: per_provider
```

| Key                | Required | Default          | Description                                                                                 |
| ------------------ | -------- | ---------------- | ------------------------------------------------------------------------------------------- |
| authentikInstance  | True     |                  | The instance of Authentik. Must match metadata.name from an `Authentik` resource.           |
| name               | True     |                  | The name of the provider.                                                                   |
| flow               | True     |                  | The authorization flow to be used in this provider. Note that this is the name of the flow. |
| clientType         | True     |                  | The client type, can be either `confidential` or `public`.                                  |
| scopes[]           | True     |                  | A list of scopes which can be used by the client. Provide the name of the scope.            |
| redirectUris[]     | True     |                  | A list of valid redirect values.                                                            |
| accessCodeValidity | False    | `minutes=1`      | Duration of the validity of generated access codes.                                         |
| tokenValidity      | False    | `days=30`        | Duration of the validity of tokens.                                                         |
| claimsInToken      | False    | `true`           | Include User claims from scopes in the id_token.                                            |
| signingKey         | False    |                  | An optional _name of the_ signing key to sign the tokens with. Required in some cases.      |
| subjectMode        | False    | `hashed_user_id` | Subject more, what data should be used to uniquely identify users. Default is mostly fine.  |
| issuerMode         | False    | `per_provider`   | Configure how the issuer field of the ID Token should be filled. Default is mostly fine.    |
