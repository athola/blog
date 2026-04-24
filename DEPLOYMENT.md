# Production Deployment Guide

This document describes deploying the blog to a DigitalOcean production environment.

## Architecture Overview

-   **Application**: Rust and Leptos application on DigitalOcean App Platform.
-   **Database**: A self-hosted SurrealDB instance on a dedicated DigitalOcean Droplet.
-   **Domain**: `alexthola.com`, managed via NameCheap.
-   **Estimated Monthly Cost**: ~$26 ($12 app, $12 Droplet, $2.40 backups).

## Prerequisites

-   A DigitalOcean account with billing enabled.
-   A registered domain name.
-   The project source code hosted in a GitHub repository.
-   `doctl` (DigitalOcean CLI) installed locally (optional, but recommended).

## Part 1: Database Setup (SurrealDB Droplet)

The recommended setup for the SurrealDB Droplet uses the provided `cloud-init` script, automating installation and initial configuration.

### Droplet Configuration

-   **Image**: Ubuntu 22.04 LTS
-   **Plan**: Basic, $12/month (2GB RAM, 1 vCPU, 50GB SSD). The 1GB plan is not recommended as it can lead to out-of-memory errors.
-   **Region**: New York (NYC3) or the same region as your App Platform application.
-   **Authentication**: Add your SSH key.
-   **User Data**: In the "Advanced Options" section, check "User data" and paste the `cloud-init` script below.

### Cloud-Init Script

This script automates SurrealDB installation, service configuration, and basic security hardening.

```yaml
#cloud-config
package_update: true
package_upgrade: true
packages:
  - curl
  - ufw
  - fail2ban
runcmd:
  - curl -sSf https://install.surrealdb.com | sh
  - useradd -r -s /bin/false surrealdb
  - mkdir -p /var/lib/surrealdb
  - chown surrealdb:surrealdb /var/lib/surrealdb
  - chmod 700 /var/lib/surrealdb
write_files:
  - path: /etc/systemd/system/surrealdb.service
    content: |
      [Unit]
      Description=SurrealDB Database
      After=network.target
      [Service]
      Type=simple
      User=surrealdb
      Group=surrealdb
      EnvironmentFile=/etc/surrealdb/env
      ExecStart=/root/.surrealdb/surreal start \
        --bind 0.0.0.0:8000 \
        --log info \
        file:/var/lib/surrealdb/data.db
      Restart=always
      RestartSec=10
      [Install]
      WantedBy=multi-user.target
system_info:
  default_user:
    name: admin
final_message: "Droplet setup is complete. SSH into the Droplet to set the database password and configure the firewall."
```

### Post-Provisioning Steps

After Droplet creation, SSH in to complete setup.

**1. Set the Database Password**

Generate a secure password and store it in a restricted environment file. Never put credentials directly in `ExecStart` — they appear in `ps aux` output.

```bash
# Generate a password
PASSWORD=$(openssl rand -base64 32)

# Create env file with restricted permissions
sudo mkdir -p /etc/surrealdb
sudo tee /etc/surrealdb/env > /dev/null <<EOF
SURREAL_PASS=$PASSWORD
SURREAL_USER=root
EOF
sudo chmod 600 /etc/surrealdb/env
sudo chown root:surrealdb /etc/surrealdb/env

# Reload the service to apply changes
sudo systemctl daemon-reload
sudo systemctl restart surrealdb

# Verify password is NOT visible in process args
ps aux | grep surreal | grep -v grep
```

**2. Configure the Firewall**

Restrict SurrealDB (port 8000) to the local VPC only. App Platform reaches the database through Caddy (see "Part 1b"), so port 8000 stays private.

```bash
# Allow SSH from your trusted source IPs only (not Anywhere)
sudo ufw allow from YOUR_ADMIN_IP to any port 22

# Allow SurrealDB on port 8000 from the droplet's VPC CIDR only
# Get the VPC CIDR: doctl vpcs get <vpc-uuid>
sudo ufw allow from 10.X.X.0/20 to any port 8000

# Allow Caddy's ACME HTTP-01 challenge and HTTPS listener (configured in Part 1b)
sudo ufw allow 80/tcp comment 'Caddy ACME'
sudo ufw allow 8443/tcp comment 'Caddy HTTPS for SurrealDB proxy'

# Enable the firewall
sudo ufw enable
```

The database setup is now complete.

## Part 1b: TLS Reverse Proxy (Caddy)

DigitalOcean App Platform containers block outbound connections on port 22, and App Platform instances don't join custom VPCs. The blog reaches SurrealDB through a Caddy reverse proxy terminating TLS on the droplet.

### 1. Add DNS record

Point a subdomain (e.g. `db.alexthola.com`) at the droplet's public IP:

```bash
doctl compute domain records create alexthola.com \
  --record-type A --record-name db \
  --record-data YOUR_DROPLET_PUBLIC_IP --record-ttl 300
```

### 2. Install Caddy

```bash
apt-get install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' \
  | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' \
  > /etc/apt/sources.list.d/caddy-stable.list
apt-get update && apt-get install -y caddy
```

### 3. Write `/etc/caddy/Caddyfile`

```caddy
{
  email YOUR_EMAIL_FOR_ACME
}

db.YOUR_DOMAIN:8443 {
  reverse_proxy http://127.0.0.1:8000
}
```

### 4. Start and verify

```bash
systemctl enable --now caddy
# From outside:
curl https://db.YOUR_DOMAIN:8443/health   # expect HTTP 200
```

Let's Encrypt issues the certificate via HTTP-01 on port 80 (opened in step 2 of Part 1). Caddy renews automatically.

### Manual Database Setup (Alternative)

If you prefer manual Droplet configuration, create a Droplet with the listed specifications (without user data) and follow these steps.

**1. Install SurrealDB**

```bash
ssh root@YOUR_DROPLET_IP
curl -sSf https://install.surrealdb.com | sh
```

**2. Create Service User**

```bash
useradd -r -s /bin/false surrealdb
mkdir -p /var/lib/surrealdb
chown surrealdb:surrealdb /var/lib/surrealdb
chmod 700 /var/lib/surrealdb
```

**3. Configure and Start the Service**

Create the systemd service file `/etc/systemd/system/surrealdb.service` with the same content as in the `cloud-init` script, then create the `/etc/surrealdb/env` credentials file as described in the Post-Provisioning section above.

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now surrealdb
```

## Part 2: Application Deployment

Once the database is running, deploy the application on the DigitalOcean App Platform.

### 1. Create the App

1.  In the DigitalOcean console, navigate to **Apps** and click **Create App**.
2.  Select your GitHub repository (`athola/blog`) and the `master` branch.
3.  Enable **Autodeploy** to automatically redeploy on pushes to `master`.
4.  DigitalOcean will detect the `Dockerfile` and configure the application.
    Ensure the service uses the Dockerfile build type (not a buildpack).

### 2. Configure the App

-   **Name**: A descriptive name, e.g., `blog-web`.
-   **Region**: Match the database droplet's region.
-   **Instance Size**: Basic is sufficient. No Professional-tier upgrade is needed since the app reaches SurrealDB over public HTTPS (via Caddy), not VPC private networking.
-   **HTTP Port**: Set to `8080`.

### 3. Set Environment Variables

Add the following as **encrypted** environment variables in the app's settings.

```
RUST_LOG=info
LEPTOS_SITE_ADDR=0.0.0.0:8080
LEPTOS_SITE_ROOT=site
LEPTOS_HASH_FILES=true
SURREAL_ADDRESS=https://db.YOUR_DOMAIN:8443
SURREAL_NS=production
SURREAL_DB=alexthola_blog
SURREAL_ROOT_USER=root
SURREAL_ROOT_PASS=YOUR_SECURE_PASSWORD
```

**Notes**:
- `SURREAL_ADDRESS` points at the Caddy reverse proxy set up in Part 1b. TLS is terminated on the droplet; SurrealDB auth (`SURREAL_ROOT_USER`/`SURREAL_ROOT_PASS`) gates access.
- Mark `SURREAL_ROOT_PASS`, `SURREAL_NS`, `SURREAL_DB`, and `SURREAL_ROOT_USER` as encrypted (`type: SECRET`) in the App spec.
- Prior versions of this guide used an SSH tunnel or a private-IP direct connection; both have been retired. The tunnel scripts (`scripts/tunnel.sh`) remain in the repo as a no-op fallback when `TUNNEL_HOST` is unset.

### 4. Deploy

Click **Create Resources**. The initial build and deployment will take 10-15 minutes.

### 5. Configure DNS

After the app is live, point your domain to it.

1.  In your app's **Settings** tab, go to the **Domains** section and add your custom domain (e.g., `alexthola.com`).
2.  Follow the instructions to configure your DNS records. The recommended approach is to use DigitalOcean's nameservers. In your domain registrar's dashboard (e.g., NameCheap), change the nameservers to `ns1.digitalocean.com`, `ns2.digitalocean.com`, and `ns3.digitalocean.com`.
3.  In the DigitalOcean **Networking** tab, ensure you have an `A` record for your root domain (`@`) pointing to your app, and a `CNAME` record for `www` pointing to the root domain (`@`).

### 6. Initialize the Database

The final step is to apply the database schema.

```bash
# Set environment variables locally
export SURREAL_ADDRESS="https://db.YOUR_DOMAIN:8443"
export SURREAL_NS="production"
export SURREAL_DB="alexthola_blog"
export SURREAL_ROOT_USER="root"
export SURREAL_ROOT_PASS="YOUR_SECURE_PASSWORD"

# Connect through the Caddy proxy and import the schema
surreal sql --conn $SURREAL_ADDRESS --user $SURREAL_ROOT_USER --pass $SURREAL_ROOT_PASS --ns $SURREAL_NS --db $SURREAL_DB < migrations/schema.surql
```

**Note**: Port 8000 on the droplet is VPC-only, so direct `http://PUBLIC_IP:8000` connections are blocked by UFW. Route all admin traffic through the public Caddy endpoint (`db.YOUR_DOMAIN:8443`) or SSH into the droplet and run `surreal sql` locally against `http://localhost:8000`.

## Troubleshooting Guide

### SSH Permission Denied

If you cannot SSH into the Droplet (`Permission denied (publickey)`), reset the root password.

1.  In the DigitalOcean console, go to your **Droplet > Access**.
2.  Click **Reset Root Password** and check your email for a temporary password.
3.  Log in as `root` using the temporary password. You will be prompted to set a new one.

To avoid this, always add your SSH key during Droplet creation.

### Cloud-Init Failures

If automated setup fails, check logs to diagnose the issue.

```bash
# Check the status of the cloud-init service
cloud-init status

# Review the output logs for errors
sudo cat /var/log/cloud-init-output.log
```

If necessary, you can run the setup steps manually by following the instructions in the "Manual Database Setup" section.

### Application Fails to Start

Check the application's runtime logs for errors. You can do this in the DigitalOcean console (**App > Logs > Runtime Logs**) or via `doctl`.

```bash
doctl apps logs <YOUR_APP_ID> --type=run --follow
```

Common causes:
-   Missing or incorrect environment variables.
-   Database connection failure (check firewall and `SURREAL_ADDRESS`).
-   Incorrect port binding (the application must bind to `0.0.0.0:8080`).

### Database Connection Errors

1.  **Verify the service is running** on the Droplet.
    ```bash
    ssh root@YOUR_DROPLET_IP
    systemctl status surrealdb
    ```
2.  **Check the service logs** for errors.
    ```bash
    journalctl -u surrealdb -n 50
    ```
3.  **Test the health endpoint**. From the Droplet, you should be able to connect to the database.
    ```bash
    curl http://localhost:8000/health
    ```
    If this fails, the database service is not running correctly. If it succeeds, the issue is likely with the firewall or the private network connection from the App Platform.

### Domain Not Propagating

DNS changes can take time to propagate. Use `dig` to check the status.

```bash
# Should return the IP of your DigitalOcean app
dig your-domain.com +short

# Should return the DigitalOcean nameservers
dig NS your-domain.com +short
```

If the issue persists after several hours, double-check the DNS records in your DigitalOcean networking panel.

## Operations and Maintenance

### Security

-   **Database**: Access restricted by firewall to app's private VPC. Use strong, generated password, rotated quarterly.
-   **Application**: App Platform provides automatic HTTPS, DDoS mitigation, and a managed runtime.
-   **Secrets**: Credentials not stored in repository; managed as encrypted environment variables in App Platform.

### Routine Maintenance

**Monthly Tasks**
-   Review application and database logs for unusual activity.
-   Update dependencies with `cargo update` and run tests before deploying.
-   Verify that database backups are being created successfully.

**Quarterly Tasks**
-   Perform a security audit of the application and its dependencies.
-   Rotate the database password and update the `SURREAL_ROOT_PASS` environment variable.
-   Review monthly costs and adjust resources as needed.

### Useful Commands

```bash
# View application logs
doctl apps logs <APP_ID> --type=run --follow

# Restart the application
doctl apps restart <APP_ID>

# Create a manual database backup
surreal export --conn $SURREAL_ADDRESS --user $SURREAL_ROOT_USER --pass $SURREAL_ROOT_PASS --ns $SURREAL_NS --db $SURREAL_DB backup.surql

# List droplets by tag
doctl compute droplet list --tag-name blog

# Access droplet metadata (from the droplet itself)
curl http://169.254.169.254/metadata/v1/id
```

## Deployment Notes

### Key Learnings

-   **Droplet Sizing**: The $12/month Droplet (2GB RAM) is recommended; the 1GB option can cause out-of-memory errors.
-   **Build Optimization**: The `Dockerfile` uses `CARGO_BUILD_JOBS=2` to limit parallel rustc instances and purges the `target/` directory before Kaniko's filesystem snapshot. Without this, the multi-GB build cache causes DO's builder to run out of memory.
-   **VPC Networking**: Only source-built apps (Dockerfile on DO) get VPC access. Deploying from a pre-built DOCR image does **not** provide VPC networking, even on Professional tier.
-   **CSS Hashing**: `hash-files = true` must be set in both Cargo.toml (runtime) and the Dockerfile build env (`LEPTOS_HASH_FILES=true`). A mismatch causes CSS 404 errors because filenames are hashed on disk but the HTML references unhashed URLs.
-   **DNS Propagation**: DNS changes can take up to two hours to fully propagate.

### Cost and Scaling

| Service                      | Monthly Cost |
| ---------------------------- | ------------ |
| App Platform (Professional)  | $12.00       |
| SurrealDB Droplet            | $12.00       |
| Droplet Backups              | $2.40        |
| **Total**                    | **$26.40**   |

Consider upgrading resources when:
-   Sustained CPU usage above 70% on the app.
-   Sustained memory usage above 80% on the Droplet.
-   Consistently slow database queries.

---
*Last updated: 2026-03-27*
