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
- `GqQt6` or base64 password strings
- `SURREAL_ROOT_PASS` with an actual value (not `${...}` or empty)
- `DIGITALOCEAN_ACCESS_TOKEN` with a hex value
- `doctl auth` tokens
- Private IP addresses like `10.116.0.2` in committed code (ok in .do/ config, not ok in source)

Then BLOCK the commit and warn:
> **Potential secret detected in staged files.** Review the diff carefully before committing. Use `git diff --cached` to inspect.
