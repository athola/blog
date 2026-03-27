---
name: security-audit
description: Audit DigitalOcean infrastructure security - droplets, firewalls, exposed ports, secrets
---

Run a security audit of the DigitalOcean infrastructure for the blog.

## Checks to perform

### 1. Droplet exposure
```bash
doctl compute droplet list --format ID,Name,PublicIPv4,PrivateIPv4,Status --no-header
```
For each droplet, check what ports are publicly accessible:
```bash
# Check common dangerous ports from outside
for port in 8000 5432 3306 6379 27017; do
  timeout 3 bash -c "echo >/dev/tcp/DROPLET_PUBLIC_IP/$port" 2>/dev/null && echo "OPEN: port $port" || echo "CLOSED: port $port"
done
```

### 2. Cloud firewalls
```bash
doctl compute firewall list --format ID,Name --no-header
```
If no firewalls exist, flag as a concern.

### 3. App Platform secrets
```bash
doctl apps spec get $(doctl apps list --format ID,Spec.Name --no-header | grep alexthola-blog | awk '{print $1}') 2>&1 | grep -A1 'type: SECRET'
```
Verify secrets don't contain placeholder values like `${VAR}`.

### 4. SurrealDB process exposure
If SSH access is available, check:
- Is SurrealDB root password visible in process args? (`ps aux | grep surreal`)
- Is UFW enabled and properly configured? (`ufw status`)
- Are credentials in cleartext config files?

### 5. Security headers on the live site
```bash
curl -sI "https://alexthola-blog-4hz6l.ondigitalocean.app/" | grep -iE "strict-transport|x-content-type|x-frame|content-security|referrer-policy"
```

Report findings with severity levels: CRITICAL, HIGH, MEDIUM, LOW.
