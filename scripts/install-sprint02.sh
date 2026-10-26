#!/usr/bin/env bash
# Install full Sprint 02 source to the existing zEXEz Git repository.
# Backs up modified files. Protects README, LICENSE, PRD and Cargo.lock.
# Usage: bash install-sprint02.sh ZIP [target-folder]
set -euo pipefail

ZIP="${1:-$HOME/Downloads/zEXEz-v0.1.0-sprint02.zip}"
TARGET="${2:-$HOME/ZDATA/ME/GITHUB/zEXEz}"

[[ -f "$ZIP" ]] || { echo "ERROR: ZIP not found: $ZIP" >&2; exit 1; }
[[ -d "$TARGET/.git" ]] || { echo "ERROR: Existing Git repository not found: $TARGET" >&2; exit 1; }
command -v unzip >/dev/null || { echo 'ERROR: unzip is required' >&2; exit 1; }
command -v git >/dev/null || { echo 'ERROR: git is required' >&2; exit 1; }

TARGET="$(cd "$TARGET" && pwd -P)"
[[ "$(basename "$TARGET")" == "zEXEz" ]] || {
    echo "ERROR: Refusing unexpected target folder: $TARGET (expected zEXEz)" >&2
    exit 1
}

STAGING="$(mktemp -d)"
trap 'rm -rf -- "$STAGING"' EXIT
unzip -tq "$ZIP" >/dev/null
# Only expected relative archive paths. No absolute paths, traversal or backslashes.
while IFS= read -r member; do
    [[ "$member" != /* && "$member" != *'..'* && "$member" != *'\'* ]] || {
        echo "ERROR: Unsafe member in ZIP: $member" >&2
        exit 1
    }
done < <(unzip -Z1 "$ZIP")
unzip -q "$ZIP" -d "$STAGING"

FILES=(
    .gitignore
    Cargo.toml
    src/main.rs
    src/runtime.rs
    src/pe.rs
    src/engine.rs
    scripts/install-sprint01.sh
    scripts/verify-sprint01.sh
    scripts/install-sprint02.sh
    scripts/verify-sprint02.sh
    docs/SPRINT-01.md
    docs/SPRINT-02.md
)
for file in "${FILES[@]}"; do
    [[ -f "$STAGING/$file" && ! -L "$STAGING/$file" ]] || {
        echo "ERROR: Missing or invalid source: $file" >&2
        exit 1
    }
    if [[ -L "$TARGET/$file" || -L "$TARGET/$(dirname "$file")" ]]; then
        echo "ERROR: Refusing symlink at destination: $file" >&2
        exit 1
    fi
done

BACKUP=""
for file in "${FILES[@]}"; do
    src="$STAGING/$file"
    dst="$TARGET/$file"
    if [[ -f "$dst" ]] && cmp -s -- "$src" "$dst"; then
        echo "Unchanged: $file"
        continue
    fi
    if [[ -e "$dst" ]]; then
        if [[ -z "$BACKUP" ]]; then
            BACKUP="$HOME/.local/share/zexez/backups/sprint02-$(date +%Y%m%d-%H%M%S)-$$"
        fi
        mkdir -p "$BACKUP/$(dirname "$file")"
        cp -p -- "$dst" "$BACKUP/$file"
        echo "Backed up: $file"
    fi
    mkdir -p -- "$(dirname "$dst")"
    temp="$(mktemp "$dst.tmp.XXXXXX")"
    cp -p -- "$src" "$temp"
    mv -f -- "$temp" "$dst"
    cmp -s -- "$src" "$dst" || { echo "ERROR: Copy mismatch for $file" >&2; exit 1; }
    echo "Installed: $file"
done

printf '\nRepository: %s\n' "$TARGET"
[[ -z "$BACKUP" ]] || printf 'Backup: %s\n' "$BACKUP"
printf '\nGit status:\n'
git -C "$TARGET" status --short

echo
if [[ "${ZEXEZ_SKIP_BUILD:-0}" == "1" ]]; then
    echo 'Build skipped by ZEXEZ_SKIP_BUILD=1. Run scripts/verify-sprint02.sh manually.'
else
    bash "$TARGET/scripts/verify-sprint02.sh"
fi
