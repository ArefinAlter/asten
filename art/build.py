#!/usr/bin/env python3
"""Build the painted artwork for apps/www from public-domain sources.

    python art/build.py            # fetch (cached), process, write apps/www/public/art
    python art/build.py hero thoth # only these assets

Sources, licences and credits are in art/SOURCES.md. Raw downloads go to
art/.cache (ignored by git); only the processed WebP files are committed.
Needs Python 3.10+, Pillow, NumPy and opencv-python, and curl on the PATH.
Every random choice is seeded, so the same sources give the same files.
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import urllib.parse
from pathlib import Path

import cv2
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
CACHE = Path(os.environ.get("ASTEN_ART_CACHE", ROOT / "art" / ".cache"))
OUT = ROOT / "apps" / "www" / "public" / "art"
LOCK = ROOT / "art" / "sources.lock.json"
UA = "asten-art-build/1.0 (open-access sources; info@vernesystems.com)"

# Warm paper white the whole set is pulled towards, so every plate shares one light.
PAPER = np.array([245, 238, 223], np.float32)
LIGHT = np.array([252, 244, 228], np.float32)

# ───────────────────────── sources ─────────────────────────

MET = {
    "kensett": 11326,   # John Frederick Kensett, Sunset Sky, 1872
    "sargent": 12061,   # John Singer Sargent, Cliffs at Deir el Bahri, Egypt, 1890-91
    "denis": 438642,    # Simon Denis, Cloud Study (Early Evening), c. 1786-1806
    "menna": 548574,    # Charles K. Wilkinson, Harvest Scenes, Tomb of Menna
    "palette": 544302,  # Scribe's Palette and Brushes
    "statue": 544692,   # Haremhab as a Scribe of the King
    "lamp": 246975,     # Bronze lamp
}
COMMONS = {
    "hunefer": ("The judgement of the dead in the presence of Osiris.jpg", 2446),
    "ibis": ("Zoologie. Oiseau. 1. Ibis blanc ou sacré (Ibis religiosa); 2. Ibis noir (Ibis falcinellus) (NYPL b14212718-1268479).jpg", 3840),
}


def sh(args: list[str]) -> bytes:
    return subprocess.run(args, capture_output=True, check=True).stdout


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fetch(name: str) -> Path:
    """Download a source once; verify it against art/sources.lock.json."""
    CACHE.mkdir(parents=True, exist_ok=True)
    path = CACHE / f"{name}.jpg"
    if not path.exists():
        if name in MET:
            meta = json.loads(sh(["curl", "-s", "-m", "60", "-A", UA, f"https://collectionapi.metmuseum.org/public/collection/v1/objects/{MET[name]}"]))
            if not meta.get("isPublicDomain"):
                raise SystemExit(f"{name}: Met object {MET[name]} is not public domain")
            url = meta["primaryImage"]
        else:
            title, width = COMMONS[name]
            url = "https://commons.wikimedia.org/wiki/Special:FilePath/" + urllib.parse.quote(title) + f"?width={width}"
        data = sh(["curl", "-s", "-L", "-m", "240", "-A", UA, url])
        path.write_bytes(data)
    lock = json.loads(LOCK.read_text()) if LOCK.exists() else {}
    digest = sha256(path)
    if name in lock and lock[name] != digest:
        print(f"warning: {name} differs from art/sources.lock.json", file=sys.stderr)
    lock.setdefault(name, digest)
    LOCK.write_text(json.dumps(dict(sorted(lock.items())), indent=2) + "\n")
    return path


def load(name: str) -> Image.Image:
    return Image.open(fetch(name)).convert("RGB")


# ───────────────────────── image craft ─────────────────────────


def f32(img: Image.Image) -> np.ndarray:
    return np.asarray(img.convert("RGB"), np.float32)


def to_img(a: np.ndarray) -> Image.Image:
    return Image.fromarray(np.clip(a, 0, 255).astype(np.uint8), "RGB")


def kuwahara(a: np.ndarray, r: int) -> np.ndarray:
    """Quadrant-mean (Kuwahara) filter: flattens print halftone and scan noise into brush-sized patches
    while keeping edges. a is float32 HxWx3."""
    k = r + 1
    gray = a.mean(axis=2)
    best_var = None
    out = np.zeros_like(a)
    for anchor in ((k - 1, k - 1), (0, k - 1), (k - 1, 0), (0, 0)):
        mean = cv2.boxFilter(a, -1, (k, k), anchor=anchor, borderType=cv2.BORDER_REFLECT)
        g = cv2.boxFilter(gray, -1, (k, k), anchor=anchor, borderType=cv2.BORDER_REFLECT)
        g2 = cv2.boxFilter(gray * gray, -1, (k, k), anchor=anchor, borderType=cv2.BORDER_REFLECT)
        var = g2 - g * g
        if best_var is None:
            best_var, out = var, mean
        else:
            take = var < best_var
            best_var = np.where(take, var, best_var)
            out = np.where(take[..., None], mean, out)
    return out


def grade(a: np.ndarray, alpha: np.ndarray | None = None, sat: float = 0.9, lift: float = 0.035, light: float = 0.65,
          contrast: float = 1.0, tint: tuple[float, float, float] = (1.0, 0.985, 0.95)) -> np.ndarray:
    """One grade for the whole set: gain the highlights toward warm paper light, tame saturation, lift the
    blacks a little (matte), and warm the cast. alpha (0-1) limits the statistics to the subject."""
    sel = (alpha > 0.5) if alpha is not None else np.ones(a.shape[:2], bool)
    px = a[sel]
    hi = np.percentile(px, 96, axis=0).astype(np.float32)
    gain = np.clip(LIGHT / np.maximum(hi, 1), 0.85, 1.18)
    gain = 1 + (gain - 1) * light
    b = a * gain
    grey = b @ np.array([0.299, 0.587, 0.114], np.float32)
    b = grey[..., None] + (b - grey[..., None]) * sat
    b = (b - 128) * contrast + 128
    b = b * (1 - lift) + np.array([34, 27, 20], np.float32) * lift * 1.0 + b * lift * 0.0
    b = b * np.array(tint, np.float32)
    return b


def fbm_1d(n: int, rng: np.random.RandomState, base: float, octaves: int = 5) -> np.ndarray:
    """1-D fractal value noise in [-1, 1]."""
    out = np.zeros(n, np.float32)
    amp, freq, total = 1.0, 1.0 / base, 0.0
    for _ in range(octaves):
        pts = rng.uniform(-1, 1, int(n * freq) + 3).astype(np.float32)
        xs = np.linspace(0, len(pts) - 3, n)
        out += amp * np.interp(xs, np.arange(len(pts)), pts)
        total += amp
        amp *= 0.5
        freq *= 2.1
    return out / total


def torn_alpha(w: int, h: int, seed: int, amp: float = 12.0, inset: float = 16.0, base: float = 46.0) -> np.ndarray:
    """A deckled, hand-torn rectangle: each side wanders by fractal noise, plus a fibrous fringe."""
    rng = np.random.RandomState(seed)
    S = 2
    pts = []
    xs = np.arange(0, w, 2)
    ys = np.arange(0, h, 2)
    top = inset + amp * fbm_1d(len(xs), rng, base) + 1.6 * fbm_1d(len(xs), rng, 6, 2)
    right = (w - inset) - amp * (0.6 + 0.4 * fbm_1d(len(ys), rng, base)) - 1.6 * np.abs(fbm_1d(len(ys), rng, 6, 2))
    bottom = (h - inset) - amp * (0.6 + 0.4 * fbm_1d(len(xs), rng, base)) - 1.6 * np.abs(fbm_1d(len(xs), rng, 6, 2))
    left = inset + amp * fbm_1d(len(ys), rng, base) + 1.6 * fbm_1d(len(ys), rng, 6, 2)
    pts += [(x, t) for x, t in zip(xs, top)]
    pts += [(r, y) for y, r in zip(ys, right)]
    pts += [(x, b) for x, b in zip(xs[::-1], bottom[::-1])]
    pts += [(l, y) for y, l in zip(ys[::-1], left[::-1])]
    m = np.zeros((h * S, w * S), np.uint8)
    cv2.fillPoly(m, [np.array(pts, np.float32).reshape(-1, 1, 2).__mul__(S).astype(np.int32)], 255)
    m = cv2.resize(m, (w, h), interpolation=cv2.INTER_AREA)
    return m.astype(np.float32) / 255.0


def torn(img: Image.Image, seed: int, amp: float = 12.0, inset: float = 16.0, rim: float = 0.6) -> Image.Image:
    """Return an RGBA plate with a torn edge whose last few pixels fade toward bare paper fibre."""
    a = f32(img)
    h, w = a.shape[:2]
    al = torn_alpha(w, h, seed, amp, inset)
    hard = (al > 0.5).astype(np.uint8)
    edge = hard - cv2.erode(hard, np.ones((3, 3), np.uint8), iterations=2)
    edge = cv2.GaussianBlur(edge.astype(np.float32), (0, 0), 0.8) * rim
    a = a * (1 - edge[..., None]) + PAPER * edge[..., None]
    rgba = np.dstack([a, al * 255])
    return Image.fromarray(np.clip(rgba, 0, 255).astype(np.uint8), "RGBA")


def save_webp(img: Image.Image, name: str, quality: int = 80, method: int = 6) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / name
    img.save(path, "WEBP", quality=quality, method=method)
    print(f"{name:26s} {img.size[0]:>5}x{img.size[1]:<5} {path.stat().st_size / 1024:6.1f} KB")


def fit(img: Image.Image, width: int) -> Image.Image:
    return img.resize((width, round(img.height * width / img.width)), Image.LANCZOS)


def grain(a: np.ndarray, seed: int, strength: float = 5.0, size: float = 1.0) -> np.ndarray:
    """Fine canvas-like grain, luminance only, so the plates take the site's paper."""
    rng = np.random.RandomState(seed)
    n = rng.normal(0, 1, a.shape[:2]).astype(np.float32)
    if size > 1:
        n = cv2.GaussianBlur(n, (0, 0), size)
        n /= max(n.std(), 1e-3)
    return a + n[..., None] * strength


def studio_cut(a: np.ndarray, sure: float = 38.0, maybe: float = 15.0, floor: float = 7.0, iters: int = 5) -> np.ndarray:
    """Cut an object photographed on a studio sweep. The backdrop is modelled per row from the left and right
    margins (the sweep only changes vertically), then GrabCut settles the pixels that are neither clearly
    backdrop nor clearly object."""
    u8 = np.clip(a, 0, 255).astype(np.uint8)
    lab = cv2.cvtColor(u8, cv2.COLOR_RGB2LAB).astype(np.float32)
    h, w = lab.shape[:2]
    m = max(24, w // 50)
    edge = np.concatenate([lab[:, :m], lab[:, -m:]], axis=1)
    bg = np.median(edge, axis=1)  # h x 3
    bg = cv2.GaussianBlur(bg[:, None, :], (0, 0), 6)[:, 0, :]
    dist = np.linalg.norm(lab - bg[:, None, :], axis=2)
    mask = np.full((h, w), cv2.GC_PR_BGD, np.uint8)
    mask[dist > maybe] = cv2.GC_PR_FGD
    mask[dist > sure] = cv2.GC_FGD
    mask[dist < floor] = cv2.GC_BGD
    mask[:3, :] = cv2.GC_BGD
    mask[-3:, :] = cv2.GC_BGD
    mask[:, :3] = cv2.GC_BGD
    mask[:, -3:] = cv2.GC_BGD
    bgr = cv2.cvtColor(u8, cv2.COLOR_RGB2BGR)
    bgd = np.zeros((1, 65), np.float64)
    fgd = np.zeros((1, 65), np.float64)
    cv2.grabCut(bgr, mask, None, bgd, fgd, iters, cv2.GC_INIT_WITH_MASK)
    fg = ((mask == cv2.GC_FGD) | (mask == cv2.GC_PR_FGD)).astype(np.uint8)
    n, lcc, stats, _ = cv2.connectedComponentsWithStats(fg, 8)
    keep = [i for i in range(1, n) if stats[i, cv2.CC_STAT_AREA] > 0.004 * fg.size]
    fg = np.isin(lcc, keep).astype(np.uint8)
    fg = cv2.morphologyEx(fg, cv2.MORPH_CLOSE, np.ones((7, 7), np.uint8))
    inv = 1 - fg
    n, lcc, stats, _ = cv2.connectedComponentsWithStats(inv, 4)
    for i in range(1, n):
        x, y, ww, hh, area = stats[i]
        if area < 0.01 * fg.size and not (x == 0 or y == 0 or x + ww == w or y + hh == h):
            fg[lcc == i] = 1
    return fg


def soft_alpha(fg: np.ndarray, blur: float = 1.2, lo: float = 0.35, hi: float = 0.75, shrink: int = 1) -> np.ndarray:
    """Feather a binary cut: pull in 'shrink' px (no halo of the old backdrop), blur, re-threshold into a smooth ramp."""
    if shrink:
        fg = cv2.erode(fg, np.ones((3, 3), np.uint8), iterations=shrink)
    al = cv2.GaussianBlur(fg.astype(np.float32), (0, 0), blur)
    return np.clip((al - lo) / (hi - lo), 0, 1)


def rgba(a: np.ndarray, alpha: np.ndarray) -> Image.Image:
    return Image.fromarray(np.dstack([np.clip(a, 0, 255), alpha * 255]).astype(np.uint8), "RGBA")


def crop_to_alpha(img: Image.Image, pad: int = 6) -> Image.Image:
    bbox = img.getchannel("A").point(lambda v: 255 if v > 8 else 0).getbbox()
    x0, y0, x1, y1 = bbox
    return img.crop((max(0, x0 - pad), max(0, y0 - pad), min(img.width, x1 + pad), min(img.height, y1 + pad)))


# ───────────────────────── the assets ─────────────────────────


def build_hero() -> None:
    """The arched window: Kensett's sunset over Sargent's cliffs at Deir el-Bahri."""
    CW, CH = 1040, 1210
    kens = load("kensett")
    sarg = load("sargent")
    # Sky: a portrait slice from the top of Sunset Sky, down past the gold band.
    sx, slice_w, slice_h = 520, 1520, 1215
    sky = fit(kens.crop((sx, 0, sx + slice_w, slice_h)), CW)
    canvas = np.zeros((CH, CW, 3), np.float32)
    canvas[: sky.height] = f32(sky)
    canvas[sky.height:] = canvas[sky.height - 6].mean(axis=0)
    # Cliffs: Sargent's oil sketch with its own sky taken out.
    sw, sh_ = sarg.size
    cl = fit(sarg.crop((int(0.26 * sw), int(0.08 * sh_), int(0.99 * sw), int(0.96 * sh_))), CW)
    cla = f32(cl)
    lab = cv2.cvtColor(cla.astype(np.uint8), cv2.COLOR_RGB2LAB).astype(np.float32)
    L, B = lab[..., 0], lab[..., 2] - 128
    skylike = ((B < 9) & (L > 120)).astype(np.uint8)
    n, comp, _, _ = cv2.connectedComponentsWithStats(skylike, 4)
    top = set(comp[0:6, :].ravel()) - {0}
    sky_mask = np.isin(comp, list(top)).astype(np.uint8)
    # the hazy valley on the left is sky too
    haze = np.array([(0, 0), (300, 0), (480, 60), (440, 100), (350, 180), (230, 212), (0, 228)], np.int32)
    cv2.fillPoly(sky_mask, [haze], 1)
    sky_mask = cv2.morphologyEx(sky_mask, cv2.MORPH_CLOSE, np.ones((9, 9), np.uint8))
    sky_mask = cv2.GaussianBlur(sky_mask.astype(np.float32), (0, 0), 2.6)
    alpha = 1 - np.clip((sky_mask - 0.3) / 0.4, 0, 1)
    alpha[:6] = 0
    y_off = CH - cl.height + 40
    ys, hh = max(0, y_off), min(CH - max(0, y_off), cl.height)
    A = np.zeros((CH, CW), np.float32)
    C = np.zeros((CH, CW, 3), np.float32)
    A[ys:ys + hh] = alpha[:hh]
    C[ys:ys + hh] = cla[:hh]
    out = canvas * (1 - A[..., None]) + C * A[..., None]
    # Horizon glow where the gold band meets the rock, and a low evening vignette.
    yy, xx = np.mgrid[0:CH, 0:CW].astype(np.float32)
    glow = np.exp(-((yy - 0.52 * CH) ** 2) / (2 * (0.07 * CH) ** 2)) * (1 - A) * 0.35
    out = out + glow[..., None] * np.array([40, 22, 0], np.float32)
    vig = 1 - 0.28 * np.clip(((xx - CW / 2) / (CW * 0.75)) ** 2 + ((yy - CH * 0.45) / (CH * 0.8)) ** 2, 0, 1)
    out = out * vig[..., None]
    out = kuwahara(out, 2) * 0.55 + out * 0.45
    out = grade(out, sat=0.92, lift=0.03, light=0.5)
    out = grain(out, 11, 3.2, 1.0)
    save_webp(to_img(out), "hero-window.webp", quality=74)


def cut_hunefer_thoth() -> tuple[np.ndarray, np.ndarray]:
    """Thoth from the Papyrus of Hunefer (British Museum EA 9901): cut along his own outline."""
    im = np.asarray(load("hunefer"))[..., ::-1].copy()  # BGR
    X0, Y0, X1, Y1 = 985, 455, 1250, 965
    crop = im[Y0:Y1, X0:X1].copy()
    h, w = crop.shape[:2]
    lab = cv2.cvtColor(crop, cv2.COLOR_BGR2LAB).astype(np.float32)
    gray = cv2.cvtColor(crop, cv2.COLOR_BGR2GRAY)
    ground = np.median(lab[420:480, 10:50].reshape(-1, 3), axis=0)
    dist = np.linalg.norm(lab - ground, axis=2)
    outline = cv2.dilate((gray < 120).astype(np.uint8), np.ones((3, 3), np.uint8))
    cand = ((dist < 22) & (outline == 0)).astype(np.uint8)
    cand = cv2.morphologyEx(cand, cv2.MORPH_OPEN, np.ones((2, 2), np.uint8))
    n, comp, _, _ = cv2.connectedComponentsWithStats(cand, 4)
    bg = np.zeros((h, w), np.uint8)
    for lbl in set(comp[0, :]) | set(comp[-1, :]) | set(comp[:, 0]) | set(comp[:, -1]):
        if lbl != 0:
            bg[comp == lbl] = 1
    fg = 1 - bg

    def O(x, y):
        return (x - X0, y - Y0)

    gap = np.array([O(*p) for p in [(1133, 801), (1158, 799), (1165, 822), (1174, 850), (1181, 880), (1185, 910), (1187, 935),
                                    (1160, 946), (1136, 952), (1108, 952), (1093, 945), (1099, 915), (1108, 880), (1120, 840), (1133, 805)]], np.int32)
    cv2.fillPoly(fg, [gap], 0)
    fg[O(0, 640)[1]:, O(1212, 0)[0]:] = 0
    fg[:, O(1238, 0)[0]:] = 0
    fg[: O(0, 474)[1], :] = 0
    fg[O(0, 620)[1]:, : O(1003, 0)[0]] = 0
    # keep a short ground line under the feet, drop the rest of the strip
    fg[O(0, 962)[1]:, :] = 0
    n, lcc, stats, _ = cv2.connectedComponentsWithStats(fg.astype(np.uint8), 8)
    fg = (lcc == 1 + np.argmax(stats[1:, cv2.CC_STAT_AREA])).astype(np.uint8)
    fg = cv2.morphologyEx(fg, cv2.MORPH_OPEN, np.ones((3, 3), np.uint8))
    n, lcc, stats, _ = cv2.connectedComponentsWithStats(fg, 8)
    fg = (lcc == 1 + np.argmax(stats[1:, cv2.CC_STAT_AREA])).astype(np.uint8)
    return cv2.cvtColor(crop, cv2.COLOR_BGR2RGB).astype(np.float32), fg


def build_thoth() -> None:
    rgb, fg = cut_hunefer_thoth()
    up = 2.2
    rgb_u = cv2.resize(rgb, None, fx=up, fy=up, interpolation=cv2.INTER_CUBIC)
    fg_u = cv2.resize(fg.astype(np.float32), None, fx=up, fy=up, interpolation=cv2.INTER_LINEAR)
    fg_u = (fg_u > 0.5).astype(np.uint8)
    alpha = soft_alpha(fg_u, blur=1.5, lo=0.3, hi=0.8, shrink=1)
    # a brush-sized smoothing so the upscale reads as paint, then re-sharpen the ink lines
    sm = kuwahara(rgb_u, 2)
    soft = cv2.GaussianBlur(sm, (0, 0), 1.2)
    rgb_u = sm + (sm - soft) * 0.9
    rgb_u = grade(rgb_u, alpha, sat=0.95, lift=0.02, light=0.55)
    rgb_u = grain(rgb_u, 5, 2.4, 0.9)
    img = crop_to_alpha(rgba(rgb_u, alpha), 8)
    save_webp(img, "thoth.webp", quality=86)


def build_scribes() -> None:
    """Surveyors and their boy-scribes with writing boards, from the Tomb of Menna (Wilkinson's facsimile)."""
    img = load("menna")
    w, h = img.size
    c = img.crop((int(0.075 * w), int(0.045 * h), int(0.46 * w), int(0.49 * h)))
    c = fit(c, 1200)
    a = f32(c)
    a = kuwahara(a, 1) * 0.5 + a * 0.5
    a = grade(a, None, sat=0.95, light=0.5, lift=0.03)
    a = grain(a, 21, 3.0, 1.0)
    save_webp(torn(to_img(a), 31, amp=9, inset=12), "plate-scribes.webp", quality=78)


def build_palette() -> None:
    img = load("palette")
    a = f32(img)
    h, w = a.shape[:2]
    fg = studio_cut(a)
    alpha = soft_alpha(fg, 2.0, 0.3, 0.8, 2)
    a = grade(a, alpha, sat=0.9, light=0.6, lift=0.02)
    a = kuwahara(a, 2) * 0.6 + a * 0.4
    a = grain(a, 41, 2.0, 1.0)
    out = crop_to_alpha(rgba(a, alpha), 14)
    save_webp(fit(out, 1100), "plate-palette.webp", quality=84)


def build_weighing() -> None:
    img = load("hunefer")
    w, h = img.size
    c = img.crop((int(0.115 * w), int(0.225 * h), int(0.675 * w), int(0.985 * h)))
    c = fit(c, 960)
    a = f32(c)
    a = kuwahara(a, 1) * 0.5 + a * 0.5
    a = grade(a, None, sat=0.95, light=0.55, lift=0.025)
    a = grain(a, 51, 2.6, 1.0)
    save_webp(torn(to_img(a), 61, amp=8, inset=10), "plate-weighing.webp", quality=80)


def build_dusk() -> None:
    img = load("denis")
    w, h = img.size
    c = fit(img.crop((int(0.03 * w), int(0.04 * h), int(0.97 * w), int(0.95 * h))), 960)
    a = f32(c)
    a = grade(a, None, sat=0.9, light=0.6, lift=0.03, tint=(1.03, 0.99, 0.93))
    a = grain(a, 71, 3.0, 1.0)
    save_webp(torn(to_img(a), 81, amp=9, inset=12), "plate-dusk.webp", quality=78)


def build_cliffs() -> None:
    img = load("sargent")
    w, h = img.size
    c = fit(img.crop((int(0.02 * w), int(0.04 * h), int(0.98 * w), int(0.96 * h))), 1000)
    a = f32(c)
    a = grade(a, None, sat=0.92, light=0.55, lift=0.03)
    a = grain(a, 91, 2.8, 1.0)
    save_webp(torn(to_img(a), 101, amp=8, inset=10), "plate-cliffs.webp", quality=78)


def largest(fg: np.ndarray, open_px: int = 0) -> np.ndarray:
    if open_px:
        fg = cv2.morphologyEx(fg, cv2.MORPH_OPEN, np.ones((open_px, open_px), np.uint8))
    n, lcc, stats, _ = cv2.connectedComponentsWithStats(fg, 8)
    return (lcc == 1 + np.argmax(stats[1:, cv2.CC_STAT_AREA])).astype(np.uint8)


def build_lamp() -> None:
    img = load("lamp")
    a = f32(img)
    h, w = a.shape[:2]
    fg = largest(studio_cut(a), 25)
    alpha = soft_alpha(fg, 2.0, 0.3, 0.8, 2)
    a = grade(a, alpha, sat=0.85, light=0.6, lift=0.03, tint=(1.04, 0.99, 0.92))
    a = kuwahara(a, 2) * 0.6 + a * 0.4
    a = grain(a, 111, 2.2, 1.0)
    out = crop_to_alpha(rgba(a, alpha), 14)
    save_webp(fit(out, 760), "plate-lamp.webp", quality=84)


def build_statue() -> None:
    img = load("statue")
    a = f32(img)
    h, w = a.shape[:2]
    fg = largest(studio_cut(a), 9)
    fg[int(0.935 * h):] = 0  # the museum's plinth, not the statue
    alpha = soft_alpha(fg, 2.0, 0.3, 0.8, 2)
    a = grade(a, alpha, sat=1.25, light=0.75, lift=0.03, tint=(1.1, 0.99, 0.86))
    a = kuwahara(a, 2) * 0.6 + a * 0.4
    a = grain(a, 121, 2.2, 1.0)
    out = crop_to_alpha(rgba(a, alpha), 14)
    save_webp(fit(out, 560), "plate-scribe.webp", quality=84)


def build_ibis() -> None:
    """The sacred ibis (Ibis religiosa) from Savigny's plate in the Description de l'Egypte, 1809: the
    neighbouring black ibis is removed, the engraving is toned to the site's paper."""
    img = load("ibis")
    w, h = img.size
    full = np.asarray(fit(img, 1400), np.float32)
    grey = full @ np.array([0.299, 0.587, 0.114], np.float32)
    dark = cv2.morphologyEx((grey < 150).astype(np.uint8), cv2.MORPH_CLOSE, np.ones((5, 5), np.uint8))
    n, lcc, _, _ = cv2.connectedComponentsWithStats(dark, 8)
    black_ibis = lcc[430, 480]  # a point on the black ibis's body
    gone = cv2.dilate((lcc == black_ibis).astype(np.uint8), np.ones((15, 15), np.uint8)) if black_ibis else np.zeros_like(dark)
    # paper colour from the plate's own margin, so the patch is invisible
    paper = np.median(full[20:60, 700:1300].reshape(-1, 3), axis=0)
    full[gone == 1] = paper
    full = cv2.GaussianBlur(full, (0, 0), 0.4)
    c = fit(to_img(full).crop((int(0.368 * 1400), int(0.02 * full.shape[0]), 1392, int(0.955 * full.shape[0]))), 940)
    a = f32(c)
    g = a @ np.array([0.299, 0.587, 0.114], np.float32)
    ink = np.clip((g - 40) / 200, 0, 1)
    paper_c = np.array([244, 236, 218], np.float32)
    shade = np.array([52, 42, 33], np.float32)
    a = paper_c * ink[..., None] + shade * (1 - ink[..., None])
    a = grain(a, 131, 2.4, 1.0)
    save_webp(torn(to_img(a), 141), "plate-ibis.webp", quality=78)


def build_grain() -> None:
    """A seamless paper-fibre tile (greyscale, transparent) for the page background."""
    rng = np.random.RandomState(7)
    S = 128
    n = np.zeros((S, S), np.float32)
    for sigma, amp in ((0.7, 1.0), (1.6, 0.7), (5.0, 0.22), (16.0, 0.08)):
        g = rng.normal(0, 1, (S, S)).astype(np.float32)
        g = cv2.GaussianBlur(np.tile(g, (3, 3)), (0, 0), sigma)[S:2 * S, S:2 * S]
        n += amp * g / g.std()
    # a few long fibres
    fib = np.zeros((S * 3, S * 3), np.float32)
    for _ in range(70):
        x, y = rng.randint(0, S * 3), rng.randint(0, S * 3)
        ang = rng.uniform(0, np.pi)
        ln = rng.randint(8, 30)
        cv2.line(fib, (x, y), (int(x + ln * np.cos(ang)), int(y + ln * np.sin(ang))), float(rng.uniform(0.4, 1.0)), 1, cv2.LINE_AA)
    fib = cv2.GaussianBlur(fib, (0, 0), 0.6)
    fibs = np.zeros((S, S), np.float32)
    for dy in range(3):
        for dx in range(3):
            fibs += fib[dy * S:(dy + 1) * S, dx * S:(dx + 1) * S]
    n = (n / n.std()) * 0.5 + fibs * 1.2
    n = (n - n.mean()) / n.std()
    dark = np.clip(n, 0, None)
    light = np.clip(-n, 0, None)
    a = np.zeros((S, S, 4), np.uint8)
    # one alpha-tile that darkens a little and lightens a little: ink-brown specks and pale fibre
    a[..., 0:3] = np.where((n > 0)[..., None], np.array([60, 44, 28]), np.array([255, 250, 238]))
    a[..., 3] = np.clip(np.where(n > 0, dark * 9, light * 12), 0, 30).astype(np.uint8)
    OUT.mkdir(parents=True, exist_ok=True)
    img = Image.fromarray(a, "RGBA")
    img.save(OUT / "grain.webp", "WEBP", quality=50, method=6)
    print(f"{'grain.webp':26s} {S:>5}x{S:<5} {(OUT / 'grain.webp').stat().st_size / 1024:6.1f} KB")


ASSETS = {
    "hero": build_hero,
    "thoth": build_thoth,
    "scribes": build_scribes,
    "palette": build_palette,
    "weighing": build_weighing,
    "dusk": build_dusk,
    "cliffs": build_cliffs,
    "lamp": build_lamp,
    "statue": build_statue,
    "ibis": build_ibis,
    "grain": build_grain,
}

if __name__ == "__main__":
    wanted = sys.argv[1:] or list(ASSETS)
    for key in wanted:
        if key not in ASSETS:
            raise SystemExit(f"unknown asset {key!r}; choose from {', '.join(ASSETS)}")
        ASSETS[key]()
