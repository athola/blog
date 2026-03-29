---
name: deploy-logs
description: Tail DigitalOcean App Platform logs (build, deploy, or run)
args: type
---

Fetch DigitalOcean App Platform logs.

Argument `$ARGUMENTS` should be one of: `build`, `deploy`, `run` (defaults to `run`).

```bash
LOG_TYPE="${ARGUMENTS:-run}"
doctl apps logs $(doctl apps list --format ID,Spec.Name --no-header | grep alexthola-blog | awk '{print $1}') --type "$LOG_TYPE" 2>&1 | tail -30
```

Run the command above and display the output.
