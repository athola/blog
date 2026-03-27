---
name: prevent-secret-commit
description: Block commits that contain DigitalOcean secrets, SurrealDB passwords, or API tokens
hooks:
  - event: PreToolUse
    tools: [Bash]
    pattern: "git commit"
---

Before any `git commit`, check staged files for secrets:

```bash
git diff --cached --name-only
```

If any staged file contains patterns like:
- Long base64 strings that look like passwords (32+ chars ending in `=`)
- `SURREAL_ROOT_PASS` with an actual value (not `${...}` or empty)
- `DIGITALOCEAN_ACCESS_TOKEN` with a hex value
- `doctl auth` tokens
- Droplet public IP addresses in source code (ok in .do/ config, not ok in .rs/.md files)

Then BLOCK the commit and warn:
> **Potential secret detected in staged files.** Review the diff carefully before committing. Use `git diff --cached` to inspect.
