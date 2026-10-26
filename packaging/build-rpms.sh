#!/usr/bin/env bash
# Build, sign and index the Blaze RPM repository.
#
#   packaging/build-rpms.sh            # build everything into packaging/out/repo
#
# Host needs: bash, tar, podman, python3 and the Blaze signing key in
#   $BLAZE_GNUPGHOME (default ~/.blaze-signing). The private key never enters git.
# Everything RPM related runs inside a Fedora container, so the host distribution
# does not matter.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$HERE")"
OVERLAY="$ROOT/blazeos_custom_apps"
OUT="$HERE/out"
GNUPGHOME_HOST="${BLAZE_GNUPGHOME:-$HOME/.blaze-signing}"
FEDORA_IMAGE="${FEDORA_IMAGE:-registry.fedoraproject.org/fedora:44}"
KEYID="${BLAZE_KEYID:-Blaze Release}"

[ -d "$GNUPGHOME_HOST" ] || { echo "missing signing keyring: $GNUPGHOME_HOST" >&2; exit 1; }

rm -rf "$OUT"
mkdir -p "$OUT/SOURCES" "$OUT/repo"

echo "==> staging source tarballs from overlay"
python3 - "$HERE" "$OVERLAY" "$OUT/SOURCES" <<'PY'
import json, os, subprocess, sys
here, overlay, dest = sys.argv[1:4]
for p in json.load(open(os.path.join(here, 'packages.json'))):
    paths = [f.lstrip('/') for f in p['files']]
    if p.get('icons'):
        for root, _, names in os.walk(os.path.join(overlay, 'usr/share/icons/hicolor')):
            for n in names:
                if n == 'blaze-house.png':
                    paths.append(os.path.relpath(os.path.join(root, n), overlay))
    for rel in paths:
        if not os.path.exists(os.path.join(overlay, rel)):
            sys.exit(f"{p['name']}: missing in overlay: {rel}")
    tar = os.path.join(dest, f"{p['name']}-{p['ver']}.tar")
    subprocess.check_call(['tar', '-C', overlay, '--owner=0', '--group=0', '--numeric-owner',
                           '--exclude=__pycache__', '-cf', tar] + paths)
    print('  ', os.path.basename(tar))
PY
cp "$HERE/blaze.repo" "$HERE/keys/RPM-GPG-KEY-blaze" "$OUT/SOURCES/"
cp "$HERE/specs/"*.spec "$OUT/"

echo "==> building, signing and indexing inside $FEDORA_IMAGE"
podman run --rm \
  -v "$OUT:/work:Z" \
  -v "$GNUPGHOME_HOST:/gnupg:Z" \
  -e KEYID="$KEYID" \
  "$FEDORA_IMAGE" bash -euo pipefail -c '
    dnf -y -q install rpm-build rpm-sign createrepo_c gnupg2 rpmlint >/dev/null
    export GNUPGHOME=/tmp/gnupg; cp -a /gnupg /tmp/gnupg; chmod 700 /tmp/gnupg
    cat > ~/.rpmmacros <<EOF
%dist .blaze
%_topdir /work/rpmbuild
%_sourcedir /work/SOURCES
%_gpg_name $KEYID
%_gpg_path /tmp/gnupg
EOF
    mkdir -p /work/rpmbuild/{BUILD,RPMS,SRPMS}
    for spec in /work/*.spec; do rpmbuild -bb "$spec" >/work/$(basename "$spec" .spec).build.log 2>&1 || { tail -30 /work/$(basename "$spec" .spec).build.log; exit 1; }; done
    find /work/rpmbuild/RPMS -name "*.rpm" -exec cp {} /work/repo/ \;
    rpmsign --addsign /work/repo/*.rpm
    rpm --import /work/SOURCES/RPM-GPG-KEY-blaze
    rpm -K /work/repo/*.rpm
    createrepo_c -q /work/repo
    gpg --batch --yes --pinentry-mode loopback -u "$KEYID" --detach-sign --armor /work/repo/repodata/repomd.xml
    cp /work/SOURCES/RPM-GPG-KEY-blaze /work/repo/RPM-GPG-KEY-blaze
    rpmlint /work/repo/*.rpm > /work/rpmlint.txt 2>&1 || true
  '
echo "==> done"
ls -lh "$OUT/repo"
