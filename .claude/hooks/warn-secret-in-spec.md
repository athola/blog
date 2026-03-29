---
name: warn-secret-in-spec
description: Warn when editing .do/app.yaml with secret placeholder values that will be sent literally to DO
hooks:
  - event: PreToolUse
    tools: [Edit, Write]
    pattern: ".do/app.yaml"
---

When editing `.do/app.yaml`, check if any env var values contain `${...}` placeholder syntax with `type: SECRET`.

If found, warn:
> **Secret placeholder detected.** Values like `${SURREAL_ADDRESS}` in the app spec will be sent literally to DigitalOcean — they are NOT resolved from environment variables. Either:
> 1. Omit the `value` field to preserve existing secrets
> 2. Inject actual values via the deploy script before `doctl apps update`
