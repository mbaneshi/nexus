# REST API

Start the server with `nexus serve`. All endpoints are prefixed with `/api`.

## Endpoints

### `GET /api/health`

Health check. Returns `200 OK`.

### `GET /api/stats`

Home directory statistics.

```json
{
  "total_files": 150000,
  "total_size": 5368709120,
  "categories": { "code": 45000, "config": 3200, ... },
  "last_scan": "2026-03-24T10:00:00Z"
}
```

### `GET /api/search?q=<query>&category=<cat>&limit=<n>`

Full-text search.

| Param | Required | Description |
|-------|----------|-------------|
| `q` | yes | Search query |
| `category` | no | Filter by category |
| `limit` | no | Max results (default 20) |

### `GET /api/config/tools`

List all discovered config tools.

### `GET /api/config/snapshots`

List all config snapshots.

### `POST /api/config/backup`

Create a new config snapshot. Returns the snapshot ID.

### `POST /api/config/restore/:id`

Restore configs from a snapshot.

### `GET /api/config/:tool/diff`

Diff current config vs last snapshot for a specific tool.

### `GET /api/daemon/status`

Daemon running status.

### `GET /api/changes?limit=<n>`

Recent filesystem changes.

| Param | Required | Description |
|-------|----------|-------------|
| `limit` | no | Max changes to return |
