# Complete DigitalOcean Deployment Guide for alexthola.com

This comprehensive guide walks you through deploying the Rust/Leptos blog as a WASM application to DigitalOcean App Platform with your NameCheap domain.

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Architecture Overview](#architecture-overview)
3. [Part 1: DigitalOcean Setup](#part-1-digitalocean-setup)
4. [Part 2: Database Configuration](#part-2-database-configuration)
5. [Part 3: Domain Configuration with NameCheap](#part-3-domain-configuration-with-namecheap)
6. [Part 4: Application Deployment](#part-4-application-deployment)
7. [Part 5: Security Hardening](#part-5-security-hardening)
8. [Part 6: Monitoring and Maintenance](#part-6-monitoring-and-maintenance)
9. [Troubleshooting](#troubleshooting)
10. [Cost Estimation](#cost-estimation)

---

## Prerequisites

Before starting, ensure you have:

- **DigitalOcean Account**: With billing enabled and at least $15 credit
- **Domain**: alexthola.com registered with NameCheap
- **GitHub Account**: With repository access to athola/blog
- **Local Development**: Git, Rust, and cargo-leptos installed
- **doctl CLI** (optional but recommended): For command-line management

### Required Secrets/Credentials

Prepare these values (you'll need them later):
- DigitalOcean Personal Access Token
- SurrealDB credentials (username, password, namespace, database name)
- GitHub Personal Access Token (for automated deployments)

---

## Architecture Overview

### Deployment Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     alexthola.com (NameCheap)                │
│                     DNS A/CNAME Records                       │
└─────────────────────────┬────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│              DigitalOcean App Platform                       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  SSL/TLS Termination (Auto-managed)                    │  │
│  └──────────────────────┬─────────────────────────────────┘  │
│                         │                                     │
│  ┌──────────────────────▼─────────────────────────────────┐  │
│  │  Rust/Leptos WASM Application (blog-web)              │  │
│  │  - Axum Web Server (Port 8080)                        │  │
│  │  - WASM Frontend (compiled from Leptos)               │  │
│  │  - Security Headers Middleware                         │  │
│  │  - Rate Limiting                                       │  │
│  └──────────────────────┬─────────────────────────────────┘  │
└─────────────────────────┼─────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│              SurrealDB Database Instance                     │
│  - Managed or Self-hosted on DigitalOcean Droplet            │
│  - Version: 3.0.0-alpha.10 or later                          │
│  - Automatic backups enabled                                 │
└──────────────────────────────────────────────────────────────┘
```

### Technology Stack
- **Frontend**: Leptos framework compiled to WASM (~150KB gzipped)
- **Backend**: Axum web server with SSR support
- **Database**: SurrealDB 3.0.0-alpha.10+
- **CDN/Edge**: DigitalOcean's global edge network
- **SSL/TLS**: Auto-managed Let's Encrypt certificates

---

## Part 1: DigitalOcean Setup

### Step 1.1: Create DigitalOcean Account

1. Navigate to https://cloud.digitalocean.com/registrations/new
2. Sign up using email or GitHub authentication
3. Verify your email address
4. Add a payment method (credit card or PayPal)
5. Optionally apply any promotional credits

### Step 1.2: Generate DigitalOcean API Token

1. Log into DigitalOcean Control Panel
2. Click **API** in the left sidebar (or visit https://cloud.digitalocean.com/account/api/tokens)
3. Click **Generate New Token**
4. Configure the token:
   - **Name**: `blog-deployment`
   - **Scopes**: Select both Read and Write
   - **Expiration**: Choose based on your security policy (90 days recommended)
5. Click **Generate Token**
6. **CRITICAL**: Copy the token immediately and store it securely (you won't see it again)
7. Add this to GitHub Secrets as `DIGITALOCEAN_ACCESS_TOKEN`

### Step 1.3: Install doctl CLI (Optional but Recommended)

**On macOS:**
```bash
brew install doctl
```

**On Linux:**
```bash
cd ~
wget https://github.com/digitalocean/doctl/releases/download/v1.100.0/doctl-1.100.0-linux-amd64.tar.gz
tar xf doctl-1.100.0-linux-amd64.tar.gz
sudo mv doctl /usr/local/bin
```

**Authenticate doctl:**
```bash
doctl auth init
# Paste your DigitalOcean API token when prompted
doctl auth list  # Verify authentication
```

---

## Part 2: Database Configuration

You have two options for SurrealDB: managed PostgreSQL with SurrealDB on top, or self-hosted SurrealDB.

### Option A: Self-Hosted SurrealDB on DigitalOcean Droplet (Recommended)

This provides full control and uses the latest SurrealDB version.

#### Step 2A.1: Create a Droplet

1. In DigitalOcean Control Panel, click **Create** → **Droplets**
2. Choose configuration:
   - **Region**: New York (NYC3) - Same as your app for low latency
   - **Image**: Ubuntu 22.04 LTS x64
   - **Plan**: Basic
   - **Size**: $12/mo (2 GB RAM, 1 vCPU, 50 GB SSD) - Minimum for production
   - **VPC**: Create new or use default
3. Add SSH key or use password authentication
4. **Hostname**: `surrealdb-production`
5. Click **Create Droplet**
6. Note the droplet's IP address (e.g., `157.230.xxx.xxx`)

#### Step 2A.2: Install and Configure SurrealDB

SSH into your droplet:
```bash
ssh root@YOUR_DROPLET_IP
```

Install SurrealDB:
```bash
# Install SurrealDB
curl -sSf https://install.surrealdb.com | sh

# Verify installation
surreal version
# Should show: surrealdb 3.0.0-alpha.10 or later
```

Create a systemd service for SurrealDB:
```bash
# Create SurrealDB user
useradd -r -s /bin/false surrealdb

# Create data directory
mkdir -p /var/lib/surrealdb
chown surrealdb:surrealdb /var/lib/surrealdb
chmod 700 /var/lib/surrealdb

# Create systemd service file
cat > /etc/systemd/system/surrealdb.service << 'EOF'
[Unit]
Description=SurrealDB Database
After=network.target

[Service]
Type=simple
User=surrealdb
Group=surrealdb
ExecStart=/root/.surrealdb/surreal start \
  --bind 0.0.0.0:8000 \
  --user root \
  --pass CHANGE_THIS_SECURE_PASSWORD \
  --log info \
  file:/var/lib/surrealdb/data.db

Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=surrealdb

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/surrealdb

[Install]
WantedBy=multi-user.target
EOF
```

**CRITICAL**: Replace `CHANGE_THIS_SECURE_PASSWORD` with a strong password (e.g., use `openssl rand -base64 32`)

Start SurrealDB:
```bash
# Enable and start the service
systemctl daemon-reload
systemctl enable surrealdb
systemctl start surrealdb

# Check status
systemctl status surrealdb

# View logs
journalctl -u surrealdb -f
```

#### Step 2A.3: Configure Firewall

```bash
# Allow SSH (if not already allowed)
ufw allow 22/tcp

# Allow SurrealDB only from App Platform
# Get your App Platform IP ranges from: https://docs.digitalocean.com/products/platform/
# For now, allow your app's VPC (you can restrict this further later)
ufw allow from YOUR_VPC_IP_RANGE to any port 8000

# Enable firewall
ufw enable
ufw status
```

#### Step 2A.4: Set Database Credentials

Store these securely for later use:
```bash
SURREAL_ADDRESS=http://YOUR_DROPLET_IP:8000
SURREAL_NS=production
SURREAL_DB=alexthola_blog
SURREAL_USERNAME=root
SURREAL_PASSWORD=YOUR_SECURE_PASSWORD  # From Step 2A.2
```

### Option B: Managed PostgreSQL (Fallback Option)

If you prefer managed infrastructure:

1. In DigitalOcean, click **Create** → **Databases**
2. Choose **PostgreSQL 15**
3. Select datacenter: **New York (NYC3)**
4. Choose plan: **Basic** ($15/mo - 1GB RAM, 1 vCPU, 10GB storage)
5. Name: `blog-postgres-production`
6. Click **Create Database**

**Note**: You'll need to run SurrealDB on top of PostgreSQL, which adds complexity. Option A is recommended.

---

## Part 3: Domain Configuration with NameCheap

### Step 3.1: Prepare Domain in NameCheap

1. Log into your NameCheap account at https://www.namecheap.com/myaccount/login/
2. Navigate to **Domain List**
3. Find **alexthola.com** and click **Manage**

### Step 3.2: Configure DNS Records

You have two approaches: **Approach A (Recommended)** using DigitalOcean nameservers, or **Approach B** using NameCheap DNS with CNAME records.

#### Approach A: Use DigitalOcean Nameservers (Recommended)

This gives you full DNS management in DigitalOcean.

**In NameCheap:**
1. On the alexthola.com management page, find **Nameservers** section
2. Select **Custom DNS**
3. Add DigitalOcean nameservers:
   ```
   ns1.digitalocean.com
   ns2.digitalocean.com
   ns3.digitalocean.com
   ```
4. Click **✓** (checkmark) to save
5. Wait 15-30 minutes for propagation

**In DigitalOcean:**
1. Go to **Networking** → **Domains**
2. Add domain: Enter `alexthola.com`
3. Click **Add Domain**
4. You'll configure A records later after app deployment

#### Approach B: Use NameCheap DNS with CNAME

If you want to keep DNS at NameCheap:

1. In NameCheap, keep nameservers as **NameCheap BasicDNS**
2. Go to **Advanced DNS** tab
3. You'll add CNAME records after app deployment (Step 4.4)

**Note**: We'll complete DNS records after deploying the app to get the DigitalOcean App Platform URL.

---

## Part 4: Application Deployment

### Step 4.1: Prepare GitHub Repository

1. Ensure your blog repository is pushed to GitHub:
```bash
git remote -v
# Should show: origin  https://github.com/athola/blog.git

# If needed, push latest changes
git push origin master
```

2. Verify deployment files exist:
```bash
ls -la .do/app.yaml Dockerfile DEPLOYMENT.md
```

### Step 4.2: Add GitHub Secrets

These secrets enable automated deployments via GitHub Actions.

1. Go to your GitHub repository: https://github.com/athola/blog
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret** for each of the following:

| Secret Name | Value | Example |
|-------------|-------|---------|
| `DIGITALOCEAN_ACCESS_TOKEN` | Your DO API token from Step 1.2 | `dop_v1_a1b2c3...` |
| `SURREAL_ADDRESS` | SurrealDB connection URL | `http://157.230.xxx.xxx:8000` |
| `SURREAL_NS` | SurrealDB namespace | `production` |
| `SURREAL_DB` | SurrealDB database name | `alexthola_blog` |
| `SURREAL_USERNAME` | Database username | `root` |
| `SURREAL_PASSWORD` | Database password | `[your secure password]` |

4. Verify all 6 secrets are added

### Step 4.3: Deploy Application to DigitalOcean

You have two deployment methods:

#### Method A: Using DigitalOcean Web Console (Easier for First Deploy)

1. Log into DigitalOcean Control Panel
2. Click **Apps** in the left sidebar
3. Click **Create App**
4. **Choose Source**:
   - Select **GitHub**
   - Authorize DigitalOcean to access your GitHub account
   - Select repository: `athola/blog`
   - Select branch: `master` (or `main`)
   - **Autodeploy**: Enable (deploys on every push to master)
5. Click **Next**

6. **Configure Resources**:
   - DigitalOcean will detect the Dockerfile
   - Edit the web service:
     - **Name**: `blog-web`
     - **Region**: New York (NYC3)
     - **Instance Size**: Basic - $5/mo (512MB RAM)
     - **HTTP Port**: 8080
   - Click **Edit Plan** to select the $5 Basic plan
   - Click **Back** then **Next**

7. **Environment Variables**:
   Click **Edit** on blog-web, then **Environment Variables**:

   Add each variable:
   ```
   RUST_LOG=info
   LEPTOS_SITE_ADDR=0.0.0.0:8080
   LEPTOS_SITE_ROOT=site
   LEPTOS_HASH_FILES=true
   SURREAL_ADDRESS=[paste from secrets]
   SURREAL_NS=[paste from secrets]
   SURREAL_DB=[paste from secrets]
   SURREAL_USERNAME=[paste from secrets]
   SURREAL_PASSWORD=[paste from secrets - mark as ENCRYPTED]
   ```

   **IMPORTANT**: For `SURREAL_PASSWORD`, check "Encrypt" to hide it in logs

8. **Name Your App**:
   - App name: `alexthola-blog`
   - Region: New York 3
   - Click **Next**

9. **Review**:
   - Review all settings
   - Click **Create Resources**

10. **Wait for Build** (10-15 minutes):
    - Watch the build logs in the console
    - The app will go through: Building → Deploying → Active
    - Once **Active**, note the URL: `https://alexthola-blog-xxxxx.ondigitalocean.app`

#### Method B: Using doctl CLI (For Advanced Users)

```bash
# Create app from spec file
doctl apps create --spec .do/app.yaml

# Get app ID
doctl apps list

# Monitor deployment
doctl apps get YOUR_APP_ID

# View logs
doctl apps logs YOUR_APP_ID --type build --follow
```

### Step 4.4: Configure Custom Domain

Now that your app is deployed, configure the custom domain:

**Get your App's URL:**
1. In DigitalOcean Apps dashboard, find your app
2. Note the default URL: `alexthola-blog-xxxxx.ondigitalocean.app`

**Add Custom Domain in DigitalOcean:**
1. In your app's dashboard, click **Settings** tab
2. Click **Domains** in the left sidebar
3. Click **Add Domain**
4. Enter: `alexthola.com`
5. Click **Add Domain**
6. DigitalOcean will show DNS records you need to add:
   ```
   CNAME  alexthola.com  →  alexthola-blog-xxxxx.ondigitalocean.app
   ```
   OR
   ```
   A  alexthola.com  →  [IP address provided]
   ```
7. **Also add www subdomain**:
   - Click **Add Domain** again
   - Enter: `www.alexthola.com`
   - Choose "Redirect to" → `alexthola.com`

**Update DNS Records:**

**If using DigitalOcean nameservers (Approach A):**
1. Go to **Networking** → **Domains** → `alexthola.com`
2. Add A record:
   - **Hostname**: `@`
   - **Will Direct To**: Your app (select from dropdown)
   - **TTL**: 3600
3. Add CNAME record for www:
   - **Hostname**: `www`
   - **Is An Alias Of**: `@`
   - **TTL**: 3600

**If using NameCheap DNS (Approach B):**
1. In NameCheap, go to **Advanced DNS**
2. Delete any existing A records for `@` or `www`
3. Add new records:

   | Type | Host | Value | TTL |
   |------|------|-------|-----|
   | CNAME | @ | alexthola-blog-xxxxx.ondigitalocean.app | 300 |
   | CNAME | www | alexthola-blog-xxxxx.ondigitalocean.app | 300 |

4. Click **Save All Changes**

**Note**: Some registrars don't allow CNAME for root domain (@). If NameCheap rejects the CNAME for @, use the A record IP address provided by DigitalOcean instead.

### Step 4.5: Wait for DNS Propagation

DNS changes can take 15 minutes to 48 hours (usually 1-2 hours).

**Check propagation status:**
```bash
# Check nameservers (should show DigitalOcean or NameCheap)
dig NS alexthola.com +short

# Check A record
dig A alexthola.com +short

# Check CNAME for www
dig CNAME www.alexthola.com +short

# Alternative: use online tools
# https://dnschecker.org (enter alexthola.com)
# https://www.whatsmydns.net/#A/alexthola.com
```

### Step 4.6: SSL Certificate Provisioning

DigitalOcean automatically provisions Let's Encrypt SSL certificates.

1. In your app's **Domains** settings, watch for SSL status
2. Status will change from **Pending** → **Active** (5-30 minutes after DNS propagates)
3. Once **Active**, your site is accessible via HTTPS

**Verify SSL:**
```bash
curl -I https://alexthola.com
# Should return: HTTP/2 200

# Check certificate details
openssl s_client -connect alexthola.com:443 -servername alexthola.com < /dev/null 2>/dev/null | openssl x509 -noout -dates -issuer
# Should show: Let's Encrypt Authority
```

### Step 4.7: Initialize Database Schema

After your app is deployed, initialize the database:

**Option 1: Using Local Migration Script**
```bash
# Set environment variables
export SURREAL_ADDRESS="http://YOUR_DROPLET_IP:8000"
export SURREAL_NS="production"
export SURREAL_DB="alexthola_blog"
export SURREAL_USERNAME="root"
export SURREAL_PASSWORD="YOUR_SECURE_PASSWORD"

# Run migration (if you have a migration script)
./scripts/migrate.sh

# Or manually import using SurrealDB CLI
surreal sql --conn $SURREAL_ADDRESS --user $SURREAL_USERNAME --pass $SURREAL_PASSWORD --ns $SURREAL_NS --db $SURREAL_DB < migrations/schema.surql
```

**Option 2: Via App Platform Console**
```bash
# Using doctl to run a one-time job
doctl apps create-deployment YOUR_APP_ID
```

The GitHub Actions workflow includes a `db-migrate` job that runs automatically on deployment.

### Step 4.8: Verify Deployment

Test all endpoints:

```bash
# Health check
curl https://alexthola.com/health
# Expected: {"status":"healthy","timestamp":"...","service":"blog-api","version":"..."}

# Main page
curl -I https://alexthola.com/
# Expected: HTTP/2 200

# RSS feed
curl -I https://alexthola.com/rss.xml
# Expected: HTTP/2 200, Content-Type: application/rss+xml

# Sitemap
curl -I https://alexthola.com/sitemap.xml
# Expected: HTTP/2 200, Content-Type: application/xml

# WWW redirect
curl -I https://www.alexthola.com/
# Expected: HTTP/2 301, Location: https://alexthola.com/

# Test HTTPS redirect (App Platform handles this automatically)
curl -I http://alexthola.com/
# Expected: HTTP/2 301, Location: https://alexthola.com/
```

---

## Part 5: Security Hardening

### Step 5.1: Verify Security Headers

Check that security headers are present:

```bash
curl -I https://alexthola.com/
```

Expected headers:
```
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Content-Security-Policy: default-src 'self'; ...
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

If any are missing, they will be added by the security middleware (see Part 6 for implementation).

### Step 5.2: Configure App Platform Security Settings

In DigitalOcean App Platform:

1. Go to your app → **Settings** → **App-Level Security**
2. Enable:
   - **HTTPS Only**: Force HTTPS redirects
   - **HSTS**: HTTP Strict Transport Security (max-age: 31536000)

### Step 5.3: Database Security Checklist

- [ ] SurrealDB uses strong password (min 32 characters)
- [ ] Firewall restricts database access to app VPC only
- [ ] Database uses latest stable version
- [ ] Backups are enabled (manual or automated)
- [ ] Database credentials stored as encrypted secrets

**Enable Automated Backups for Droplet** (if using Option A):
1. Go to **Droplets** → Select your SurrealDB droplet
2. Click **Backups**
3. Enable backups (+20% of droplet cost = $2.40/mo for $12 droplet)

### Step 5.4: Enable Rate Limiting (Application Level)

Rate limiting is implemented in the application code (see security middleware updates).

### Step 5.5: Monitor and Update Dependencies

```bash
# Regular dependency audits (run locally)
cargo audit

# Update dependencies
cargo update

# Check for outdated packages
cargo outdated
```

Set up automated alerts:
1. GitHub Dependabot: Already enabled (see `.github/dependabot.yml`)
2. Rust Security Advisory Database: Integrated with `cargo audit`

### Step 5.6: Secret Management Best Practices

- **Never commit secrets** to git (already enforced by secret scanning)
- Use DigitalOcean's **encrypted environment variables** for all secrets
- Rotate credentials every 90 days:
  - Database passwords
  - API tokens
  - TLS certificates (auto-renewed by Let's Encrypt)

### Step 5.7: Enable Web Application Firewall (WAF)

DigitalOcean App Platform includes basic DDoS protection. For advanced WAF:

**Option A: Use Cloudflare (Free Tier)**
1. Sign up at https://www.cloudflare.com
2. Add alexthola.com to Cloudflare
3. Change nameservers to Cloudflare's (in NameCheap)
4. In Cloudflare, add DNS records pointing to your DO app
5. Enable:
   - SSL/TLS: Full (Strict)
   - Firewall: Security level High
   - Rate Limiting: 100 requests/minute per IP

**Option B: Stick with DigitalOcean** (Simpler, included in App Platform)

---

## Part 6: Monitoring and Maintenance

### Step 6.1: Application Monitoring

**DigitalOcean Built-in Metrics:**
1. Go to your app → **Insights**
2. Monitor:
   - CPU usage
   - Memory usage
   - Request rate
   - Response time
   - Error rate

**Set up Alerts:**
1. Go to **Settings** → **Alerts**
2. Configure alerts for:
   - CPU > 80% for 5 minutes
   - Memory > 80% for 5 minutes
   - Error rate > 5% for 5 minutes
   - Restart count > 3 in 5 minutes
3. Add notification email

### Step 6.2: Log Management

**View Logs:**
```bash
# Via doctl
doctl apps logs YOUR_APP_ID --type=run --follow

# Via web console
# Go to app → Runtime Logs
```

**Log Retention:**
- DigitalOcean keeps logs for 7 days
- For longer retention, export to external service:
  - Papertrail (free tier: 50MB/month)
  - Logtail
  - Self-hosted ELK stack

### Step 6.3: Database Backups

**For Droplet-hosted SurrealDB:**

Create automated backup script:
```bash
#!/bin/bash
# /root/backup-surrealdb.sh

BACKUP_DIR="/var/backups/surrealdb"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/backup_$DATE.surql"

mkdir -p $BACKUP_DIR

surreal export \
  --conn http://127.0.0.1:8000 \
  --user root \
  --pass YOUR_SECURE_PASSWORD \
  --ns production \
  --db alexthola_blog \
  $BACKUP_FILE

# Keep only last 30 days of backups
find $BACKUP_DIR -name "backup_*.surql" -mtime +30 -delete

# Optional: Upload to DigitalOcean Spaces
# s3cmd put $BACKUP_FILE s3://your-backup-bucket/
```

Add to crontab:
```bash
crontab -e

# Add line (runs daily at 2 AM):
0 2 * * * /root/backup-surrealdb.sh
```

### Step 6.4: Uptime Monitoring

**Option A: Use UptimeRobot** (Free)
1. Sign up at https://uptimerobot.com
2. Add monitor:
   - Type: HTTPS
   - URL: https://alexthola.com/health
   - Interval: 5 minutes
   - Alert contacts: Your email

**Option B: Use DigitalOcean Monitoring** (Included)
- Already configured in App Platform

### Step 6.5: Performance Monitoring

**Lighthouse CI** (Automated performance testing):
```bash
# Install Lighthouse CI
npm install -g @lhci/cli

# Run audit
lhci autorun --collect.url=https://alexthola.com
```

**Expected scores:**
- Performance: > 90
- Accessibility: > 95
- Best Practices: > 90
- SEO: > 95

### Step 6.6: Scaling Strategy

**Current Setup**: $5/mo app + $12/mo database = **$17/mo total**

**Traffic thresholds:**
| Traffic | Plan | Cost |
|---------|------|------|
| < 10K visitors/month | Basic $5 | $17/mo |
| 10K-50K visitors/month | Basic $10 (1GB RAM) | $22/mo |
| 50K-100K visitors/month | Professional $25 (2GB RAM) | $37/mo |
| 100K+ visitors/month | Professional $50 + CDN | $62/mo+ |

**Scaling triggers:**
- **CPU > 70% sustained**: Upgrade instance size
- **Response time > 500ms**: Add caching layer (Redis)
- **Database queries slow**: Add indexes, optimize queries
- **Global traffic**: Add Cloudflare CDN

**Horizontal Scaling** (when vertical scaling isn't enough):
1. In App Platform, increase **Instance Count** to 2-3
2. DigitalOcean automatically load-balances across instances
3. Ensure database can handle concurrent connections

### Step 6.7: Maintenance Schedule

| Task | Frequency | Estimated Time |
|------|-----------|----------------|
| Check monitoring alerts | Daily | 2 min |
| Review application logs | Weekly | 10 min |
| Update dependencies | Weekly | 20 min |
| Test backups | Monthly | 30 min |
| Security audit | Monthly | 1 hour |
| Rotate credentials | Quarterly | 30 min |
| Review and optimize costs | Quarterly | 1 hour |

---

## Troubleshooting

### Issue: Build Fails in DigitalOcean

**Symptoms**: Build process times out or errors during cargo build

**Solutions**:
1. Check build logs for specific errors
2. Verify Dockerfile syntax
3. Ensure Cargo.toml has correct dependencies
4. Try building locally first:
   ```bash
   docker build -t blog-test .
   docker run -p 8080:8080 blog-test
   ```
5. If build timeout (>15 min), optimize:
   - Use multi-stage builds (already in Dockerfile)
   - Cache dependencies
   - Use `.dockerignore` to exclude unnecessary files

### Issue: Application Fails to Start

**Symptoms**: Build succeeds but app crashes on startup

**Solutions**:
1. Check runtime logs:
   ```bash
   doctl apps logs YOUR_APP_ID --type=run --follow
   ```
2. Common causes:
   - Missing environment variables
   - Database connection failure
   - Port binding issues (ensure app listens on 0.0.0.0:8080)
3. Verify environment variables in App Platform settings
4. Test database connection:
   ```bash
   curl -X POST http://YOUR_DROPLET_IP:8000/sql \
     -H "Content-Type: application/json" \
     -d '{"sql":"INFO FOR DB;"}'
   ```

### Issue: Database Connection Errors

**Symptoms**: "Failed to connect to SurrealDB" in logs

**Solutions**:
1. Verify SurrealDB is running on droplet:
   ```bash
   ssh root@YOUR_DROPLET_IP
   systemctl status surrealdb
   journalctl -u surrealdb -n 50
   ```
2. Check firewall rules allow App Platform to access port 8000
3. Verify credentials in environment variables match SurrealDB config
4. Test connection from app's perspective:
   ```bash
   # From App Platform console (if available)
   curl http://YOUR_DROPLET_IP:8000/health
   ```

### Issue: Domain Not Resolving

**Symptoms**: `alexthola.com` returns "DNS_PROBE_FINISHED_NXDOMAIN"

**Solutions**:
1. Verify DNS propagation:
   ```bash
   dig alexthola.com +short
   ```
2. Check nameservers:
   ```bash
   dig NS alexthola.com +short
   ```
3. Wait 2-4 hours after DNS changes
4. Clear local DNS cache:
   ```bash
   # macOS
   sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder

   # Linux
   sudo systemd-resolve --flush-caches
   ```
5. Verify DNS records in DigitalOcean or NameCheap match expected values

### Issue: SSL Certificate Not Provisioning

**Symptoms**: Browser shows "Not Secure" or certificate error

**Solutions**:
1. Ensure DNS is fully propagated first (can take up to 48 hours)
2. In App Platform → Domains, check SSL status
3. If stuck on "Pending", try:
   - Remove and re-add the domain
   - Verify CAA records don't block Let's Encrypt
   ```bash
   dig CAA alexthola.com
   # Should be empty or allow Let's Encrypt
   ```
4. Let's Encrypt rate limits: 50 certs/week per domain
   - If exceeded, wait 7 days
   - Check status: https://crt.sh/?q=alexthola.com

### Issue: Slow Response Times

**Symptoms**: Pages load slowly (> 2 seconds)

**Solutions**:
1. Check App Platform metrics (CPU, Memory)
2. Analyze database queries (add indexes if needed)
3. Enable caching:
   - Browser caching (Cache-Control headers)
   - Add Redis for server-side caching
4. Optimize WASM bundle size:
   ```bash
   # Check bundle size
   ls -lh target/site/pkg/*.wasm

   # Should be < 500KB gzipped
   ```
5. Use Lighthouse to identify bottlenecks:
   ```bash
   lighthouse https://alexthola.com --view
   ```

### Issue: High Costs

**Symptoms**: Bill higher than expected

**Solutions**:
1. Review usage in DigitalOcean → Billing
2. Check for:
   - Unused droplets
   - Oversized app instances
   - Data transfer overages (usually included)
3. Optimize:
   - Downgrade app instance if CPU/memory usage is low
   - Use object storage (Spaces) for static assets
   - Enable compression (already configured)
4. Set billing alerts:
   - DigitalOcean → Billing → Set alerts for $25, $50

### Issue: Application Errors (500)

**Symptoms**: Specific endpoints return 500 Internal Server Error

**Solutions**:
1. Check application logs for stack traces
2. Enable more verbose logging:
   - Set `RUST_LOG=debug` temporarily
3. Test locally with production config:
   ```bash
   # Copy production env vars to .env.production
   cargo leptos build --release
   ./target/release/server
   ```
4. Common causes:
   - Database query errors
   - Missing data
   - Serialization issues

---

## Cost Estimation

### Monthly Cost Breakdown

| Service | Configuration | Cost |
|---------|--------------|------|
| **App Platform** | Basic (512MB RAM, 1 vCPU) | $5.00 |
| **SurrealDB Droplet** | 2GB RAM, 1 vCPU, 50GB SSD | $12.00 |
| **Droplet Backups** | 20% of droplet cost | $2.40 |
| **Bandwidth** | 1TB included (usually sufficient) | $0.00 |
| **Total** | | **$19.40/mo** |

### Optional Add-ons

| Service | Description | Cost |
|---------|-------------|------|
| **Monitoring** | Included in App Platform | $0.00 |
| **Spaces (Object Storage)** | 250GB storage + 1TB transfer | $5.00 |
| **Load Balancer** | For high availability | $12.00 |
| **Managed Redis** | For caching | $15.00 |
| **Reserved IPs** | Static IP address | $4.00 |

### Cost Optimization Tips

1. **Start small**: Begin with $5 app + $12 database = $17/mo
2. **Monitor usage**: Check CPU/memory weekly
3. **Scale gradually**: Upgrade only when metrics justify it
4. **Use free tiers**:
   - Cloudflare (CDN, DNS)
   - UptimeRobot (monitoring)
   - GitHub Actions (CI/CD - 2000 min/month free)
5. **Annual billing**: Some providers offer discounts for yearly payment

---

## Security Checklist

Before going live, verify:

- [ ] SSL/TLS certificate is active and valid
- [ ] HTTPS is enforced (HTTP redirects to HTTPS)
- [ ] Security headers are present (X-Frame-Options, CSP, HSTS, etc.)
- [ ] Database uses strong, unique password (min 32 characters)
- [ ] Database is firewall-protected (only app can access)
- [ ] All secrets are stored as encrypted environment variables
- [ ] Rate limiting is enabled
- [ ] Automated backups are configured
- [ ] Monitoring alerts are set up
- [ ] Dependencies are up to date (no known CVEs)
- [ ] Secret scanning passes (Gitleaks, Semgrep, TruffleHog)
- [ ] www subdomain redirects to apex domain
- [ ] Health endpoint returns 200 OK
- [ ] Application logs don't expose secrets
- [ ] CORS is configured correctly (if needed)
- [ ] Input validation is implemented
- [ ] Error messages don't leak sensitive info

---

## Next Steps After Deployment

1. **Create Content**: Write your first blog post
2. **SEO Setup**:
   - Submit sitemap to Google Search Console: https://alexthola.com/sitemap.xml
   - Verify domain ownership
   - Add structured data (schema.org)
3. **Analytics**:
   - Add privacy-focused analytics (e.g., Plausible, Fathom)
   - Monitor traffic patterns
4. **Performance**:
   - Run Lighthouse audits monthly
   - Optimize images
   - Minify assets
5. **Marketing**:
   - Share RSS feed: https://alexthola.com/rss.xml
   - Promote on social media
   - Engage with readers

---

## Additional Resources

### Official Documentation
- [DigitalOcean App Platform Docs](https://docs.digitalocean.com/products/app-platform/)
- [SurrealDB Documentation](https://surrealdb.com/docs)
- [Leptos Book](https://leptos-rs.github.io/leptos/)
- [Axum Framework](https://docs.rs/axum/)

### Community Support
- [DigitalOcean Community](https://www.digitalocean.com/community)
- [SurrealDB Discord](https://discord.gg/surrealdb)
- [Rust Web Development Forum](https://users.rust-lang.org/c/help/13)

### Tools
- [DNS Checker](https://dnschecker.org)
- [SSL Labs Test](https://www.ssllabs.com/ssltest/)
- [Security Headers Check](https://securityheaders.com/)
- [Lighthouse](https://developers.google.com/web/tools/lighthouse)

---

## Conclusion

You now have a complete, production-ready deployment of your Rust/Leptos blog on DigitalOcean with your custom domain from NameCheap. The application runs as a WASM instance with:

- ✅ Automatic HTTPS with Let's Encrypt
- ✅ Global CDN via DigitalOcean's edge network
- ✅ Secure database configuration
- ✅ Automated deployments via GitHub Actions
- ✅ Comprehensive monitoring and alerting
- ✅ Security hardening (headers, rate limiting, firewall)
- ✅ Automated backups

**Estimated setup time**: 2-4 hours (depending on DNS propagation)
**Monthly cost**: ~$19-20 (scales with traffic)

For questions or issues, open a GitHub issue at: https://github.com/athola/blog/issues

---

**Last Updated**: 2025-11-05
**Version**: 2.0.0
