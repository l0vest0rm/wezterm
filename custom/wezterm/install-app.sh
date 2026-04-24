#!/bin/bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
LATEST_APP_DIR=$(find "$ROOT_DIR" -maxdepth 1 -type d -name 'WezTerm-macos-*' | sort | tail -n 1)
APP_SOURCE="${LATEST_APP_DIR}/WezTerm.app"
APP_TARGET="/Applications/WezTerm.app"
STAMP=$(date +%Y%m%d-%H%M%S)

if [[ -z "${LATEST_APP_DIR:-}" || ! -d "$APP_SOURCE" ]]; then
  echo "no built WezTerm.app found under $ROOT_DIR" >&2
  echo "run 'make app' first" >&2
  exit 1
fi

if [[ -e "$APP_TARGET" ]]; then
  BACKUP_APP="/Applications/WezTerm.app.bak.${STAMP}"
  mv "$APP_TARGET" "$BACKUP_APP"
  echo "backed up existing app to $BACKUP_APP"
fi

cp -R "$APP_SOURCE" "$APP_TARGET"
echo "installed app: $APP_TARGET"
