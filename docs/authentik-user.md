# `AuthentikUser`

`AuthentikUser` will create a new normal user on the authentik server.
For now users information won't be synced preiodically, but this might change in the future.
Note that deleting the user from the Authentik server won't work properly, it will be re-created.
To delete a user properly, delete the resource.

A secret with the name `ak-{{authentikInstance}}-user-{{metadata.name}}` will be created with the login information.

To deploy a simple example:

```bash
kubectl apply -f https://raw.githubusercontent.com/dsluijk/authentik-operator/main/docs/authentik-user.yaml
```

## Reference

A full example:

```yaml
apiVersion: ak.dany.dev/v1
kind: AuthentikUser
metadata:
    name: exampleuser
spec:
    authentikInstance: authentik
    username: user
    displayName: User
    path: users
    email: user@example.com
    password: example123
    groups:
        - supergroup
```

| Key               | Required | Default    | Description                                                                                       |
| ----------------- | -------- | ---------- | ------------------------------------------------------------------------------------------------- |
| authentikInstance | True     |            | The instance of Authentik. Must match metadata.name from an `Authentik` resource.                 |
| username          | True     |            | The username of the user, can be used to log in with.                                             |
| displayName       | True     |            | The display name of the user, this is shown in lists.                                             |
| email             | False    |            | An optional email to add to the user. If set it can also be used to login.                        |
| password          | False    | `{random}` | Set the password to a fixed value. Will be randomized if not provided.                            |
| path              | True     |            | The path of the user, used for organizing the users in a tree.                                    |
| groups[]          | False    |            | A list of group _names_ to add the user to. This can be combined with `AuthentikGroup` resources. |
