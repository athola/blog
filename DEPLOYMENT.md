# DigitalOcean Deployment Guide for alexthola.com

This guide walks through deploying the Rust/Leptos blog application to DigitalOcean App Platform with custom domain setup.

## Prerequisites

1. **DigitalOcean Account** with billing enabled
2. **Domain ownership** of `alexthola.com` 
3. **GitHub repository** with proper access permissions
4. **SurrealDB instance** (managed or self-hosted)

## Step-by-Step Deployment

### 1. Prepare Your Repository

Ensure all deployment files are committed:

```bash
git add .do/ Dockerfile DEPLOYMENT.md .env.production
git commit -m "Add DigitalOcean deployment configuration"
git push origin main
```

### 2. Create DigitalOcean App

#### Option A: Using the Web Interface

1. Log into [DigitalOcean Control Panel](https://cloud.digitalocean.com/apps)
2. Click **"Create App"**
3. Choose **"GitHub"** as source
4. Select repository: `athola/blog`
5. Choose branch: `main`
6. Upload the app spec: `.do/app.yaml`

#### Option B: Using doctl CLI

```bash
# Install doctl if not already installed
curl -sL https://github.com/digitalocean/doctl/releases/download/v1.100.0/doctl-1.100.0-linux-amd64.tar.gz | tar -xzv
sudo mv doctl /usr/local/bin

# Authenticate
doctl auth init

# Create the app
doctl apps create --spec .do/app.yaml
```

### 3. Configure Database

#### Option A: Managed PostgreSQL (Recommended for production)

1. In DigitalOcean control panel, go to **Databases**
2. Create a new **PostgreSQL** cluster:
   - Name: `blog-database`
   - Version: `15`
   - Size: `db-s-1vcpu-1gb` (can scale later)
   - Region: Same as your app (e.g., `nyc3`)
3. Add your app to the database's trusted sources
4. Update environment variables in App Platform with connection details

#### Option B: Self-hosted SurrealDB

```bash
# On your database server
docker run --rm --pull always \
  -p 8000:8000 \
  -v /path/to/data:/data \
  surrealdb/surrealdb:latest start \
  --log info \
  --user root \
  --pass YOUR_SECURE_PASSWORD \
  file:/data/database.db
```

### 4. Configure Environment Variables

In DigitalOcean App Platform, set these environment variables:

```bash
# Required for application
RUST_LOG=info
LEPTOS_SITE_ADDR=0.0.0.0:8080
LEPTOS_SITE_ROOT=site
LEPTOS_HASH_FILES=true

# Database configuration
SURREAL_ADDRESS=http://your-database-url:8000
SURREAL_NS=production
SURREAL_DB=alexthola_blog
SURREAL_USERNAME=admin
SURREAL_PASSWORD=your-secure-password  # Mark as secret
```

### 5. Domain Configuration

#### Set Up DNS Records

In your domain registrar (e.g., Namecheap, GoDaddy), set these DNS records:

```dns
# A Records pointing to DigitalOcean
A     @           your-app-ip-address
A     www         your-app-ip-address

# Or use CNAME if you prefer
CNAME @           your-app.ondigitalocean.app
CNAME www         your-app.ondigitalocean.app
```

#### Configure Custom Domain in DigitalOcean

1. Go to your app in DigitalOcean App Platform
2. Click **"Settings"** → **"Domains"**
3. Click **"Add Domain"**
4. Enter: `alexthola.com`
5. Add another domain: `www.alexthola.com` (as alias)
6. DigitalOcean will automatically provision SSL certificates

### 6. Database Migrations

Migrations will run automatically as a pre-deploy job. To run manually:

```bash
# If using SurrealDB
surreal import --conn $SURREAL_ADDRESS \
  --user $SURREAL_USERNAME \
  --pass $SURREAL_PASSWORD \
  --ns $SURREAL_NS \
  --db $SURREAL_DB \
  ./migrations/
```

### 7. Deployment Verification

Once deployed, verify the application:

```bash
# Health check
curl -f https://alexthola.com/health

# Expected response:
{
  "status": "healthy",
  "timestamp": "2025-01-XX...",
  "service": "blog-api",
  "version": "0.1.0"
}

# Check main site
curl -I https://alexthola.com
# Expected: 200 OK with proper headers
```

## DigitalOcean Usage Tiers and Costs

### App Platform Tiers

DigitalOcean App Platform offers several container tiers based on CPU, memory, and autoscaling capabilities:

#### Shared CPU Containers (No Autoscaling)
| Tier | CPU | Memory | Transfer | Monthly Cost |
|------|-----|--------|----------|--------------|
| Basic | 1 vCPU (Fixed) | 512 MiB | 50 GiB | $5.00 |
| Basic | 1 vCPU (Fixed) | 1 GiB | 100 GiB | $10.00 |
| Basic | 1 vCPU | 1 GiB | 150 GiB | $12.00 |
| Basic | 1 vCPU | 2 GiB | 200 GiB | $25.00 |
| Basic | 2 vCPUs | 4 GiB | 250 GiB | $50.00 |

#### Dedicated CPU Containers (With Autoscaling)
| Tier | CPU | Memory | Transfer | Autoscaling | Monthly Cost |
|------|-----|--------|----------|-------------|--------------|
| Professional | 1 vCPU | 512 MiB | 100 GiB | Yes | $29.00 |
| Professional | 1 vCPU | 1 GiB | 200 GiB | Yes | $34.00 |
| Professional | 1 vCPU | 2 GiB | 300 GiB | Yes | $39.00 |
| Professional | 1 vCPU | 4 GiB | 400 GiB | Yes | $49.00 |
| Professional | 2 vCPUs | 4 GiB | 500 GiB | Yes | $78.00 |
| Professional | 2 vCPUs | 8 GiB | 600 GiB | Yes | $98.00 |
| Professional | 4 vCPUs | 8 GiB | 700 GiB | Yes | $156.00 |
| Professional | 4 vCPUs | 16 GiB | 800 GiB | Yes | $196.00 |
| Professional | 8 vCPUs | 32 GiB | 900 GiB | Yes | $392.00 |

### Database Tiers

DigitalOcean offers managed PostgreSQL databases with the following typical tiers:

| Tier | CPU | Memory | Storage | Monthly Cost | Notes |
|------|-----|--------|---------|--------------|-------|
| Basic | 1 vCPU | 1 GiB | 10 GiB | $15/month | Suitable for low-traffic blogs |
| Basic | 1 vCPU | 2 GiB | 25 GiB | $25/month | Good for moderate traffic |
| Basic | 2 vCPUs | 4 GiB | 50 GiB | $50/month | Suitable for higher traffic |
| Professional | 2 vCPUs | 8 GiB | 100 GiB | $100/month | For high-traffic sites |
| Premium | 4 vCPUs | 16 GiB | 200 GiB | $200/month | For very high traffic |

### Additional Costs

- **Outbound Transfer**: $0.01-0.02 per GiB for additional transfer beyond plan limits
- **Dedicated Egress IPs**: $25.00/month per app (if needed)

## Recommended Configuration for Low-Traffic Blog

For a low-traffic blog that needs to handle potential spikes from Hacker News front page placement:

### Baseline Configuration
- **App Platform**: Basic tier with 1 vCPU and 1 GiB memory ($12/month)
- **Database**: Basic PostgreSQL with 1 vCPU and 1 GiB memory ($15/month)
- **Total Baseline Cost**: $27/month

### Spike Handling Configuration
- **App Platform**: Professional tier with 1 vCPU and 2 GiB memory with autoscaling ($39/month)
- **Database**: Basic PostgreSQL with 2 vCPUs and 4 GiB memory ($50/month)
- **Total Spike-Ready Cost**: $89/month

### Why This Configuration?

1. **App Platform Professional Tier**:
   - Allows autoscaling to handle traffic spikes
   - Dedicated CPU ensures consistent performance
   - 2 GiB memory provides adequate space for caching

2. **Database Upgrade**:
   - 2 vCPUs and 4 GiB memory can handle increased query load
   - Larger instance can sustain higher concurrent connections

3. **Cost-Effectiveness**:
   - Baseline configuration is affordable for normal traffic
   - Spike configuration provides headroom for unexpected popularity
   - Can be scaled down after traffic returns to normal

### Traffic Spike Handling

When your blog gets posted to Hacker News front page:

1. **Autoscaling**:
   - DigitalOcean App Platform will automatically add more containers
   - CPU-based autoscaling triggers when CPU usage exceeds 80% for 5 minutes
   - Can scale up to 10x your baseline capacity

2. **Database Considerations**:
   - Monitor database connection count and CPU usage
   - Consider temporarily upgrading to a larger database tier
   - Use database connection pooling in your application

3. **Performance Optimization**:
   - Leverage HTTP compression (already implemented in your app)
   - Ensure static assets are served efficiently
   - Consider implementing a CDN for global distribution

## Production Checklist

- [ ] SSL certificate is active (green padlock in browser)
- [ ] Health endpoint returns 200 status
- [ ] Database migrations completed successfully
- [ ] All static assets loading correctly
- [ ] RSS feed accessible at `/rss.xml`
- [ ] Sitemap accessible at `/sitemap.xml`
- [ ] www.alexthola.com redirects to alexthola.com
- [ ] Application logs show no errors
- [ ] Performance metrics within acceptable ranges

## Monitoring and Maintenance

### Application Metrics

DigitalOcean App Platform provides built-in monitoring for:
- CPU and memory usage
- Request volume and response times
- Error rates
- Database connections

### Alerts Configuration

The app spec includes alerts for:
- CPU > 80% for 5 minutes
- Memory > 80% for 5 minutes  
- Restart count > 3 in 5 minutes

### Log Monitoring

Access logs via:
```bash
doctl apps logs your-app-id --type=run --follow
```

### Scaling

To handle increased traffic:

1. **Vertical Scaling**: Increase instance size in App Platform
2. **Horizontal Scaling**: Increase instance count (available with dedicated CPU tiers)
3. **Database Scaling**: Upgrade database cluster
4. **CDN**: Enable DigitalOcean's built-in CDN for static assets

## Troubleshooting

### Common Issues

#### App fails to start

1. Check build logs for compilation errors
2. Verify all environment variables are set
3. Ensure database is accessible

```bash
# View build logs
doctl apps logs your-app-id --type=build

# View runtime logs  
doctl apps logs your-app-id --type=run
```

#### Database connection issues

1. Verify database URL and credentials
2. Check network connectivity
3. Ensure database accepts connections from app

#### SSL certificate issues

1. Verify DNS records point to DigitalOcean
2. Wait for DNS propagation (up to 48 hours)
3. Check domain ownership verification

#### Performance issues

1. Monitor resource usage
2. Check database query performance
3. Verify CDN configuration for static assets

### Support Resources

- [DigitalOcean App Platform Documentation](https://docs.digitalocean.com/products/app-platform/)
- [SurrealDB Documentation](https://surrealdb.com/docs)
- [Leptos Documentation](https://leptos.dev/)

## Cost Optimization

### Current Configuration Costs (Estimated)

- **App Platform**: $5-12/month (basic-xxs instance)
- **Database**: $15-25/month (managed PostgreSQL)  
- **Bandwidth**: $0.01/GB
- **Domain**: Included with App Platform

### Cost Reduction Tips

1. Use smaller instance sizes during development
2. Enable auto-scaling to handle traffic spikes efficiently
3. Optimize WASM bundle size to reduce bandwidth costs
4. Use database connection pooling to reduce database load

## Security Considerations

### Implemented Security Measures

- [x] Non-root container execution
- [x] Distroless base image for minimal attack surface
- [x] Environment variable encryption
- [x] HTTPS enforcement with TLS 1.2+
- [x] Security headers in responses
- [x] Input validation and sanitization

### Additional Security Recommendations

1. **Enable VPC**: Isolate application network traffic
2. **Database Encryption**: Enable encryption at rest
3. **Audit Logging**: Monitor access patterns  
4. **Rate Limiting**: Prevent abuse and DDoS
5. **WAF**: Consider adding Web Application Firewall

---

**Need Help?** Open an issue in the GitHub repository or consult the troubleshooting section above.