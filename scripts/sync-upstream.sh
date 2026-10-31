#!/usr/bin/env sh
# Fetch each upstream engine at the revision pinned in upstream.lock and apply Wenshu's patch
# queue (patches/<name>/*.patch, in order). Safe to re-run: an existing checkout is reset first.
#
#   scripts/sync-upstream.sh            fetch + patch every upstream
#   scripts/sync-upstream.sh --export   write the current upstream diffs back to patches/ (after editing)
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

while read -r name url rev; do
    case "$name" in ''|'#'*) continue ;; esac
    dir="upstream/$name"
    if [ "${1:-}" = "--export" ]; then
        mkdir -p "patches/$name"
        git -C "$dir" diff > "patches/$name/0001-wenshu.patch.new"
        echo "wrote patches/$name/0001-wenshu.patch.new (split it into topic patches before committing)"
        continue
    fi
    if [ ! -d "$dir/.git" ]; then
        git clone --filter=blob:none "$url" "$dir"
    fi
    git -C "$dir" fetch --quiet origin "$rev" 2>/dev/null || git -C "$dir" fetch --quiet origin
    git -C "$dir" checkout --quiet --force "$rev"
    git -C "$dir" clean -fdq
    for p in patches/"$name"/*.patch; do
        [ -e "$p" ] || continue
        git -C "$dir" apply --whitespace=nowarn "$root/$p"
        echo "applied $p"
    done
    echo "$name at $rev"
done < upstream.lock

# The interface translations WordCraft embeds (zh-Hans among them): storytold/craft-i18n at the
# revision WordCraft pins. `.cargo/config.toml` points CRAFT_I18N_DIR here; without it the desktop
# interface is English only.
if [ "${1:-}" != "--export" ] && [ -f upstream/wordcraft/craft-i18n.rev ]; then
    rev=$(tr -d '[:space:]' < upstream/wordcraft/craft-i18n.rev)
    dir=upstream/craft-i18n
    if [ ! -d "$dir/.git" ]; then
        git clone --filter=blob:none --quiet https://github.com/storytold/craft-i18n "$dir"
    fi
    git -C "$dir" fetch --quiet origin "$rev" 2>/dev/null || git -C "$dir" fetch --quiet origin
    git -C "$dir" checkout --quiet --force "$rev"
    echo "craft-i18n at $rev"
fi
