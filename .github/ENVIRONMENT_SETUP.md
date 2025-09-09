# GitHub Environment Configuration

To enable production deployments, you need to configure GitHub environments and secrets.

## 1. Create Production Environment

In your GitHub repository:

1. Go to **Settings** → **Environments**
2. Click **New environment**
3. Name: `production`
4. Configure protection rules:
   - ✅ **Required reviewers**: Add yourself or team members
   - ✅ **Wait timer**: 0 minutes (can be increased for additional safety)
   - ✅ **Deployment branches**: Only `main` branch

## 2. Configure Repository Secrets

Add these secrets in **Settings** → **Secrets and variables** → **Actions**:

### Required for Deployment
```bash
DIGITALOCEAN_ACCESS_TOKEN  # DigitalOcean API token with App Platform access
```

### Required for Database
```bash
SURREAL_ADDRESS    # SurrealDB server URL (e.g., https://your-db.domain.com:8000)
SURREAL_NS         # SurrealDB namespace (e.g., production)
SURREAL_DB         # SurrealDB database name (e.g., alexthola_blog)
SURREAL_USERNAME   # SurrealDB username (e.g., admin)
SURREAL_PASSWORD   # SurrealDB password (use strong password)
```

## 3. DigitalOcean App Platform Setup

### Create API Token
1. Log into DigitalOcean
2. Go to **API** → **Tokens/Keys**
3. Click **Generate New Token**
4. Name: `GitHub Actions Deploy`
5. Scopes: **Write** (for deployment operations)
6. Copy token and add to GitHub secrets as `DIGITALOCEAN_ACCESS_TOKEN`

### Prepare App Platform
1. Create app using the `.do/app.yaml` spec file
2. Configure custom domain `alexthola.com`
3. Set up SSL certificates
4. Configure environment variables

## 4. Database Setup Options

### Option A: DigitalOcean Managed Database (Recommended)
```bash
# Create managed PostgreSQL database
doctl databases create blog-db --engine pg --version 15 --size db-s-1vcpu-1gb --region nyc3

# Get connection details for secrets
doctl databases connection blog-db
```

### Option B: Self-hosted SurrealDB
```bash
# Deploy SurrealDB container
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

Once configured, deployments will only proceed if:

- [ ] All required secrets are configured
- [ ] Domain `alexthola.com` is accessible via DNS
- [ ] DigitalOcean API token has proper permissions
- [ ] Database server is reachable
- [ ] GitHub environment protection passes (if configured)

## 6. Troubleshooting

### Deployment Skipped
- Check that all required secrets are set in repository settings
- Verify `DIGITALOCEAN_ACCESS_TOKEN` has App Platform permissions
- Confirm database connectivity from the deployment environment

### Domain Not Accessible
- Check DNS records point to DigitalOcean
- Allow up to 48 hours for DNS propagation
- Verify domain ownership in DigitalOcean App Platform

### Database Connection Issues
- Test database connectivity manually
- Check firewall rules allow connections from DigitalOcean IPs
- Verify username/password combination

## 7. Security Best Practices

- Use separate database credentials for production vs staging
- Regularly rotate API tokens and database passwords
- Enable two-factor authentication on DigitalOcean account
- Monitor deployment logs for security issues
- Use GitHub environment protection for production deployments