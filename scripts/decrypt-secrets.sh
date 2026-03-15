#!/bin/bash
# Run after git pull to restore secret files to their target paths
set -e

SECRETS_DIR="$(cd "$(dirname "$0")/../secrets" && pwd)"
DEPLOY_PATH="/opt/deployments/mahalaxmi-website"

echo "Decrypting secrets..."

if [ -f "$SECRETS_DIR/.env.enc" ]; then
  mkdir -p "$DEPLOY_PATH"
  sops --decrypt "$SECRETS_DIR/.env.enc" > "$DEPLOY_PATH/.env"
  chmod 600 "$DEPLOY_PATH/.env"
  echo "  .env -> $DEPLOY_PATH/.env"
fi

echo ""
echo "All secrets decrypted successfully"
