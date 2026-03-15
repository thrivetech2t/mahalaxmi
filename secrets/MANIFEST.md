# Secrets Manifest — mahalaxmi-website

## Encrypted files in this directory

| Encrypted file | Decrypted target path | Purpose |
|---|---|---|
| `.env.enc` | `/opt/deployments/mahalaxmi-website/.env` | All service environment variables |

## Secret variables in .env

Based on the audit of `.env.example` files found in this repo:

| Variable | Purpose |
|---|---|
| NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY | Stripe publishable key for client-side |
| STRIPE_SECRET_KEY | Stripe secret key for server-side |
| STRIPE_WEBHOOK_SECRET | Stripe webhook signing secret |
| MAHALAXMI_CLOUD_PAK_KEY | Mahalaxmi cloud channel provisioning key |
| NEXTAUTH_SECRET | NextAuth.js session signing secret |
| NEXTAUTH_URL | Canonical URL for NextAuth callbacks |
| DATABASE_URL | PostgreSQL connection string |
| GHCR_PULL_TOKEN | GitHub PAT for pulling private Docker images |
| CLOUDFLARE_API_TOKEN | Cloudflare DNS API token |
| SMTP_USER | SMTP credentials for email |
| SMTP_PASSWORD | SMTP password |

## How to decrypt

```bash
bash scripts/decrypt-secrets.sh
```

## How to encrypt after editing

```bash
bash scripts/encrypt-secrets.sh
```

## Key location

The age private key lives at:
- Server: `/root/.config/sops/age/keys.txt`
- Local: `~/.config/sops/age/keys.txt`
- Backup: stored in ThriveTech password manager under "SOPS age key"
