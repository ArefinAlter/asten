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
import zlib
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

WORKS_FILE = ROOT / "art" / "works.json"
PLATES_FILE = ROOT / "art" / "plates.json"
MANIFEST = ROOT / "apps" / "www" / "src" / "personality" / "art-manifest.json"
WORKS: dict[str, dict] = json.loads(WORKS_FILE.read_text(encoding="utf-8"))["works"]


def sh(args: list[str]) -> bytes:
    return subprocess.run(args, capture_output=True, check=True).stdout


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fetch(name: str) -> Path:
    """Download a source once; verify it against art/sources.lock.json."""
    CACHE.mkdir(parents=True, exist_ok=True)
    path = CACHE / f"{name}.jpg"
    src = WORKS[name]["source"]
    if not path.exists():
        if "met" in src:
            meta = json.loads(sh(["curl", "-s", "-m", "60", "-A", UA, f"https://collectionapi.metmuseum.org/public/collection/v1/objects/{src['met']}"]))
            if not meta.get("isPublicDomain"):
                raise SystemExit(f"{name}: Met object {src['met']} is not public domain")
            url = meta["primaryImage"]
        else:
            url = "https://commons.wikimedia.org/wiki/Special:FilePath/" + urllib.parse.quote(src["commons"]) + f"?width={src['width']}"
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
    m = cv2.morphologyEx(m, cv2.MORPH_OPEN, cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (15, 15)))  # no spikes at the corners
    n, lcc, stats, _ = cv2.connectedComponentsWithStats((m > 127).astype(np.uint8), 8)
    if n > 2:  # a fold at a corner can leave a scrap of paper beside the sheet
        m = np.where(lcc == 1 + int(np.argmax(stats[1:, cv2.CC_STAT_AREA])), 255, 0).astype(np.uint8)
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


def largest(fg: np.ndarray, open_px: int = 0) -> np.ndarray:
    if open_px:
        fg = cv2.morphologyEx(fg, cv2.MORPH_OPEN, np.ones((open_px, open_px), np.uint8))
    n, lcc, stats, _ = cv2.connectedComponentsWithStats(fg, 8)
    return (lcc == 1 + np.argmax(stats[1:, cv2.CC_STAT_AREA])).astype(np.uint8)


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


def arch_alpha(w: int, h: int, seed: int, amp: float = 6.0, inset: float = 14.0) -> np.ndarray:
    """A round-topped window: a semicircle over a torn-paper body. The curve wanders a little, as if cut by hand."""
    ss = 2
    x0, x1 = inset, w - inset
    r = (x1 - x0) / 2
    cy = inset + r
    m = np.zeros((h * ss, w * ss), np.uint8)
    cv2.circle(m, (int(w * ss / 2), int(cy * ss)), int(r * ss), 255, -1)
    cv2.rectangle(m, (int(x0 * ss), int(cy * ss)), (int(x1 * ss), int((h - inset) * ss)), 255, -1)
    hard = cv2.resize(m, (w, h), interpolation=cv2.INTER_AREA) > 127
    hard8 = hard.astype(np.uint8)
    sd = cv2.distanceTransform(hard8, cv2.DIST_L2, 5) - cv2.distanceTransform(1 - hard8, cv2.DIST_L2, 5)
    rng = np.random.RandomState(seed)
    wob = rng.normal(0, 1, (h, w)).astype(np.float32)
    wob = cv2.GaussianBlur(wob, (0, 0), 18)
    wob /= max(wob.std(), 1e-3)
    fib = cv2.GaussianBlur(rng.normal(0, 1, (h, w)).astype(np.float32), (0, 0), 1.4)
    fib /= max(fib.std(), 1e-3)
    al = np.clip((sd + amp * wob + 1.2 * fib) / 1.6 + 0.5, 0, 1)
    # the foot of the sheet is torn like the others
    return np.minimum(al, torn_alpha(w, h, seed + 7, amp=amp, inset=inset))


def edge_to_paper(a: np.ndarray, al: np.ndarray, rim: float = 0.6) -> np.ndarray:
    """Pull the last pixels of a cut edge toward bare paper, so no dark rim shows."""
    hard = (al > 0.5).astype(np.uint8)
    edge = hard - cv2.erode(hard, np.ones((3, 3), np.uint8), iterations=2)
    edge = cv2.GaussianBlur(edge.astype(np.float32), (0, 0), 0.8) * rim
    return a * (1 - edge[..., None]) + PAPER * edge[..., None]


def tone(a: np.ndarray, paper: tuple[int, int, int] = (244, 236, 218), shade: tuple[int, int, int] = (52, 42, 33), lo: float = 40, hi: float = 240) -> np.ndarray:
    """Re-tone a black-and-white print: ink to a warm brown, paper to the site's cream."""
    g = a @ np.array([0.299, 0.587, 0.114], np.float32)
    ink = np.clip((g - lo) / (hi - lo), 0, 1)
    return np.array(paper, np.float32) * ink[..., None] + np.array(shade, np.float32) * (1 - ink[..., None])


def seed_of(key: str) -> int:
    return zlib.crc32(key.encode()) % 100000


def limit(img: Image.Image, longest: int) -> Image.Image:
    """Shrink so the longest side is at most `longest` (the cut is computed at this size)."""
    k = longest / max(img.size)
    return img if k >= 1 else img.resize((round(img.width * k), round(img.height * k)), Image.LANCZOS)


def crop_fr(img: Image.Image, box) -> Image.Image:
    if not box:
        return img
    w, h = img.size
    return img.crop((round(box[0] * w), round(box[1] * h), round(box[2] * w), round(box[3] * h)))


def trim_margin(img: Image.Image, thr: float = 45.0, pad: int = 8) -> Image.Image:
    """Crop away a plain mount or margin: keep the largest region that differs from the colour of the corners."""
    a = f32(img)
    ref = np.median(np.concatenate([a[:12, :12].reshape(-1, 3), a[:12, -12:].reshape(-1, 3), a[-12:, :12].reshape(-1, 3), a[-12:, -12:].reshape(-1, 3)]), axis=0)
    diff = (np.linalg.norm(a - ref, axis=2) > thr).astype(np.uint8)
    diff = cv2.morphologyEx(diff, cv2.MORPH_CLOSE, np.ones((15, 15), np.uint8))
    diff = cv2.morphologyEx(diff, cv2.MORPH_OPEN, np.ones((9, 9), np.uint8))
    n, lcc, stats, _ = cv2.connectedComponentsWithStats(diff, 8)
    k = 1 + int(np.argmax(stats[1:, cv2.CC_STAT_AREA]))
    x, y, w, h = stats[k, :4]
    return img.crop((x + pad, y + pad, x + w - pad, y + h - pad))


def build_plate(spec: dict) -> tuple[int, int]:
    """One plate from plates.json: crop, optional cut-out, smoothing, grade, grain, and the paper shape.

    shape  torn   a deckled rectangle (paintings and prints)
           arch   a round-topped window with a torn foot
           band   a wide torn strip
           cut    the object alone, cut from its studio backdrop
    """
    key = spec["key"]
    shape = spec.get("shape", "torn")
    seed = spec.get("seed", seed_of(key))
    gr = dict(spec.get("grade", {}))
    if "tint" in gr:
        gr["tint"] = tuple(gr["tint"])
    kw = spec.get("kuwahara", {})
    gn = spec.get("grain", {})
    img = crop_fr(load(spec["work"]), spec.get("crop"))
    if spec.get("trim"):
        img = trim_margin(img)
    width = spec["width"]

    def smooth(a: np.ndarray) -> np.ndarray:
        if not kw:
            return a
        return kuwahara(a, kw.get("r", 1)) * kw.get("mix", 0.5) + a * (1 - kw.get("mix", 0.5))

    if shape == "cut":
        cut = spec.get("cut", {})
        a = f32(limit(img, cut.get("work_max", 1800)))
        h, w = a.shape[:2]
        fg = studio_cut(a, cut.get("sure", 38.0), cut.get("maybe", 15.0), cut.get("floor", 7.0))
        if "largest" in cut:
            fg = largest(fg, cut["largest"])
        if "trim_bottom" in cut:
            fg[int(cut["trim_bottom"] * h):] = 0
        if "trim_top" in cut:
            fg[: int(cut["trim_top"] * h)] = 0
        if "drop_dark" in cut:  # slivers of a black backdrop left along a ragged edge
            k = cut.get("drop_band", 31)
            band = fg - cv2.erode(fg, np.ones((k, k), np.uint8))
            fg[(band == 1) & (a.mean(axis=2) < cut["drop_dark"])] = 0
            fg = cv2.morphologyEx(fg, cv2.MORPH_OPEN, np.ones((5, 5), np.uint8))
        if "kill_black" in cut:  # near-black pixels along the side edges are the backdrop, not the object
            thr, edge = cut["kill_black"]
            dark = a.mean(axis=2) < thr
            dark[:, int(edge * w):int((1 - edge) * w)] = False
            fg[dark] = 0
            fg = cv2.morphologyEx(fg, cv2.MORPH_OPEN, np.ones((5, 5), np.uint8))
        for x0, y0, x1, y1 in cut.get("erase", []):  # hand-placed boxes (fractions) to remove
            fg[int(y0 * h):int(y1 * h), int(x0 * w):int(x1 * w)] = 0
        if cut.get("largest_after_erase"):
            fg = largest(fg, 0)
        alpha = soft_alpha(fg, *cut.get("soft", [2.0, 0.3, 0.8, 2]))
        a = grade(a, alpha, **gr)
        a = smooth(a)
        a = grain(a, seed, gn.get("strength", 2.2), gn.get("size", 1.0))
        a = edge_to_paper(a, alpha, cut.get("rim", 0.0))
        out = fit(crop_to_alpha(rgba(a, alpha), cut.get("pad", 14)), width)
    else:
        a = f32(fit(img, width))
        if spec.get("tone"):
            a = tone(a, **{k: tuple(v) if isinstance(v, list) else v for k, v in spec["tone"].items()})
        if not kw.get("after_grade"):
            a = smooth(a)
        a = grade(a, None, **gr)
        if kw.get("after_grade"):
            a = smooth(a)
        a = grain(a, seed, gn.get("strength", 2.6), gn.get("size", 1.0))
        base = to_img(a)
        tr = spec.get("torn", {})
        if shape == "arch":
            al = arch_alpha(base.width, base.height, seed, tr.get("amp", 6.0), tr.get("inset", 14.0))
            aa = edge_to_paper(a, al, tr.get("rim", 0.6))
            out = Image.fromarray(np.dstack([np.clip(aa, 0, 255), al * 255]).astype(np.uint8), "RGBA")
        else:
            out = torn(base, seed + 1, amp=tr.get("amp", 9.0), inset=tr.get("inset", 12.0), rim=tr.get("rim", 0.6))
    save_webp(out, f"plate-{key}.webp", quality=spec.get("quality", 78))
    return out.size


def expand_series(specs: list[dict]) -> list[dict]:
    """A spec with "series": N becomes N plates, windows along a long work (a frieze) taken across its rows."""
    out: list[dict] = []
    for sp in specs:
        n = sp.get("series")
        if not n:
            out.append(sp)
            continue
        rows = sp["rows"]
        x0, x1 = sp["xrange"]
        win = sp["window"]
        base = {k: v for k, v in sp.items() if k not in ("series", "window", "rows", "xrange")}
        for i in range(n):
            r = i % len(rows)
            idx = i // len(rows)
            count = len(range(r, n, len(rows)))
            left = x0 + (x1 - x0 - win) * (idx / max(count - 1, 1))
            out.append({**base, "key": f"{sp['key']}-{i + 1:02d}", "crop": [left, rows[r][0], left + win, rows[r][1]]})
    return out


def load_specs() -> tuple[list[dict], dict]:
    doc = json.loads(PLATES_FILE.read_text(encoding="utf-8"))
    return expand_series(doc["plates"]), doc["pages"]


SOURCES_MD = ROOT / "art" / "SOURCES.md"

SOURCES_HEAD = """# Sources for the painted artwork

Everything under `apps/www/public/art/` is made by `art/build.py` from the public-domain and CC0 works below. Raw
downloads stay in `art/.cache` (ignored by git); `art/sources.lock.json` holds the SHA-256 of each source file the build
used, so a later run can tell if a museum has replaced an image. No work here was drawn, generated or redrawn: each is
cropped, cut, graded and set on paper, nothing more.

This file is generated by `python art/build.py` from `art/works.json` (the works) and `art/plates.json` (what is made from
them and where it is shown). Edit those, not this. The same list, as a table with links, is on `/credits`, made from the
same data, and the label under each plate on the site is made from it too.

Run `python art/build.py` (Python 3.10+, Pillow, NumPy, opencv-python, curl). `python art/build.py astrolabe luke`
builds only those plates (a plate key, a work key, or a prefix such as `frieze`); `ASTEN_ART_CACHE=/some/dir` moves the
download cache.

Licences. Met, Cleveland, Yale, National Gallery of Art and British Library images are released by the holder under
Creative Commons Zero; the Met API field `isPublicDomain` is checked before any Met download. The others are in the public
domain by age, and Wikimedia Commons marks each as public domain; where a library or museum made the scan, it is named in
the table and claims no right over a faithful reproduction.

## Works

"""

SOURCES_FONTS = """
## Fonts

| Font | Copyright | Licence |
| --- | --- | --- |
| Caveat (the margin hand) | (c) 2014 The Caveat Project Authors, Impallari Type for Google Fonts | SIL Open Font License 1.1 |

The other fonts (Fraunces, Literata, IBM Plex Mono, Noto Sans Egyptian Hieroglyphs) are listed on `/credits`.

## What the build does

One grade for the whole set (`grade()` in `build.py`): the highlights are gained toward a warm paper light, saturation is
eased to 0.85 to 0.95, the blacks are lifted a little (a matte), and the cast is warmed. The goal is one lighting
temperature across a Hudson River sky, a New Kingdom papyrus, a Roman lamp, an 1809 engraving and a faience water clock.

Shapes (`shape` in `plates.json`):

- `torn`: a deckled rectangle. Each side wanders by fractal noise (`torn_alpha`) and the last pixels of the edge are blended
  toward bare paper. Seeds are fixed, so the same tear comes out every build.
- `arch`: a round-topped window with the same torn foot (`arch_alpha`).
- `band`: a wide torn strip, used as a banner or a frieze detail.
- `cut`: the object alone. The studio backdrop is modelled per row from the left and right margins, GrabCut settles the
  uncertain band (`studio_cut`), the largest component is kept, the matte is feathered and shrunk 2 px, and a quadrant-mean
  (Kuwahara) smoothing is blended in so the photograph sits with the paintings.
- Frieze series (`frieze-01` to `frieze-13`) are windows along the Tomb of Menna facsimile, two rows of seven and six.

Engravings and prints (`tone` in `plates.json`) are re-toned: ink to a warm brown, paper to the site's cream.

Four assets are made by hand-written builders because they need hand-placed cuts:

| Output | From | Treatment |
| --- | --- | --- |
| `hero-window.webp` | `kensett`, `sargent` | A portrait slice of the sky down past its gold band; Sargent's cliffs with their own sky removed (flood-filled from the top plus one hand-drawn polygon over the hazy valley, edge feathered 2.6 px); a horizon glow and a low vignette; Kuwahara radius 2 blended 55%; grade; fine grain. |
| `thoth.webp` | `hunefer` | Thoth is cut along his own ink outline: pixels close to the papyrus ground, not enclosed by a dark outline and connected to the border, are background; the gap between the legs and the neighbouring figures are removed with hand-placed polygons; the figure is enlarged 2.2x, smoothed and re-sharpened on the ink line; feathered 1.5 px and shrunk 1 px so no ground colour rims the edge. |
| `plate-ibis.webp` | `ibis` | The black ibis is removed by painting over the largest dark connected component with the plate's own margin colour; the engraving is re-toned to the site's paper; torn edge. |
| `grain.webp` | none (procedural) | A 128 px seamless paper-fibre tile: four octaves of Gaussian noise and seventy short fibres, dark specks and pale fibres as a low-alpha overlay. |
"""


def write_sources(specs: list[dict], pages: dict) -> None:
    """Regenerate art/SOURCES.md: the works table and the plates table come from the JSON, so they cannot drift."""
    used_on: dict[str, list[str]] = {}
    for path, pp in pages.items():
        used_on.setdefault(pp["plate"], []).append(path)
    lines = [SOURCES_HEAD, "| Key | Work | Maker and date | Holding institution and object | Licence | Source |", "| --- | --- | --- | --- | --- | --- |"]
    for key, w in WORKS.items():
        medium = f", {w['medium']}" if w.get("medium") else ""
        lines.append(f"| `{key}` | *{w['title']}*{medium} | {w['by']}, {w['date']} | {w['where']} | {w['licence']} | {w['href']} |")
    lines.append(SOURCES_FONTS)
    lines += ["", "## Plates", "", "| File | Work | Shape | Crop | Width | Shown on |", "| --- | --- | --- | --- | --- | --- |"]
    for sp in specs:
        f = OUT / f"plate-{sp['key']}.webp"
        kb = f"{f.stat().st_size / 1024:.0f} KB" if f.exists() else "not built"
        crop = "none" if not sp.get("crop") else ", ".join(f"{v:.2f}" for v in sp["crop"])
        shown = ", ".join(f"`{x}`" for x in used_on.get(sp["key"], [])) or "Home"
        lines.append(f"| `plate-{sp['key']}.webp` ({kb}) | `{sp['work']}` | {sp.get('shape', 'torn')} | {crop} | {sp.get('width', '')} | {shown} |")
    SOURCES_MD.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    print("art/SOURCES.md written")


CUSTOM = {
    "hero": (build_hero, "hero-window.webp"),
    "thoth": (build_thoth, "thoth.webp"),
    "ibis": (build_ibis, "plate-ibis.webp"),
    "grain": (build_grain, "grain.webp"),
}


def write_manifest(specs: list[dict], pages: dict) -> None:
    """Everything the site needs to show the art: sizes from the built files, labels, credits, page map."""
    plates: dict[str, dict] = {}
    for sp in specs:
        f = OUT / f"plate-{sp['key']}.webp"
        if not f.exists():
            continue
        with Image.open(f) as im:
            w, h = im.size
        plates[sp["key"]] = {
            "src": f"/art/plate-{sp['key']}.webp",
            "width": w,
            "height": h,
            "bytes": f.stat().st_size,
            "shape": sp.get("shape", "torn"),
            "works": sp.get("works") or [sp["work"]],
            "label": sp["label"],
            "alt": sp["alt"],
        }
    doc = {"_note": "Generated by art/build.py from works.json and plates.json. Do not edit.", "works": {k: {kk: vv for kk, vv in v.items() if kk != "source"} for k, v in WORKS.items()}, "plates": plates, "pages": pages}
    MANIFEST.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"manifest: {len(plates)} plates, {len(WORKS)} works")


if __name__ == "__main__":
    specs, pages = load_specs()
    by_key = {sp["key"]: sp for sp in specs}
    names = sys.argv[1:]
    todo_custom = [n for n in CUSTOM if not names or n in names]
    def wanted(sp: dict) -> bool:
        return not names or any(sp["key"] == n or sp["key"].startswith(n + "-") or sp["work"] == n for n in names)

    todo_plates = [sp for sp in specs if not sp.get("custom") and wanted(sp)]
    unknown = [n for n in names if n not in CUSTOM and n not in WORKS and not any(k == n or k.startswith(n + "-") for k in by_key)]
    if unknown:
        raise SystemExit(f"unknown {unknown}; choose from {', '.join([*CUSTOM, *by_key])}")
    for n in todo_custom:
        CUSTOM[n][0]()
    for sp in todo_plates:
        build_plate(sp)
    write_manifest(specs, pages)
    write_sources(specs, pages)
