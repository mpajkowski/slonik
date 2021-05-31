# Slonik

Slonik will be your last PostgreSQL client (as long as it is not ready, better stick with pgadmin4 or similar tools).

## Connection config

Currently `slonik` supports only one db connection at once. Connection to PG instance can be configured via
environment variables. Missing settings are replaced with defaults.

variables:

```
PG_HOST (default: 'localhost')
PG_PORT (default: '5432')
PG_USER (default: 'postgres')
PG_PASS (default: none)
PG_DBNAME (default: 'postgres')
```
