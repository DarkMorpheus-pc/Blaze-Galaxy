#!/usr/bin/env bash
# BlazeOS ISO Release & Verification Script
# Generates SHA256 checksums and GPG cryptographic detached signatures for release ISOs.
set -euo pipefail

ISO_PATH="${1:-Blaze-SolarEvolution-5-x86_64.iso}"
GNUPGHOME_HOST="${BLAZE_GNUPGHOME:-$HOME/.blaze-signing}"
KEYID="${BLAZE_KEYID:-Blaze Release}"

if [ ! -f "$ISO_PATH" ]; then
    echo "Hata: ISO dosyası bulunamadı: $ISO_PATH" >&2
    exit 1
fi

echo "==> [1/3] SHA256 Özeti Hesaplanıyor: $ISO_PATH"
sha256sum "$ISO_PATH" > "${ISO_PATH}.sha256"
cat "${ISO_PATH}.sha256"

echo "==> [2/3] GPG Kriptografik İmzası Üretiliyor..."
if [ -d "$GNUPGHOME_HOST" ]; then
    GNUPGHOME="$GNUPGHOME_HOST" gpg --batch --yes --pinentry-mode loopback -u "$KEYID" --detach-sign --armor --output "${ISO_PATH}.sig" "$ISO_PATH"
    echo "İmza oluşturuldu: ${ISO_PATH}.sig"
else
    echo "Uyarı: GPG anahtarlık dizini ($GNUPGHOME_HOST) bulunamadı. İmza atlanıyor."
fi

echo "==> [3/3] Doğrulama Kontrolü:"
sha256sum -c "${ISO_PATH}.sha256"
if [ -f "${ISO_PATH}.sig" ] && [ -d "$GNUPGHOME_HOST" ]; then
    GNUPGHOME="$GNUPGHOME_HOST" gpg --verify "${ISO_PATH}.sig" "$ISO_PATH"
fi
echo "✓ Doğrulama Başarılı!"
