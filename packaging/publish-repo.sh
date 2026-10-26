#!/usr/bin/env bash
# Publish packaging/out/repo to the gh-pages branch (served by GitHub Pages at
# https://darkmorpheus-pc.github.io/Blaze-Galaxy/repo/).
# The branch holds only the latest repo snapshot (orphan commit, force push) so
# binary history does not accumulate in the repository.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$HERE")"
REPO_DIR="$HERE/out/repo"
REMOTE="${BLAZE_REMOTE:-$(git -C "$ROOT" remote get-url origin)}"

[ -f "$REPO_DIR/repodata/repomd.xml.asc" ] || { echo "run build-rpms.sh first (signed repomd missing)" >&2; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/repo"
cp -a "$REPO_DIR/." "$TMP/repo/"
touch "$TMP/.nojekyll"
cat > "$TMP/index.html" <<'HTML'
<!doctype html><meta charset="utf-8"><title>Blaze SolarEvolution package repository</title>
<h1>Blaze SolarEvolution package repository</h1>
<p>Install <code>blaze-release</code> to enable it, or add <code>repo/</code> to <code>/etc/yum.repos.d/</code>.</p>
<p>Signing key: <a href="repo/RPM-GPG-KEY-blaze">RPM-GPG-KEY-blaze</a></p>
<ul><li><a href="repo/">Packages</a></li></ul>
HTML

cd "$TMP"
git init -q -b gh-pages
git add -A
git -c user.name="Blaze Release" -c user.email="release@blazeos.com.tr" commit -q -m "Publish Blaze repo snapshot $(date -u +%Y-%m-%dT%H:%MZ)"
git push -q --force "$REMOTE" gh-pages
echo "published to gh-pages of $REMOTE"
