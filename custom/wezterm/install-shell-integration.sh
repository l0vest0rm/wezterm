#!/bin/bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
SOURCE_SHELL_INTEGRATION="$ROOT_DIR/assets/shell-integration/wezterm.sh"
TARGET_SHELL_DIR="${HOME}/.config/wezterm"
SHELL_INTEGRATION="$TARGET_SHELL_DIR/wezterm.sh"
TAB_TITLE_HELPER="$TARGET_SHELL_DIR/tab-title.sh"
ZSHRC="${HOME}/.zshrc"
MARK_BEGIN="# >>> wezterm shell integration >>>"
MARK_END="# <<< wezterm shell integration <<<"
BLOCK=$(cat <<EOF
$MARK_BEGIN
if [ -f "$SHELL_INTEGRATION" ]; then
  . "$SHELL_INTEGRATION"
fi
if [ -f "$TAB_TITLE_HELPER" ]; then
  . "$TAB_TITLE_HELPER"
fi
$MARK_END
EOF
)

if [[ ! -f "$SOURCE_SHELL_INTEGRATION" ]]; then
  echo "missing shell integration script: $SOURCE_SHELL_INTEGRATION" >&2
  exit 1
fi

mkdir -p "$TARGET_SHELL_DIR"
cp "$SOURCE_SHELL_INTEGRATION" "$SHELL_INTEGRATION"

if [[ ! -f "$ZSHRC" ]]; then
  touch "$ZSHRC"
fi

if grep -Fq "$MARK_BEGIN" "$ZSHRC"; then
  TMP_FILE="${ZSHRC}.tmp.$$"
  BEGIN_LINE=$(grep -nF "$MARK_BEGIN" "$ZSHRC" | head -n 1 | cut -d: -f1)
  END_LINE=$(grep -nF "$MARK_END" "$ZSHRC" | head -n 1 | cut -d: -f1)

  if [[ -z "${BEGIN_LINE:-}" || -z "${END_LINE:-}" || "$BEGIN_LINE" -gt "$END_LINE" ]]; then
    echo "failed to locate existing wezterm shell integration block in $ZSHRC" >&2
    exit 1
  fi

  if [[ "$BEGIN_LINE" -gt 1 ]]; then
    sed -n "1,$((BEGIN_LINE - 1))p" "$ZSHRC" >"$TMP_FILE"
  else
    : >"$TMP_FILE"
  fi

  printf '%s\n' "$BLOCK" >>"$TMP_FILE"

  sed -n "$((END_LINE + 1)),\$p" "$ZSHRC" >>"$TMP_FILE"
  mv "$TMP_FILE" "$ZSHRC"
  echo "updated wezterm shell integration block in $ZSHRC"
  exit 0
fi

printf '\n%s\n' "$BLOCK" >>"$ZSHRC"

echo "appended wezterm shell integration block to $ZSHRC"
