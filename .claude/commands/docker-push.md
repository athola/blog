---
name: docker-push
description: Build the Docker image and push to DigitalOcean Container Registry
args: tag
---

Build the Docker image from the current branch and push to DOCR.

Tag argument `$ARGUMENTS` defaults to `latest`.

```bash
TAG="${ARGUMENTS:-latest}"
```

Steps:
1. Login to DOCR: `doctl registry login --expiry-seconds 3600`
2. Build: `docker build --platform linux/amd64 -t registry.digitalocean.com/athola-blog/blog-web:$TAG .`
3. Push: `docker push registry.digitalocean.com/athola-blog/blog-web:$TAG`

The Rust build takes ~15-20 minutes. Run the build in the background and notify when complete.

After pushing, report the image digest and size.
