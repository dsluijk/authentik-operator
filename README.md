# Authentik Operator

**Not ready for production use yet!**

## Development

Install Postgres and Redis first.

```bash
helm repo add bitnami https://charts.bitnami.com/bitnami

helm install postgres --set auth.username=authentik --set auth.password=authentik --set auth.database=authentik bitnami/postgresql
helm install redis --set auth.enabled=false bitnami/redis
```
