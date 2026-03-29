---
name: validate-assets
description: Verify CSS/JS/WASM asset hashing and serving on the deployed site
---

Validate that static assets are correctly hashed and served on the deployed blog.

First, resolve the app URL:
```bash
APP_URL=$(doctl apps list --format DefaultIngress,Spec.Name --no-header | grep alexthola-blog | awk '{print $1}')
```

1. Fetch the homepage HTML and extract CSS/JS/WASM links:
```bash
curl -s "$APP_URL/" 2>&1 | grep -oP 'href="[^"]*\.(css|js|wasm)[^"]*"'
```

2. For each asset URL found, check the HTTP status and Content-Type:
```bash
# Example for CSS
curl -sI "$APP_URL/pkg/blog.HASH.css" | head -5
```

3. Check the local hash.txt to compare:
```bash
cat target/release/hash.txt 2>/dev/null || echo "No local hash.txt (run cargo leptos build first)"
```

Report whether:
- CSS link in HTML uses a hashed filename (e.g., `blog.HASH.css` not `blog.css`)
- All asset URLs return HTTP 200 with correct MIME types
- `text/css` for CSS, `text/javascript` for JS, `application/wasm` for WASM
