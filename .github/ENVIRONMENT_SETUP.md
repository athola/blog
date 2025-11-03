# GitHub Environment Configuration

This guide explains how to configure GitHub environments and secrets for production deployments.

## 1. Create Production Environment

In your GitHub repository:

1.  Go to **Settings** → **Environments**.
2.  Click **New environment**.
3.  Name it `production`.
4.  Configure protection rules:
    -   **Required reviewers**: Add yourself or team members.
    -   **Wait timer**: 0 minutes.
    -   **Deployment branches**: Only the `main` branch.

## 2. Configure Repository Secrets

Add these secrets in **Settings** → **Secrets and variables** → **Actions**:

### Required for Deployment

```bash
DIGITALOCEAN_ACCESS_TOKEN  # DigitalOcean API token with App Platform access
```

### Required for Database

```bash
SURREAL_ADDRESS    # SurrealDB server URL
SURREAL_NS         # SurrealDB namespace
SURREAL_DB         # SurrealDB database name
SURREAL_USERNAME   # SurrealDB username
SURREAL_PASSWORD   # SurrealDB password
```

## 3. DigitalOcean App Platform Setup

### Create API Token

1.  Log into DigitalOcean.
2.  Go to **API** → **Tokens/Keys**.
3.  Click **Generate New Token**.
4.  Name it `GitHub Actions Deploy`.
5.  Give it **Write** scope.
6.  Copy the token and add it to GitHub secrets as `DIGITALOCEAN_ACCESS_TOKEN`.

### Prepare App Platform

1.  Create an app using the `.do/app.yaml` spec file.
2.  Configure the custom domain `alexthola.com`.
3.  Set up SSL certificates.
4.  Configure environment variables.

## 4. Database Setup Options

### Option A: DigitalOcean Managed Database

```bash
# Create a managed PostgreSQL database
doctl databases create blog-db --engine pg --version 15 --size db-s-1vcpu-1gb --region nyc3

# Get connection details for secrets
doctl databases connection blog-db
```

### Option B: Self-hosted SurrealDB

```bash
# Deploy a SurrealDB container
docker run -d --name surrealdb \
  -p 8000:8000 \
  -v /opt/surrealdb:/data \
  --restart unless-stopped \
  surrealdb/surrealdb:latest start \
  --user admin \
  --pass YOUR_SECURE_PASSWORD \
  file:/data/database.db
```

## 5. Deployment Verification Checklist

Deployments will only proceed if:

- [ ] All required secrets are configured.
- [ ] The domain `alexthola.com` is accessible via DNS.
- [ ] The DigitalOcean API token has the correct permissions.
- [ ] The database server is reachable.
- [ ] GitHub environment protection rules pass.

## 6. Troubleshooting

*   **Deployment Skipped**: Check that all required secrets are set and that the `DIGITALOCEAN_ACCESS_TOKEN` has App Platform permissions.
*   **Domain Not Accessible**: Check DNS records and allow time for propagation.
*   **Database Connection Issues**: Test connectivity manually and check firewall rules.

## 7. Security Best Practices

- Use separate database credentials for production and staging.
- Regularly rotate API tokens and database passwords.
- Enable two-factor authentication on your DigitalOcean account.
- Monitor deployment logs.
- Use GitHub environment protection for production deployments.
