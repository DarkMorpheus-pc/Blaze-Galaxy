#!/usr/bin/env bash
# Install the freshly built packages from the signed local repo into a clean Fedora
# container, with gpgcheck and repo_gpgcheck enabled, and smoke-test them.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEDORA_IMAGE="${FEDORA_IMAGE:-registry.fedoraproject.org/fedora:44}"

podman run --rm -v "$HERE/out/repo:/repo:ro,Z" "$FEDORA_IMAGE" bash -euo pipefail -c '
  cat > /etc/yum.repos.d/blaze-test.repo <<REPO
[blaze-test]
name=Blaze test
baseurl=file:///repo
enabled=1
gpgcheck=1
repo_gpgcheck=1
gpgkey=file:///repo/RPM-GPG-KEY-blaze
REPO
  dnf -y -q install blaze-release blaze-sentinel blaze-tools blaze-recovery blaze-continuum 2>&1 | tail -5
  echo "--- installed:"
  rpm -q blaze-release blaze-sentinel blaze-tools blaze-recovery blaze-continuum
  echo "--- sentinel smoke test:"
  blaze-sentinel status | head -6
  echo "--- blaze.repo shipped by blaze-release:"
  cat /etc/yum.repos.d/blaze.repo
  echo "--- tamper test (must be detected):"
  cp /repo/blaze-sentinel-*.rpm /tmp/x.rpm
  sz=$(stat -c %s /tmp/x.rpm)
  printf X | dd of=/tmp/x.rpm bs=1 seek=$((sz/2)) conv=notrunc 2>/dev/null
  if rpm -K /tmp/x.rpm; then echo UNEXPECTED-PASS; exit 1; else echo tamper-detected-OK; fi
'
