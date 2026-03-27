---
name: deployment-debugging
description: Use when diagnosing DigitalOcean App Platform deployment failures, CSS/asset issues, SurrealDB connectivity problems, or Docker build OOM errors
---

# Blog Deployment Debugging

## Architecture

- **App Platform**: Runs the Leptos/Axum server from a Dockerfile
- **SurrealDB**: Runs on a separate droplet (private IP in `10.116.0.0/20` VPC, check `doctl compute droplet list` for current IPs)
- **Static assets**: CSS/JS/WASM in `target/site/pkg/`, served via `ServeDir` at `/pkg`
- **CSS hashing**: `hash-files = true` in Cargo.toml; `resolve_css_href()` in `app/src/lib.rs` reads `hash.txt`

## Common Issues

### CSS 404 / MIME type error
The browser error "MIME type ('') is not a supported stylesheet MIME type" means the CSS file returns 404.
- **Cause**: `hash-files` not set in Cargo.toml → `options.hash_files = false` at runtime → unhashed URL `/pkg/blog.css` → file is actually `blog.HASH.css` → 404
- **Check**: `curl -s SITE_URL/ | grep -oP 'href="[^"]*\.css[^"]*"'` — should show hashed filename
- **Fix**: Ensure `hash-files = true` in `[package.metadata.leptos]` section of Cargo.toml

### Build OOM on DO App Platform
- **Symptom**: `BuildJobTerminated` / "resource exhaustion" in deployment details
- **Check**: `doctl apps get-deployment APP_ID DEPLOY_ID --output json | python3 -c "import json,sys; print(json.load(sys.stdin)[0]['progress']['summary_steps'])"`
- **Fix**: `CARGO_BUILD_JOBS=2` in the Dockerfile limits parallel rustc instances

### SurrealDB connection failure
- **Symptom**: App health returns 200 but main page returns 503 "starting up"
- **Cause**: Two-phase startup — bootstrap router serves `/health`, main router activates only after SurrealDB connects
- **VPC requirement**: Source-built apps (Dockerfile on DO) get VPC access; DOCR image deploys do NOT
- **Check connectivity**: SSH to droplet, run `tcpdump -i eth1 port 8000 -nn` to see if packets arrive

### Secrets lost after spec update
- **Symptom**: Logs show `http://${SURREAL_ADDRESS}` literally
- **Cause**: `doctl apps update --spec` with `value: ${VAR}` overwrites secrets with literal placeholder
- **Fix**: Omit `value` field for SECRET type envs to preserve existing values, or inject actual values via python script before `doctl apps update`

## Key Files
- `app/src/lib.rs` — `resolve_css_href()`, `build_css_href()`, `shell()`
- `server/src/main.rs` — Router setup, static file serving, health endpoint
- `server/src/utils.rs` — SurrealDB connection logic, env var parsing
- `.do/app.yaml` — DO App Platform spec
- `Dockerfile` — Multi-stage build (builder → runner)
- `.github/workflows/deploy.yml` — CI/CD deployment pipeline
