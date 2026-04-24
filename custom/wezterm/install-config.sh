#!/bin/bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
SOURCE_CONFIG="$ROOT_DIR/custom/wezterm/wezterm.lua"
TARGET_DIR="${HOME}/.config/wezterm"
TARGET_CONFIG="${TARGET_DIR}/wezterm.lua"
TARGET_TAB_TITLE_HELPER="${TARGET_DIR}/tab-title.sh"
LEGACY_CONFIG="${HOME}/.wezterm.lua"
STAMP=$(date +%Y%m%d-%H%M%S)

if [[ ! -f "$SOURCE_CONFIG" ]]; then
  echo "missing source config: $SOURCE_CONFIG" >&2
  exit 1
fi

mkdir -p "$TARGET_DIR"

if [[ -e "$TARGET_CONFIG" || -L "$TARGET_CONFIG" ]]; then
  BACKUP_CONFIG="${TARGET_CONFIG}.bak.${STAMP}"
  mv "$TARGET_CONFIG" "$BACKUP_CONFIG"
  echo "backed up existing config to $BACKUP_CONFIG"
fi

cp "$SOURCE_CONFIG" "$TARGET_CONFIG"
echo "installed config copy: $TARGET_CONFIG"
cp "$ROOT_DIR/custom/wezterm/tab-title.sh" "$TARGET_TAB_TITLE_HELPER"
echo "installed tab title helper: $TARGET_TAB_TITLE_HELPER"

if [[ -e "$LEGACY_CONFIG" || -L "$LEGACY_CONFIG" ]]; then
  BACKUP_LEGACY="${LEGACY_CONFIG}.bak.${STAMP}"
  mv "$LEGACY_CONFIG" "$BACKUP_LEGACY"
  echo "backed up legacy config to $BACKUP_LEGACY"
fi

bash "$ROOT_DIR/custom/wezterm/install-shell-integration.sh"
