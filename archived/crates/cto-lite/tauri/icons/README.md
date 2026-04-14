# CTO Lite App Icons

## Required Files

### Cross-Platform
| File | Size | Format | Purpose |
|------|------|--------|---------|
| `icon.png` | 1024x1024 | PNG | Source icon |
| `32x32.png` | 32x32 | PNG | Small icon |
| `128x128.png` | 128x128 | PNG | Medium icon |
| `128x128@2x.png` | 256x256 | PNG | Retina medium icon |

### macOS
| File | Format | Purpose |
|------|--------|---------|
| `icon.icns` | ICNS | macOS app icon (contains multiple sizes) |

### Windows
| File | Format | Purpose |
|------|--------|---------|
| `icon.ico` | ICO | Windows app icon (contains multiple sizes) |
| `nsis-header.bmp` | BMP (150x57) | NSIS installer header image |
| `nsis-sidebar.bmp` | BMP (164x314) | NSIS installer sidebar image |

## Generating Icons

### From source PNG (1024x1024)

```bash
# Install ImageMagick
brew install imagemagick

# Generate PNGs
convert icon.png -resize 32x32 32x32.png
convert icon.png -resize 128x128 128x128.png
convert icon.png -resize 256x256 128x128@2x.png

# Generate ICO (Windows)
convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico

# Generate ICNS (macOS) - requires iconutil
mkdir icon.iconset
sips -z 16 16     icon.png --out icon.iconset/icon_16x16.png
sips -z 32 32     icon.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32     icon.png --out icon.iconset/icon_32x32.png
sips -z 64 64     icon.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128   icon.png --out icon.iconset/icon_128x128.png
sips -z 256 256   icon.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256   icon.png --out icon.iconset/icon_256x256.png
sips -z 512 512   icon.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512   icon.png --out icon.iconset/icon_512x512.png
sips -z 1024 1024 icon.png --out icon.iconset/icon_512x512@2x.png
iconutil -c icns icon.iconset
rm -rf icon.iconset

# NSIS images (Windows installer)
convert icon.png -resize 150x57 -gravity center -background white -extent 150x57 nsis-header.bmp
convert icon.png -resize 164x314 -gravity center -background white -extent 164x314 nsis-sidebar.bmp
```

## Design Guidelines

- **Style:** Clean, modern, minimal
- **Colors:** Match CTO brand (blue gradient preferred)
- **Shapes:** Rounded corners for consistency with modern OS icons
- **Contrast:** Must be visible on both light and dark backgrounds

## Current Status

⚠️ **Placeholder icons** - Real icons need to be designed and generated before release.
