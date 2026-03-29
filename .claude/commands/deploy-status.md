---
name: deploy-status
description: Check DigitalOcean deployment status, health, and SurrealDB connectivity
---

Check the current DigitalOcean App Platform deployment status for the blog.

Run these commands and report the results:

First, resolve the app ID and URL:
```bash
APP_ID=$(doctl apps list --format ID,Spec.Name --no-header | grep alexthola-blog | awk '{print $1}')
APP_URL=$(doctl apps list --format DefaultIngress,Spec.Name --no-header | grep alexthola-blog | awk '{print $1}')
```

1. `doctl apps list-deployments "$APP_ID" --format ID,Cause,Phase,Progress --no-header | head -5`
2. `curl -s "$APP_URL/health"`
3. `doctl apps logs "$APP_ID" --type run 2>&1 | grep -i "surreal\|connect\|router\|error\|listen" | tail -10`

Summarize:
- Current deployment phase and progress
- App version from health endpoint
- Whether SurrealDB is connected
- Any errors in runtime logs
