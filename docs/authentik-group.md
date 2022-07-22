# `AuthentikGroup`

`AuthentikGroup` can create new groups on the server.
Note that deleting the group from the Authentik server won't work properly, it will be re-created.
To delete a group properly, delete the resource.

To deploy a simple example:

```bash
kubectl apply -f https://raw.githubusercontent.com/dsluijk/authentik-operator/main/docs/authentik-group.yaml
```

## Reference

A full example:

```yaml
apiVersion: ak.dany.dev/v1
kind: AuthentikGroup
metadata:
    name: some-group
spec:
    authentikInstance: authentik
    name: group
    superuser: false
    parent: supergroup
```

| Key               | Required | Default | Description                                                                         |
| ----------------- | -------- | ------- | ----------------------------------------------------------------------------------- |
| authentikInstance | True     |         | The instance of Authentik. Must match metadata.name from an `Authentik` resource.   |
| name              | True     |         | The name of the group in Authentik. Note that this is different from metadata.name. |
| superuser         | False    | `false` | Set to true to mark all members of this group as superuser.                         |
| parent            | False    |         | The name of the parent group. Note that this is the name, not the ID.               |
