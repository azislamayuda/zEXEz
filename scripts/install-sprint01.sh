#!/usr/bin/env bash
# Safely install ONLY the Sprint 01 files from the official project ZIP.
# Usage: bash install-sprint01.sh [archive.zip] [project-directory]
set -euo pipefail

ARCHIVE="${1:-$HOME/Downloads/zEXEz-v0.1.0-sprint01-hotfix01.zip}"
TARGET="${2:-$HOME/ZDATA/ME/GITHUB/zEXEz}"

if [[ ! -f "$ARCHIVE" ]]; then
    echo "ERROR: ZIP not found: $ARCHIVE" >&2
    exit 1
fi
if [[ ! -d "$TARGET/.git" ]]; then
    echo "ERROR: Destination must be the existing zEXEz Git repository: $TARGET" >&2
    exit 1
fi

STAGING="$(mktemp -d)"
trap 'rm -rf -- "$STAGING"' EXIT
unzip -tq "$ARCHIVE" >/dev/null
unzip -q "$ARCHIVE" -d "$STAGING"

FILES=(
    .gitignore
    Cargo.toml
    src/main.rs
    src/runtime.rs
    scripts/verify-sprint01.sh
    scripts/install-sprint01.sh
    docs/SPRINT-01.md
)

for file in "${FILES[@]}"; do
    [[ -f "$STAGING/$file" && ! -L "$STAGING/$file" ]] || {
        echo "ERROR: Missing or invalid file in ZIP: $file" >&2
        exit 1
    }
done

BACKUP=""
for file in "${FILES[@]}"; do
    if [[ -e "$TARGET/$file" ]] && ! cmp -s -- "$STAGING/$file" "$TARGET/$file"; then
        if [[ -z "$BACKUP" ]]; then
            BACKUP="$HOME/.local/share/zexez/backups/sprint01-$(date +%Y%m%d-%H%M%S)-$$"
        fi
        mkdir -p -- "$BACKUP/$(dirname -- "$file")"
        cp -p -- "$TARGET/$file" "$BACKUP/$file"
        printf 'Backed up: %s\n' "$file"
    fi
    mkdir -p -- "$TARGET/$(dirname -- "$file")"
    cp -p -- "$STAGING/$file" "$TARGET/$file"
    cmp -s -- "$STAGING/$file" "$TARGET/$file" || {
        echo "ERROR: Copy verification failed: $file" >&2
        exit 1
    }
    printf 'Installed and verified: %s\n' "$file"
done

printf '\nTarget: %s\n' "$TARGET"
if [[ -n "$BACKUP" ]]; then
    printf 'Backup: %s\n' "$BACKUP"
fi
printf '\nGit changes:\n'
git -C "$TARGET" status --short

if [[ "${ZEXEZ_SKIP_BUILD:-0}" == 1 ]]; then
    echo 'Build verification skipped (ZEXEZ_SKIP_BUILD=1).'
else
    echo
    echo '=== Ubuntu-side build verification ==='
    bash "$TARGET/scripts/verify-sprint01.sh"
fi
