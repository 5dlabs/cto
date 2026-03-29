#!/usr/bin/env python3
"""Build public/5dlabs-logo-3d.png: rembg cutout + removal of soft nebula / matte haze.

Requires: pip install rembg onnxruntime
"""
from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
from numpy.lib.stride_tricks import sliding_window_view
from PIL import Image
from rembg import remove

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path(__file__).resolve().parent / "source-logo.png"
OUT = ROOT / "public" / "5dlabs-logo-3d.png"


def local_luma_variance(L255: np.ndarray, ksize: int = 11) -> np.ndarray:
    pad = ksize // 2
    Lp = np.pad(L255, pad, mode="edge")
    win = sliding_window_view(Lp, (ksize, ksize))
    return win.var(axis=(2, 3))


def strip_nebula_haze(rgba: np.ndarray) -> np.ndarray:
    """Drop smooth purple mist / smoke haze rembg often keeps as foreground.

    Keeps sharp 3D facets, lightning, portal (high local contrast). Skips the bottom
    band so any embedded 5D LABS wordmark in the source raster is not erased.
    """
    h, w = rgba.shape[:2]
    yy, xx = np.mgrid[0:h, 0:w]
    rgb = rgba[:, :, :3].astype(np.float64) / 255.0
    mx = np.max(rgb, axis=2)
    V = mx
    L255 = (
        0.2126 * rgb[:, :, 0] + 0.7152 * rgb[:, :, 1] + 0.0722 * rgb[:, :, 2]
    ) * 255.0
    lv = local_luma_variance(L255, 11)
    alpha = rgba[:, :, 3].astype(np.float32)

    # Only strip haze above the wordmark strip (avoids eating embedded bottom text in the asset).
    upper = yy < (h * 0.78)

    # Base: soft diffuse nebula — low micro-contrast, low value, mid luma.
    nebula = (
        upper
        & (lv < 58.0)
        & (V < 0.28)
        & (L255 > 22.0)
        & (L255 < 128.0)
        & (alpha > 40.0)
    )
    # Wide smear behind the left glyph.
    left = xx < (w * 0.48)
    wisp = (
        upper
        & left
        & (lv < 72.0)
        & (V < 0.33)
        & (L255 > 25.0)
        & (L255 < 120.0)
        & (alpha > 40.0)
    )
    # Stronger wipe for the far-left nebula tail (still above wordmark).
    left_tight = xx < (w * 0.38)
    left_kill = (
        upper
        & left_tight
        & (lv < 95.0)
        & (V < 0.42)
        & (L255 > 18.0)
        & (L255 < 138.0)
        & (alpha > 40.0)
    )

    kill = nebula | wisp | left_kill
    rgba = rgba.copy()
    rgba[kill, 3] = 0
    rgba[kill, :3] = 0
    return rgba


def crop_to_alpha_box(im: Image.Image, thresh: int = 10, pad: int = 8) -> Image.Image:
    """Remove empty transparent margins so the hero raster sits tight above HTML wordmark."""
    arr = np.array(im.convert("RGBA"))
    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > thresh)
    if len(ys) == 0:
        return im
    h, w = alpha.shape
    y0, y1 = int(ys.min()), int(ys.max())
    x0, x1 = int(xs.min()), int(xs.max())
    x0c = max(0, x0 - pad)
    y0c = max(0, y0 - pad)
    x1c = min(w, x1 + pad + 1)
    y1c = min(h, y1 + pad + 1)
    return im.crop((x0c, y0c, x1c, y1c))


def main() -> None:
    src = SOURCE if SOURCE.exists() else Path(sys.argv[1]) if len(sys.argv) > 1 else None
    if src is None or not src.exists():
        print("Usage: export-hero-logo.py [source.png] (default: scripts/source-logo.png)", file=sys.stderr)
        sys.exit(1)
    im = Image.open(src).convert("RGBA")
    cut = remove(im)
    arr = np.array(cut)
    arr = strip_nebula_haze(arr)
    im_out = Image.fromarray(arr)
    im_out = crop_to_alpha_box(im_out)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    im_out.save(OUT, optimize=True)
    print(f"Wrote {OUT} ({im_out.size[0]}×{im_out.size[1]})")


if __name__ == "__main__":
    main()
