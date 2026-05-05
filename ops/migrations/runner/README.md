# Migration runner

CLI tool placeholder for running SQL migrations against Turso/libSQL databases. Migrations live per-service under `services/<name>/migrations/`.

```
just migrate up        # Apply all pending migrations
just migrate down N    # Rollback N migrations
just migrate status    # Show migration status
```
