# Build-time binary inputs

## htop 3.5.3 (Fedora 45)

- File: `htop-3.5.3-1.fc45.x86_64.rpm`
- Official package page: <https://packages.fedoraproject.org/pkgs/htop/htop/fedora-45.html>
- Signed Fedora 45 Beta repository download: <https://dl.fedoraproject.org/pub/fedora/linux/releases/test/45_Beta/Everything/x86_64/os/Packages/h/htop-3.5.3-1.fc45.x86_64.rpm>
- SHA-256: `a5f21195a6094f6abe43b97fb90554dc8617ff98dc8ee1ff92e85e6782719573`

The build verifies this digest before installing the RPM into the image.

## hwloc-libs 2.14.0 (Fedora 45)

- File: `hwloc-libs-2.14.0-2.fc45.x86_64.rpm`
- Official package page: <https://packages.fedoraproject.org/pkgs/hwloc/hwloc-libs/fedora-45.html>
- Signed Fedora 45 Beta repository download: <https://dl.fedoraproject.org/pub/fedora/linux/releases/test/45_Beta/Everything/x86_64/os/Packages/h/hwloc-libs-2.14.0-2.fc45.x86_64.rpm>
- SHA-256: `6e187ec895352dcd52d46a9cb0299b83389acae2d8051b93811b78720838d464`

This is htop's required hardware-topology runtime library. Both RPMs carry a
valid Fedora 45 signing-key signature (`4f50a6114cd5c6976a7f1179655a4b02f577861e`), and their digests are verified before the build transaction.
