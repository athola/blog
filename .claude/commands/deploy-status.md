---
name: deploy-status
description: Check DigitalOcean deployment status, health, and SurrealDB connectivity
---

Check the current DigitalOcean App Platform deployment status for the blog.

Run these commands and report the results:

1. `doctl apps list-deployments $(doctl apps list --format ID,Spec.Name --no-header | grep alexthola-blog | awk '{print $1}') --format ID,Cause,Phase,Progress --no-header | head -5`
2. `curl -s https://alexthola-blog-4hz6l.ondigitalocean.app/health`
3. `doctl apps logs $(doctl apps list --format ID,Spec.Name --no-header | grep alexthola-blog | awk '{print $1}') --type run 2>&1 | grep -i "surreal\|connect\|router\|error\|listen" | tail -10`

Summarize:
- Current deployment phase and progress
- App version from health endpoint
- Whether SurrealDB is connected
- Any errors in runtime logs
