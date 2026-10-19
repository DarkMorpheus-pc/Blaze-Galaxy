#!/usr/bin/env python3
import sys
from PIL import Image, ImageStat

# Allowed scheme types
SCHEMES = [
    "scheme-content",
    "scheme-expressive",
    "scheme-fidelity",
    "scheme-fruit-salad",
    "scheme-monochrome",
    "scheme-neutral",
    "scheme-rainbow",
    "scheme-tonal-spot"
]

def image_colorfulness(image):
    # Mean HSV saturation is sufficient for choosing a conservative Material
    # scheme and avoids shipping OpenCV/NumPy for this tiny operation.
    saturation = image.convert("RGB").convert("HSV").getchannel("S")
    return ImageStat.Stat(saturation).mean[0] * 100.0 / 255.0

# scheme-content respects the image's colors very well, but it might
# look too saturated, so we only use it for not very colorful images to be safe
def pick_scheme(colorfulness):
    if colorfulness < 40:
        return "scheme-neutral"
    else:
        return "scheme-tonal-spot"

def load_and_resize_image(img_path, max_dim=128):
    try:
        img = Image.open(img_path).convert("RGB")
    except (OSError, ValueError):
        return None
    w, h = img.size
    if max(h, w) > max_dim:
        scale = max_dim / max(h, w)
        img = img.resize((int(w * scale), int(h * scale)), Image.Resampling.LANCZOS)
    return img

def main():
    colorfulness_mode = False
    args = sys.argv[1:]
    if '--colorfulness' in args:
        colorfulness_mode = True
        args.remove('--colorfulness')
    if len(args) < 1:
        print("scheme-tonal-spot")
        sys.exit(1)
    img_path = args[0]
    img = load_and_resize_image(img_path)
    if img is None:
        print("scheme-tonal-spot")
        sys.exit(1)
    colorfulness = image_colorfulness(img)
    if colorfulness_mode:
        print(f"{colorfulness}")
    else:
        scheme = pick_scheme(colorfulness)
        print(scheme)

if __name__ == "__main__":
    main()
