# API Reference

This document lists the consumer-facing HTTP endpoints exposed by this service.

## Base URL

- Production: `https://<your-domain>`

## Health & Feeds

### `GET /health`

Returns basic service status (used by DigitalOcean health checks).

```bash
curl -sS https://<your-domain>/health | jq
```

### `GET /rss` and `GET /rss.xml`

Returns the RSS feed.

```bash
curl -sS https://<your-domain>/rss.xml
```

### `GET /sitemap.xml`

Returns the sitemap.

```bash
curl -sS https://<your-domain>/sitemap.xml
```

## API (server functions)

These endpoints are implemented via Leptos server functions. By default they use:

- `POST` requests with URL-encoded inputs (`application/x-www-form-urlencoded`)
- `JSON` responses (`application/json`)

### `POST /api/posts`

Fetch published posts, optionally filtered by tag.

- Inputs: `tags` (repeatable; optional)

```bash
# All posts
curl -sS -X POST https://<your-domain>/api/posts

# Filter by tags
curl -sS -X POST \
  -H 'content-type: application/x-www-form-urlencoded' \
  --data-urlencode 'tags=rust' \
  --data-urlencode 'tags=web-dev' \
  https://<your-domain>/api/posts
```

### `POST /api/tags`

Fetch all tags and counts.

```bash
curl -sS -X POST https://<your-domain>/api/tags
```

### `POST /api/post`

Fetch a single post by slug.

- Inputs: `slug` (required)

```bash
curl -sS -X POST \
  -H 'content-type: application/x-www-form-urlencoded' \
  --data-urlencode 'slug=my-post-slug' \
  https://<your-domain>/api/post
```

### `POST /api/increment_views`

Increment view counter for a post.

- Inputs: `id` (required; post record id suffix)

```bash
curl -sS -X POST \
  -H 'content-type: application/x-www-form-urlencoded' \
  --data-urlencode 'id=post-id' \
  https://<your-domain>/api/increment_views
```

### `POST /api/references`

Fetch published references.

```bash
curl -sS -X POST https://<your-domain>/api/references
```

### `POST /api/contact`

Send a contact message (requires SMTP env vars on the server).

- Inputs: `name`, `email`, `subject`, `message`

```bash
curl -sS -X POST \
  -H 'content-type: application/x-www-form-urlencoded' \
  --data-urlencode 'name=Jane Doe' \
  --data-urlencode 'email=jane@example.com' \
  --data-urlencode 'subject=Hello' \
  --data-urlencode 'message=Hi there!' \
  https://<your-domain>/api/contact
```

### `GET /api/activities?page=<n>`

Fetch activities (paginated; 0-indexed).

```bash
curl -sS 'https://<your-domain>/api/activities?page=0'
```

### `POST /api/activities`

Create an activity.

> Note: This uses Leptos server-function encoding for the `activity` payload.

## Deprecated (redirected)

The following older endpoints are retained as redirects to the `/api/*` equivalents:

- `/posts`, `/tags`, `/post`, `/increment_views`, `/contact`, `/references`
- `/api/activities/create` → `/api/activities`
