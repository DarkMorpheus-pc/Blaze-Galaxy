# Third-party binary provenance

## Limine 12.9.0

The files in `/usr/share/limine` and the `/usr/bin/limine` host utility come
from the immutable official Limine 12.9.0 binary release:

- Release: <https://github.com/Limine-Bootloader/Limine/releases/tag/v12.9.0>
- Asset: `limine-binary.tar.xz`
- Upstream SHA-256: `9a738586bff5790bd8bfef4a4868a2939cba3f81f22f121306d668c97f1c85d8`
- License: BSD-2-Clause, installed at `/usr/share/licenses/limine/LICENSE`

The host utility is compiled from the release asset's single-file `limine.c`
source. BlazeOS does not enable a COPR or download Limine during installation.
