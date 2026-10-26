#!/usr/bin/env bash
# Install Sprint 02 Hotfix 02 into an EXISTING zEXEz Git checkout.
# Whitelist files, reject malicious ZIP entries, backup modified files,
# and leave README.md, LICENSE, PRD.md, Cargo.lock and ~/.wine untouched.
set -euo pipefail

ZIP="${1:-$HOME/Downloads/zEXEz-v0.1.0-sprint02-hotfix02.zip}"
TARGET="${2:-$HOME/ZDATA/ME/GITHUB/zEXEz}"
[[ -f "$ZIP" ]] || { echo "ERROR: ZIP not found: $ZIP" >&2; exit 1; }
[[ -d "$TARGET/.git" ]] || { echo "ERROR: Existing Git repository not found: $TARGET" >&2; exit 1; }
command -v python3 >/dev/null || { echo 'ERROR: python3 required' >&2; exit 1; }
command -v git >/dev/null || { echo 'ERROR: git required' >&2; exit 1; }
TARGET="$(cd "$TARGET" && pwd -P)"
[[ "$(basename "$TARGET")" == 'zEXEz' ]] || {
    echo "ERROR: Expected zEXEz repository: $TARGET" >&2; exit 1;
}

STAGING="$(mktemp -d)"
trap 'rm -rf -- "$STAGING"' EXIT

# Securely extract known members only. ZIP may not contain links, directories,
# duplicate members, absolute paths, or unexpected extra files.
python3 - "$ZIP" "$STAGING" <<'PY'
import pathlib, stat, sys, zipfile
archive, staging = sys.argv[1:]
expected = {
    '.gitignore', 'Cargo.toml',
    'src/main.rs', 'src/runtime.rs', 'src/pe.rs', 'src/engine.rs',
    'scripts/install-sprint01.sh', 'scripts/verify-sprint01.sh',
    'scripts/install-sprint02.sh', 'scripts/verify-sprint02.sh',
    'scripts/install-sprint02-hotfix01.sh', 'scripts/verify-sprint02-hotfix01.sh',
    'scripts/install-sprint02-hotfix02.sh', 'scripts/verify-sprint02-hotfix02.sh',
    'docs/SPRINT-01.md', 'docs/SPRINT-02.md', 'docs/SPRINT-02-HOTFIX-01.md',
    'docs/SPRINT-02-HOTFIX-02.md',
}
with zipfile.ZipFile(archive) as z:
    if z.testzip() is not None:
        raise SystemExit('ERROR: ZIP CRC check failed')
    names = [f.filename for f in z.infolist()]
    if len(names) != len(set(names)) or set(names) != expected:
        raise SystemExit('ERROR: ZIP file list differs from expected hotfix contents')
    for info in z.infolist():
        mode = (info.external_attr >> 16) & 0xFFFF
        if stat.S_IFMT(mode) == stat.S_IFLNK:
            raise SystemExit('ERROR: ZIP may not contain symlinks')
        target = pathlib.Path(staging) / info.filename
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(z.read(info))
print('ZIP integrity and whitelist: PASS')
PY

BACKUP=""
# Never stage unknown files or overwrite important repository docs / lockfiles.
while IFS= read -r relative; do
    source="$STAGING/$relative"
    destination="$TARGET/$relative"
    [[ -f "$source" && ! -L "$source" ]] || { echo "ERROR: Missing file $relative" >&2; exit 1; }
    [[ ! -L "$destination" ]] || { echo "ERROR: Refusing destination symlink $relative" >&2; exit 1; }
    directory="$(dirname "$relative")"
    [[ ! -L "$TARGET/$directory" ]] || { echo "ERROR: Refusing directory symlink $directory" >&2; exit 1; }
    if [[ -f "$destination" ]] && cmp -s -- "$source" "$destination"; then
        echo "Unchanged: $relative"
        continue
    fi
    if [[ -e "$destination" ]]; then
        if [[ -z "$BACKUP" ]]; then
            BACKUP="$HOME/.local/share/zexez/backups/sprint02-hotfix02-$(date +%Y%m%d-%H%M%S)-$$"
        fi
        mkdir -p -- "$BACKUP/$directory"
        cp -p -- "$destination" "$BACKUP/$relative"
        echo "Backed up: $relative"
    fi
    mkdir -p -- "$TARGET/$directory"
    tempfile="$(mktemp "$destination.tmp.XXXXXX")"
    cp -p -- "$source" "$tempfile"
    mv -f -- "$tempfile" "$destination"
    cmp -s -- "$source" "$destination" || { echo "ERROR: Copy failed for $relative" >&2; exit 1; }
    echo "Installed: $relative"
done < <(find "$STAGING" -type f -printf '%P\n' | LC_ALL=C sort)

printf '\nRepository: %s\n' "$TARGET"
[[ -z "$BACKUP" ]] || printf 'Backups: %s\n' "$BACKUP"
printf '\nGit status:\n'
git -C "$TARGET" status --short

if [[ "${ZEXEZ_SKIP_BUILD:-0}" == '1' ]]; then
    echo 'Build verification skipped. Run scripts/verify-sprint02-hotfix02.sh to verify later.'
else
    bash "$TARGET/scripts/verify-sprint02-hotfix02.sh"
fi
