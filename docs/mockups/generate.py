#!/usr/bin/env python3
"""
Generates the widget mockups used by the roadmap issues (docs/mockups/*.png).

The mockups imitate how Pushrod draws today: flat fills, 1 px black borders,
Open Sans text, inverted colors when pressed/selected, and a 50% dither for
disabled widgets. Each image is a storyboard: numbered frames showing one
interaction step each, with the event emitted (if any) underneath.

Usage:  python3 docs/mockups/generate.py           (writes PNGs next to this file)
Needs:  Pillow  (pip install pillow)

To change a mockup, edit the frame functions below and re-run; keep frames
simple so they stay faithful to what Pushrod can draw.

Sample data must be fictional: no real names, e-mail addresses, places,
paths, or other personal information. Use placeholders such as "Jane Doe",
"jane.doe@example.com", "Anytown", and "/home/user" (see CONTRIBUTING.md).
"""
import os
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
FONT_PATH = os.path.join(HERE, "..", "..", "assets", "OpenSans-Regular.ttf")
S = 2                       # render scale (2x for crisp images)
BG = (255, 255, 255)        # window background
PANEL = (236, 236, 236)     # frame background (neutral gray)
BLACK, WHITE = (0, 0, 0), (255, 255, 255)
GRAY, MID, DARK = (200, 200, 200), (150, 150, 150), (90, 90, 90)
NOTE = (40, 90, 160)        # event annotations

_fonts = {}
def font(size=14):
    if size not in _fonts:
        _fonts[size] = ImageFont.truetype(FONT_PATH, size * S)
    return _fonts[size]


class C:
    """A frame canvas in logical pixels."""
    def __init__(self, w, h, bg=PANEL):
        self.w, self.h = w, h
        self.img = Image.new("RGB", (w * S, h * S), bg)
        self.d = ImageDraw.Draw(self.img)

    # ---- primitives -----------------------------------------------------
    def rect(self, x, y, w, h, fill=None, outline=BLACK, width=1):
        self.d.rectangle([x * S, y * S, (x + w) * S - 1, (y + h) * S - 1], fill=fill,
                         outline=outline, width=width * S if outline else 0)

    def line(self, x1, y1, x2, y2, color=BLACK, width=1):
        self.d.line([x1 * S, y1 * S, x2 * S, y2 * S], fill=color, width=width * S)

    def poly(self, pts, fill=BLACK, outline=None):
        self.d.polygon([(x * S, y * S) for x, y in pts], fill=fill, outline=outline)

    def ellipse(self, x, y, w, h, fill=None, outline=BLACK, width=1):
        self.d.ellipse([x * S, y * S, (x + w) * S - 1, (y + h) * S - 1], fill=fill, outline=outline, width=width * S)

    def tw(self, s, size=14):
        return self.d.textlength(s, font=font(size)) / S

    def text(self, x, y, s, color=BLACK, size=14, underline_index=None):
        self.d.text((x * S, y * S), s, fill=color, font=font(size))
        if underline_index is not None:
            pre = self.tw(s[:underline_index], size)
            cw = self.tw(s[underline_index], size)
            uy = y + size + 3
            self.line(pre + x, uy, x + pre + cw, uy, color)

    def ctext(self, x, y, w, h, s, color=BLACK, size=14, underline_index=None):
        tx = x + (w - self.tw(s, size)) / 2
        ty = y + (h - size * 1.36) / 2
        self.text(tx, ty, s, color, size, underline_index)

    def dither(self, x, y, w, h, color=(255, 255, 255)):
        for yy in range(y, y + h):
            for xx in range(x + (yy % 2), x + w, 2):
                self.d.rectangle([xx * S, yy * S, xx * S + S - 1, yy * S + S - 1], fill=color)

    def dotted(self, x, y, w, h, color=BLACK):
        for xx in range(x, x + w, 2):
            self.rect(xx, y, 1, 1, fill=color, outline=None); self.rect(xx, y + h - 1, 1, 1, fill=color, outline=None)
        for yy in range(y, y + h, 2):
            self.rect(x, yy, 1, 1, fill=color, outline=None); self.rect(x + w - 1, yy, 1, 1, fill=color, outline=None)

    def pointer(self, x, y):
        pts = [(x, y), (x, y + 16), (x + 4, y + 12), (x + 7, y + 19), (x + 9, y + 18), (x + 6, y + 11), (x + 11, y + 11)]
        self.poly(pts, fill=WHITE, outline=BLACK)
        self.d.line([(px * S, py * S) for px, py in pts + [pts[0]]], fill=BLACK, width=S)

    def ibeam(self, x, y):
        self.line(x, y, x, y + 16); self.line(x - 3, y, x + 3, y); self.line(x - 3, y + 16, x + 3, y + 16)

    def tri(self, x, y, size, direction, color=BLACK):
        s = size
        if direction == "up": pts = [(x, y + s), (x + s, y + s), (x + s / 2, y)]
        elif direction == "down": pts = [(x, y), (x + s, y), (x + s / 2, y + s)]
        elif direction == "left": pts = [(x + s, y), (x + s, y + s), (x, y + s / 2)]
        else: pts = [(x, y), (x, y + s), (x + s, y + s / 2)]
        self.poly(pts, fill=color)

    # ---- widgets (all drawn the way Pushrod paints) ----------------------
    def button(self, x, y, w, h, label, pressed=False, default=False, disabled=False, focused=False, mn=None):
        if default:
            self.rect(x - 3, y - 3, w + 6, h + 6, fill=None, outline=BLACK, width=2)
        bg, fg = (BLACK, WHITE) if pressed else (WHITE, BLACK)
        self.rect(x, y, w, h, fill=bg)
        self.ctext(x, y, w, h, label, fg, underline_index=mn)
        if focused: self.dotted(x + 2, y + 2, w - 4, h - 4, WHITE if pressed else BLACK)
        if disabled: self.dither(x, y, w, h, PANEL)

    def glyph(self, x, y, s, direction, pressed=False, disabled=False):
        self.rect(x, y, s, s, fill=BLACK if pressed else WHITE)
        self.tri(x + s * 0.3, y + s * 0.3, s * 0.4, direction, WHITE if pressed else BLACK)
        if disabled: self.dither(x, y, s, s, PANEL)

    def checkbox(self, x, y, label, state="off", pressed=False, disabled=False, focused=False, mn=None):
        self.rect(x, y + 3, 14, 14, fill=BLACK if pressed else WHITE)
        c = WHITE if pressed else BLACK
        if state == "on":
            self.line(x + 3, y + 10, x + 6, y + 13, c, 2); self.line(x + 6, y + 13, x + 11, y + 6, c, 2)
        elif state == "mixed":
            self.line(x + 3, y + 10, x + 11, y + 10, c, 2)
        self.text(x + 21, y, label, underline_index=mn)
        if focused: self.dotted(x + 18, y, int(self.tw(label)) + 6, 21)
        if disabled: self.dither(x, y, int(self.tw(label)) + 24, 21, PANEL)

    def radio(self, x, y, label, on=False, pressed=False, disabled=False):
        self.ellipse(x, y + 3, 14, 14, fill=BLACK if pressed else WHITE)
        if on: self.ellipse(x + 4, y + 7, 6, 6, fill=WHITE if pressed else BLACK, outline=None)
        self.text(x + 21, y, label)
        if disabled: self.dither(x, y, int(self.tw(label)) + 24, 21, PANEL)

    def field(self, x, y, w, text="", caret=None, sel=None, placeholder="", disabled=False, focused=False, h=22):
        self.rect(x, y, w, h, fill=WHITE)
        tx = x + 4
        if sel:
            a, b = sel
            sx = tx + self.tw(text[:a]); sw = self.tw(text[a:b])
            self.rect(int(sx), y + 3, int(sw) + 1, h - 6, fill=BLACK, outline=None)
            self.text(tx, y + 1, text[:a]); self.text(sx, y + 1, text[a:b], WHITE); self.text(sx + sw, y + 1, text[b:])
        elif text:
            self.text(tx, y + 1, text)
        elif placeholder:
            self.text(tx, y + 1, placeholder, MID)
        if caret is not None:
            cx = tx + self.tw(text[:caret]); self.line(cx, y + 4, cx, y + h - 4)
        if focused: self.rect(x - 2, y - 2, w + 4, h + 4, outline=BLACK, width=1)
        if disabled: self.dither(x, y, w, h, PANEL)

    def labeled(self, x, y, label, lw=60):
        self.text(x, y + 1, label)
        return x + lw

    def scrollbar(self, x, y, length, vertical=True, pos=0.0, size=0.3, pressed=None, thick=16):
        if vertical:
            self.rect(x, y, thick, length, fill=WHITE)
            self.glyph(x, y, thick, "up", pressed == "up"); self.glyph(x, y + length - thick, thick, "down", pressed == "down")
            track = length - 2 * thick; th = max(10, int(track * size)); ty = y + thick + int((track - th) * pos)
            self.rect(x + 2, ty, thick - 4, th, fill=GRAY if pressed != "thumb" else DARK)
        else:
            self.rect(x, y, length, thick, fill=WHITE)
            self.glyph(x, y, thick, "left", pressed == "left"); self.glyph(x + length - thick, y, thick, "right", pressed == "right")
            track = length - 2 * thick; tw = max(10, int(track * size)); tx = x + thick + int((track - tw) * pos)
            self.rect(tx, y + 2, tw, thick - 4, fill=GRAY)

    def slider(self, x, y, w, frac, ticks=0, disabled=False, value=None, focused=False):
        self.line(x, y + 10, x + w, y + 10, BLACK, 2)
        if ticks:
            for t in range(ticks + 1):
                tx = x + w * t / ticks; self.line(tx, y + 15, tx, y + 19)
        kx = x + int(w * frac) - 6
        self.rect(kx, y + 1, 12, 18, fill=WHITE)
        if value is not None: self.text(x + w + 10, y, str(value))
        if focused: self.dotted(x - 4, y - 3, w + 8, 26)
        if disabled: self.dither(x - 6, y, w + 50, 22, PANEL)

    def progress(self, x, y, w, frac=None, busy_at=None, h=16):
        self.rect(x, y, w, h, fill=WHITE)
        if frac: self.rect(x + 2, y + 2, int((w - 4) * frac), h - 4, fill=BLACK, outline=None)
        if busy_at is not None: self.rect(x + 2 + int((w - 44) * busy_at), y + 2, 40, h - 4, fill=BLACK, outline=None)

    def listbox(self, x, y, w, rows, selected=(), row_h=20, scroll=True, focus_row=None):
        h = row_h * len(rows) + 2
        self.rect(x, y, w, h, fill=WHITE)
        inner = w - (16 if scroll else 0)
        for n, r in enumerate(rows):
            ry = y + 1 + n * row_h
            if n in selected:
                self.rect(x + 1, ry, inner - 2, row_h, fill=BLACK, outline=None); self.text(x + 6, ry, r, WHITE)
            else:
                self.text(x + 6, ry, r)
            if focus_row == n: self.dotted(x + 2, ry + 1, inner - 4, row_h - 2, WHITE if n in selected else BLACK)
        if scroll: self.scrollbar(x + w - 16, y, h, True, 0.0, 0.5)
        return h

    def menu(self, x, y, w, items, hover=None):
        """items: list of (label, shortcut, flags) flags in {'sep','disabled','sub','check'}"""
        h = sum(9 if "sep" in f else 22 for _, _, f in items) + 4
        self.rect(x, y, w, h, fill=WHITE)
        cy = y + 2
        for n, (label, sc, f) in enumerate(items):
            if "sep" in f:
                self.line(x + 4, cy + 4, x + w - 5, cy + 4, MID); cy += 9; continue
            hot = n == hover
            if hot: self.rect(x + 1, cy, w - 2, 22, fill=BLACK, outline=None)
            col = WHITE if hot else (MID if "disabled" in f else BLACK)
            if "check" in f: self.text(x + 6, cy + 1, "✓", col)
            self.text(x + 22, cy + 1, label, col)
            if sc: self.text(x + w - 8 - self.tw(sc), cy + 1, sc, col)
            if "sub" in f: self.tri(x + w - 14, cy + 7, 8, "right", col)
            cy += 22
        return h

    def group(self, x, y, w, h, title):
        self.rect(x, y + 9, w, h - 9, fill=None)
        tw = self.tw(title)
        self.rect(x + 8, y, int(tw) + 8, 19, fill=PANEL, outline=None)
        self.text(x + 12, y, title)

    def tooltip(self, x, y, s):
        w = int(self.tw(s, 12)) + 12
        self.rect(x, y, w, 20, fill=(255, 255, 225)); self.text(x + 6, y + 2, s, size=12)

    def window(self, x, y, w, h, title=None):
        self.rect(x, y, w, h, fill=PANEL)
        if title:
            self.rect(x, y, w, 24, fill=WHITE); self.ctext(x, y, w, 24, title)


# ---- storyboard composition ------------------------------------------------
def storyboard(frames, path, cols=None, max_w=1180):
    """frames: list of (label, canvas, note). Lays frames left-to-right, wrapping."""
    pad, label_h, note_h = 14, 24, 22
    has_note = any(f[2] for f in frames)
    probe = ImageDraw.Draw(Image.new("RGB", (1, 1)))
    def colw(f):
        lw = probe.textlength(f[0], font=font(13)) / S
        nw = probe.textlength("\u2192 " + f[2], font=font(12)) / S if f[2] else 0
        return int(max(f[1].w, lw, nw)) + 2
    rows, row, row_w = [], [], pad
    for f in frames:
        fw = colw(f) + pad
        if row and ((cols and len(row) >= cols) or (not cols and row_w + fw > max_w)):
            rows.append(row); row, row_w = [], pad
        row.append(f); row_w += fw
    if row: rows.append(row)
    W = max(sum(colw(f) + pad for f in r) + pad for r in rows)
    H = pad + sum(label_h + max(f[1].h for f in r) + (note_h if has_note else 0) + pad for r in rows)
    out = Image.new("RGB", (W * S, H * S), BG)
    d = ImageDraw.Draw(out)
    y = pad
    for r in rows:
        x = pad; rh = max(f[1].h for f in r)
        for label, cv, note in r:
            d.text((x * S, y * S), label, fill=DARK, font=font(13))
            out.paste(cv.img, (x * S, (y + label_h) * S))
            d.rectangle([x * S - 1, (y + label_h) * S - 1, (x + cv.w) * S, (y + label_h + cv.h) * S], outline=GRAY, width=S)
            if note:
                d.text((x * S, (y + label_h + cv.h + 4) * S), "→ " + note, fill=NOTE, font=font(12))
            x += colw((label, cv, note)) + pad
        y += label_h + rh + (note_h if has_note else 0) + pad
    out.save(path, optimize=True)


# ============================================================================
# Storyboards, keyed by roadmap issue (e.g. "2.4") and step-set index.
# ============================================================================
SB = {}
def sb(key, n=1):
    def deco(fn):
        SB[f"{key}-{n}"] = fn
        return fn
    return deco


@sb("1.1")
def states():
    out = []
    for label, kw in [("Normal", {}), ("Pressed", dict(pressed=True)), ("Disabled", dict(disabled=True)),
                      ("Focused", dict(focused=True)), ("Hovered", {})]:
        c = C(120, 56); c.button(20, 16, 80, 24, "OK", **kw)
        if label == "Hovered": c.pointer(84, 28)
        out.append((label, c, None))
    c = C(150, 56); c.checkbox(14, 18, "Mixed", "mixed"); out.append(("Indeterminate", c, None))
    return out


@sb("1.12")
def mnemonics():
    c = C(330, 70); c.button(14, 14, 90, 26, "Save", mn=0); c.button(116, 14, 90, 26, "Cancel", mn=0)
    c2 = C(330, 70); c2.text(14, 22, "Name:", underline_index=0); c2.field(70, 20, 200)
    c3 = C(330, 70); c3.text(14, 22, "Name:", underline_index=0); c3.field(70, 20, 200, "", caret=0, focused=True)
    return [("Buttons with mnemonics", c, "Alt+S clicks Save, Alt+C clicks Cancel"),
            ("Label with mnemonic", c2, None), ("Alt+N pressed", c3, "FocusGained on the field")]


@sb("2.1")
def boxed_label():
    out = []
    for lab, dis in [("Normal", False), ("Disabled", True)]:
        c = C(230, 60); c.rect(20, 16, 190, 28, fill=WHITE); c.ctext(20, 16, 190, 28, "Boxed caption")
        if dis: c.dither(20, 16, 190, 28, PANEL)
        out.append((lab, c, None))
    return out


@sb("2.2")
def glyph_buttons():
    out = []
    for lab, kw, note in [("1. Idle", {}, None), ("2. Pressed", dict(pressed=True), "Clicked (fires on press)"),
                          ("3. Held", dict(pressed=True), "Clicked repeats every 100 ms"), ("4. Released", {}, None),
                          ("5. Disabled", dict(disabled=True), None)]:
        c = C(90, 60); c.glyph(33, 18, 24, "up", **kw)
        if lab != "5. Disabled": c.pointer(52, 36)
        out.append((lab, c, note))
    return out


@sb("2.3", 1)
def push_button():
    out = []
    for lab, pressed, px, note in [("1. Idle", False, 104, None), ("2. Pressed", True, 104, None),
                                   ("3. Dragged out", False, 150, "released early: no click"), ("4. Released inside", False, 104, "Clicked")]:
        c = C(170, 60); c.button(20, 18, 100, 26, "Cancel", pressed=pressed); c.pointer(px, 30); out.append((lab, c, note))
    return out

@sb("2.3", 2)
def default_button():
    c = C(250, 64); c.button(22, 20, 96, 26, "OK", default=True); c.button(134, 20, 96, 26, "Cancel")
    return [("Default button (Return) and cancel button (Escape)", c, None)]

@sb("2.3", 3)
def toggle_button():
    out = []
    for lab, p, note in [("1. Off", False, None), ("2. Clicked: stays pressed", True, "Toggled(true)"), ("3. Clicked again", False, "Toggled(false)")]:
        c = C(170, 60); c.button(35, 18, 100, 26, "Bold", pressed=p); out.append((lab, c, note))
    return out


@sb("2.4")
def checkbox():
    frames = [("1. Unchecked", dict(state="off"), None), ("2. Pressed", dict(state="off", pressed=True), None),
              ("3. Checked", dict(state="on"), "Toggled(true)"), ("4. Indeterminate", dict(state="mixed"), None),
              ("5. Focused", dict(state="on", focused=True), None), ("6. Disabled", dict(state="on", disabled=True), None)]
    out = []
    for lab, kw, note in frames:
        c = C(160, 50); c.checkbox(14, 14, "Send email", **kw); out.append((lab, c, note))
    return out


@sb("2.5")
def radios():
    out = []
    for lab, on, pressed, note in [("1. Medium checked", 1, None, None), ("2. Pressing Large", 1, 2, None),
                                   ("3. Released", 2, None, "Toggled: Large on, Medium off")]:
        c = C(170, 96)
        for n, t in enumerate(["Small", "Medium", "Large"]):
            c.radio(16, 12 + n * 26, t, on=(n == on), pressed=(n == pressed))
        out.append((lab, c, note))
    return out


@sb("2.6", 1)
def text_field():
    steps = [("1. Empty", dict(text="", placeholder="Your name"), None),
             ("2. Focused", dict(text="", caret=0, focused=True), None),
             ("3. Typing", dict(text="Jane", caret=4, focused=True), "TextChanged"),
             ("4. Selection", dict(text="Jane", sel=(1, 4), focused=True), None),
             ("5. Invalid key rejected", dict(text="Jane", caret=4, focused=True), "no change"),
             ("6. Disabled", dict(text="Jane", disabled=True), None)]
    out = []
    for lab, kw, note in steps:
        c = C(250, 56); x = c.labeled(12, 17, "Name:", 54); c.field(x, 17, 170, **kw); out.append((lab, c, note))
    return out

@sb("2.6", 2)
def masked_field():
    a = C(250, 56); x = a.labeled(12, 17, "Date:", 48); a.field(x, 17, 120, "__/__/____", caret=0, focused=True)
    b = C(250, 56); x = b.labeled(12, 17, "Date:", 48); b.field(x, 17, 120, "11/24/____", caret=5, focused=True)
    return [("Mask 99/99/9999", a, None), ("After typing 1 1 2 4", b, "TextChanged")]


def _picture(c, x, y, w, h):
    c.rect(x, y, w, h, fill=(120, 160, 200), outline=None)
    c.poly([(x, y + h), (x + w * 0.4, y + h * 0.45), (x + w * 0.7, y + h), ], fill=(70, 120, 70))
    c.ellipse(x + w * 0.65, y + h * 0.15, w * 0.18, w * 0.18, fill=(240, 210, 90), outline=None)

@sb("2.7")
def images():
    out = []
    for lab in ["Center", "Stretch", "Fit (keep aspect)", "Tile", "Missing file"]:
        c = C(150, 110); c.rect(15, 15, 120, 80, fill=WHITE)
        if lab == "Center": _picture(c, 50, 35, 50, 40)
        elif lab == "Stretch": _picture(c, 16, 16, 118, 78)
        elif lab.startswith("Fit"): _picture(c, 25, 16, 100, 78)
        elif lab == "Tile":
            for ty in range(16, 94, 26):
                for tx in range(16, 134, 30): _picture(c, tx, ty, min(29, 134 - tx), min(25, 94 - ty))
        else:
            c.line(15, 15, 134, 94); c.line(134, 15, 15, 94)
        out.append((lab, c, None))
    return out


def _icon(c, x, y, caption, selected):
    c.rect(x + 14, y, 40, 30, fill=WHITE); c.rect(x + 20, y + 6, 28, 12, fill=GRAY)
    tw = c.tw(caption)
    if selected: c.rect(int(x + 34 - tw / 2 - 3), y + 34, int(tw) + 6, 20, fill=BLACK, outline=None)
    c.text(x + 34 - tw / 2, y + 34, caption, WHITE if selected else BLACK)

@sb("2.8")
def icons():
    out = []
    for lab, sel, note in [("1. Idle", False, None), ("2. Selected", True, "Clicked"), ("3. Double-clicked", True, "Activated")]:
        c = C(120, 80); _icon(c, 26, 10, "Disk A", sel); out.append((lab, c, note))
    return out


@sb("2.9", 1)
def alert():
    c = C(440, 170); c.rect(0, 0, 440, 170, fill=GRAY, outline=None); c.dither(0, 0, 440, 170, MID)
    c.window(30, 14, 380, 140)
    c.poly([(60, 64), (84, 64), (72, 42)], fill=(250, 200, 0), outline=BLACK); c.text(69, 46, "!", BLACK, 13)
    c.text(100, 36, "Delete \"report.txt\"?"); c.text(100, 58, "This cannot be undone.", DARK, 13)
    c.button(200, 108, 90, 26, "Delete", default=True); c.button(304, 108, 90, 26, "Cancel")
    return [("Alert open (the rest of the window is drawn but inert)", c, "Return = Delete (default), Escape = Cancel")]


@sb("2.10")
def custom_paint():
    c = C(240, 110); c.rect(15, 15, 210, 80, fill=WHITE)
    for n, v in enumerate([30, 55, 20, 65, 45, 70, 35]):
        c.rect(28 + n * 27, 90 - v, 18, v, fill=BLACK, outline=None)
    return [("Drawn entirely by the application's closure", c, None)]


def _menubar(c, x, y, w, open_title=None):
    c.rect(x, y, w, 24, fill=WHITE)
    tx = x + 8
    for t in ["File", "Edit", "View"]:
        tw = c.tw(t)
        if t == open_title: c.rect(int(tx - 5), y + 1, int(tw) + 10, 22, fill=BLACK, outline=None)
        c.text(tx, y + 2, t, WHITE if t == open_title else BLACK); tx += tw + 20

MENU_ITEMS = [("New", "Ctrl+N", ""), ("Open...", "Ctrl+O", ""), ("", "", "sep"), ("Save", "Ctrl+S", ""),
              ("Revert", "", "disabled"), ("Recent", "", "sub"), ("", "", "sep"), ("Quit", "Ctrl+Q", "")]

@sb("2.11")
def menus():
    a = C(300, 70); _menubar(a, 10, 14, 280)
    b = C(300, 240); _menubar(b, 10, 14, 280, "File"); b.menu(13, 38, 170, MENU_ITEMS)
    d = C(300, 240); _menubar(d, 10, 14, 280, "File"); d.menu(13, 38, 170, MENU_ITEMS, hover=3); d.pointer(90, 112)
    return [("1. Menu bar", a, None), ("2. File opened", b, None), ("3. Hovering Save", d, "release: Triggered(save)")]


@sb("2.12")
def file_chooser():
    c = C(470, 330); c.window(10, 10, 450, 310, "Open")
    x = c.labeled(26, 46, "Folder:", 62); c.field(x, 44, 360, "/home/user/projects")
    x = c.labeled(26, 76, "Filter:", 62); c.field(x, 74, 120, "*.rs")
    c.listbox(26, 108, 418, ["..", "src/", "main.rs", "lib.rs", "build.rs", "README.md"], selected={2})
    x = c.labeled(26, 240, "Name:", 62); c.field(x, 238, 360, "main.rs")
    c.button(250, 280, 90, 26, "Open", default=True); c.button(354, 280, 90, 26, "Cancel")
    return [("Pushrod-drawn file chooser (never the OS dialog)", c, None)]


@sb("2.13")
def tooltips():
    a = C(220, 90); a.button(20, 20, 70, 26, "Save"); a.pointer(78, 34)
    b = C(220, 90); b.button(20, 20, 70, 26, "Save"); b.pointer(78, 34); b.tooltip(60, 58, "Save the current file")
    d = C(220, 90); d.button(20, 20, 70, 26, "Save"); d.pointer(140, 40)
    return [("1. Pointer rests", a, None), ("2. After 600 ms", b, None), ("3. Pointer moves", d, "tooltip hidden")]


@sb("2.14")
def separators():
    c = C(300, 110)
    x = c.labeled(14, 14, "Name:", 70); c.field(x, 12, 140)
    c.line(14, 50, 230, 50, MID); c.line(14, 51, 230, 51, WHITE)
    x = c.labeled(14, 66, "Address:", 70); c.field(x, 64, 140)
    c.line(260, 12, 260, 90, MID); c.line(261, 12, 261, 90, WHITE)
    return [("Horizontal and vertical separators", c, None)]


def _toolbar(c, x, y):
    c.rect(x, y, 420, 36, fill=WHITE)
    bx = x + 6
    for lab, kw in [("New", {}), ("Open", {}), ("Save", {}), ("|", {}), ("B", dict(pressed=True)), ("I", {}), ("|", {}), ("Paste", dict(disabled=True))]:
        if lab == "|": c.line(bx + 2, y + 6, bx + 2, y + 30, MID); bx += 8; continue
        w = int(c.tw(lab)) + 18
        c.button(bx, y + 5, w, 26, lab, **kw); bx += w + 6

@sb("2.15")
def toolbar():
    c = C(450, 60); _toolbar(c, 14, 12)
    return [("Toolbar: Bold toggled on, Paste disabled", c, "Triggered(action) on click")]


@sb("2.16")
def statusbar():
    c = C(480, 50); c.rect(10, 14, 460, 24, fill=WHITE); c.text(18, 16, "Saved report.txt")
    c.line(360, 14, 360, 37); c.text(372, 16, "Ln 12, Col 4")
    return [("Status bar: message area and a permanent field", c, None)]


def _switch(c, x, y, on, disabled=False):
    c.rect(x, y, 50, 22, fill=BLACK if on else WHITE)
    kx = x + 28 if on else x + 2
    c.rect(kx, y + 2, 20, 18, fill=WHITE)
    if disabled: c.dither(x, y, 50, 22, PANEL)

@sb("2.17")
def switch():
    out = []
    for lab, on, dis, note in [("Off", False, False, None), ("On", True, False, "Toggled(true)"), ("Disabled", True, True, None)]:
        c = C(150, 50); c.text(12, 14, "Wi-Fi"); _switch(c, 70, 14, on, dis); out.append((lab, c, note))
    return out


@sb("2.18")
def text_area():
    c = C(360, 120); c.rect(14, 12, 320, 96, fill=WHITE); c.scrollbar(334, 12, 96, True, 0.0, 0.4)
    c.text(20, 16, "Dear Customer,"); c.text(20, 36, "Thanks for the ")
    sx = 20 + c.tw("Thanks for the "); sel = "quick update"
    c.rect(int(sx), 39, int(c.tw(sel)) + 1, 17, fill=BLACK, outline=None); c.text(sx, 36, sel, WHITE)
    c.text(20, 56, "on the release."); c.line(20 + c.tw("on the release."), 59, 20 + c.tw("on the release."), 74)
    return [("Editing with a selection", c, "TextChanged on each edit")]


@sb("2.19")
def context_menu():
    c = C(300, 190)
    c.listbox(14, 12, 170, ["Apples", "Bananas", "Cherries", "Dates"], selected={2}, scroll=False)
    c.menu(110, 66, 130, [("Cut", "", ""), ("Copy", "", ""), ("", "", "sep"), ("Delete", "", "")], hover=1)
    c.pointer(104, 58)
    return [("Right-click on a list row", c, "ContextMenuRequested; then Triggered(copy)")]


@sb("2.20")
def message_bar():
    c = C(560, 60); c.rect(10, 14, 540, 32, fill=(255, 245, 200))
    c.ellipse(20, 21, 18, 18, fill=WHITE); c.ctext(20, 21, 18, 18, "i", size=12)
    c.text(48, 19, "Connection lost. Retrying in 5 s.")
    c.button(390, 18, 100, 24, "Retry now"); c.button(500, 18, 40, 24, "x")
    return [("Message bar with an action", c, "Clicked on Retry now or the close box")]


@sb("3.1", 1)
def scrollbar_v():
    out = []
    for lab, pos, pressed, note in [("1. Idle", 0.0, None, None), ("2. Track clicked below", 0.45, None, "ValueChanged (page down)"),
                                    ("3. Thumb dragged", 0.7, "thumb", "ValueChanged"), ("4. Down arrow held", 0.85, "down", "ValueChanged (repeating)")]:
        c = C(110, 190); c.scrollbar(46, 12, 166, True, pos, 0.3, pressed); out.append((lab, c, note))
    return out

@sb("3.1", 2)
def scrollbar_h():
    c = C(320, 44); c.scrollbar(14, 14, 290, False, 0.1, 0.3)
    return [("Horizontal", c, None)]


@sb("3.2")
def sliders():
    out = []
    for lab, f, v, t, dis, foc, note in [("1. Idle", 0.4, 40, 0, False, False, None), ("2. Dragging", 0.75, 75, 0, False, False, "ValueChanged (continuous)"),
                                         ("3. Ticks with snap", 0.5, 50, 6, False, False, None), ("4. Focused", 0.5, 50, 0, False, True, "arrow keys adjust"),
                                         ("5. Disabled", 0.4, 40, 0, True, False, None)]:
        c = C(270, 50); c.text(12, 14, "Volume"); c.slider(80, 14, 140, f, t, dis, v, foc); out.append((lab, c, note))
    return out


@sb("3.3")
def group_box():
    c = C(220, 130); c.group(12, 10, 190, 110, "Size")
    for n, t in enumerate(["Small", "Medium", "Large"]): c.radio(28, 36 + n * 26, t, on=(n == 1))
    return [("Group box", c, None)]


def _choice(c, x, y, w, text, cycle=False):
    c.rect(x, y, w, 24, fill=WHITE); c.rect(x + 2, y + 2, w - 4, 20, outline=GRAY)
    if cycle:
        c.ellipse(x + 8, y + 6, 12, 12); c.text(x + 28, y + 2, text)
    else:
        c.text(x + 8, y + 2, text); c.tri(x + w - 18, y + 9, 9, "down")

@sb("3.6", 1)
def choice_popup():
    a = C(230, 130); a.text(12, 16, "Size:"); _choice(a, 60, 14, 140, "Medium")
    b = C(230, 130); b.text(12, 16, "Size:"); b.menu(60, -8 + 14, 140, [("Small", "", ""), ("Medium", "", ""), ("Large", "", "")], hover=1); b.pointer(120, 32)
    d = C(230, 130); d.text(12, 16, "Size:"); _choice(d, 60, 14, 140, "Large")
    return [("1. Closed", a, None), ("2. Open: current item under the pointer", b, None), ("3. Large chosen", d, "SelectionChanged")]

@sb("3.6", 2)
def choice_cycle():
    out = []
    for n, t in enumerate(["Left", "Center", "Right", "Left"]):
        c = C(190, 50); c.text(12, 14, "Align:"); _choice(c, 64, 12, 110, t, cycle=True)
        out.append((f"{n + 1}. " + ("Start" if n == 0 else "Click"), c, None if n == 0 else "SelectionChanged"))
    return out


@sb("3.10")
def expander():
    a = C(260, 110); a.tri(14, 16, 9, "right"); a.text(30, 10, "Advanced options")
    b = C(260, 110); b.tri(13, 17, 10, "down"); b.text(30, 10, "Advanced options")
    b.checkbox(30, 40, "Verbose logging", "on"); x = b.labeled(30, 72, "Retries:", 64); b.field(x, 70, 50, "3")
    return [("Collapsed", a, None), ("Expanded", b, "Toggled(true)")]


@sb("3.11")
def date_picker():
    c = C(280, 250); x = c.labeled(12, 14, "Due:", 40); c.field(x, 12, 120, "11/24/2026"); c.glyph(x + 122, 12, 22, "down", pressed=True)
    c.rect(52, 40, 214, 196, fill=WHITE); c.tri(62, 50, 9, "left"); c.ctext(52, 42, 214, 22, "November 2026"); c.tri(246, 50, 9, "right")
    for n, d in enumerate(["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]): c.text(60 + n * 29, 68, d, DARK, 12)
    day = 1
    for row in range(5):
        for col in range(7):
            if day > 30: break
            dx, dy = 60 + col * 29, 90 + row * 28
            if day == 24: c.rect(dx - 4, dy - 2, 26, 24, fill=BLACK, outline=None); c.text(dx, dy, str(day), WHITE, 13)
            else: c.text(dx, dy, str(day), BLACK, 13)
            day += 1
    return [("Calendar open, 24 highlighted", c, "ValueChanged when a day is chosen")]


@sb("4.1")
def progress():
    out = []
    for lab, kw in [("0%", dict(frac=0)), ("40%", dict(frac=0.4)), ("Busy (moving block)", dict(busy_at=0.2)), ("Busy, later", dict(busy_at=0.7))]:
        c = C(240, 46); c.progress(14, 15, 210, **kw); out.append((lab, c, None))
    return out


def _spin(c, x, y, value, up=False, disabled=False):
    c.field(x, y, 60, str(value).rjust(4)); c.glyph(x + 60, y - 1, 12 + 0, "up", up); c.glyph(x + 60, y + 11, 12, "down")
    if disabled: c.dither(x, y - 1, 74, 24, PANEL)

@sb("4.2")
def spin_box():
    out = []
    for lab, v, up, dis, note in [("1. Idle", 3, False, False, None), ("2. Up arrow held", 7, True, False, "ValueChanged (repeating)"),
                                  ("3. Typed 99 (max 50)", 99, False, False, None), ("4. After Return", 50, False, False, "clamped; ValueChanged"),
                                  ("5. Disabled", 3, False, True, None)]:
        c = C(180, 50); x = c.labeled(12, 14, "Copies:", 62); _spin(c, x, 13, v, up, dis); out.append((lab, c, note))
    return out


def _tabs(c, x, y, w, h, tabs, active):
    tx = x
    for n, t in enumerate(tabs):
        tw = int(c.tw(t)) + 20
        c.rect(tx, y + (0 if n == active else 3), tw, 24 - (0 if n == active else 3), fill=PANEL if n == active else GRAY)
        c.ctext(tx, y, tw, 24, t)
        if n == active: ax, aw = tx, tw
        tx += tw - 1
    c.rect(x, y + 23, w, h - 23, fill=PANEL)
    c.line(ax + 1, y + 23, ax + aw - 2, y + 23, PANEL)

@sb("4.4")
def tab_view():
    a = C(330, 140); _tabs(a, 12, 10, 300, 120, ["General", "Advanced", "Network"], 0)
    x = a.labeled(26, 48, "Name:", 56); a.field(x, 46, 180); a.checkbox(26, 80, "Start at login", "on")
    b = C(330, 140); _tabs(b, 12, 10, 300, 120, ["General", "Advanced", "Network"], 1)
    b.text(26, 50, "Cache:"); b.slider(90, 50, 140, 0.5, value="128 MB")
    return [("1. General", a, None), ("2. Advanced clicked", b, "SelectionChanged")]


FRUIT = ["Apples", "Bananas", "Cherries", "Dates", "Elderberries"]

@sb("4.5")
def list_view():
    out = []
    for lab, sel, note in [("1. Idle", set(), None), ("2. Click selects", {1}, "SelectionChanged"),
                           ("3. Shift-click extends", {1, 2, 3}, "SelectionChanged"), ("4. Double-click", {2}, "Activated")]:
        c = C(200, 130); c.listbox(14, 12, 170, FRUIT, selected=sel); out.append((lab, c, note))
    return out


@sb("4.6")
def combo_box():
    a = C(250, 110); x = a.labeled(12, 14, "Font:", 46); a.field(x, 12, 150, "Helvetica"); a.glyph(x + 150, 12, 22, "down")
    b = C(250, 110); x = b.labeled(12, 14, "Font:", 46); b.field(x, 12, 150, "Co", caret=2, focused=True); b.glyph(x + 150, 12, 22, "down", True)
    b.listbox(x, 36, 172, ["Courier", "Courier New"], selected={0}, scroll=False)
    d = C(250, 110); x = d.labeled(12, 14, "Font:", 46); d.field(x, 12, 150, "Courier"); d.glyph(x + 150, 12, 22, "down")
    return [("1. Idle", a, None), ("2. Typing filters", b, "TextChanged"), ("3. Chosen", d, "SelectionChanged")]


@sb("4.9")
def table_view():
    c = C(460, 120); x0, y0 = 14, 12
    cols = [("Name", 190), ("Size", 90), ("Modified", 130)]
    c.rect(x0, y0, 430, 24, fill=WHITE); cx = x0
    for n, (t, w) in enumerate(cols):
        c.text(cx + 6, y0 + 2, t)
        if n == 1: c.tri(cx + w - 16, y0 + 9, 8, "down")
        c.line(cx + w, y0, cx + w, y0 + 88); cx += w
    rows = [("report.txt", "12 KB", "11/24/2026"), ("notes.md", "4 KB", "11/20/2026"), ("plan.pdf", "2 KB", "11/02/2026")]
    c.rect(x0, y0 + 24, 430, 64, fill=WHITE)
    for r, vals in enumerate(rows):
        ry = y0 + 25 + r * 21
        if r == 0: c.rect(x0 + 1, ry, 410, 21, fill=BLACK, outline=None)
        cx = x0
        for (t, w), v in zip(cols, vals):
            c.text(cx + 6, ry, v, WHITE if r == 0 else BLACK); cx += w
    c.scrollbar(x0 + 414, y0, 88, True, 0, 0.6)
    return [("Sorted by Size (descending)", c, "SortChanged when a header is clicked")]


CONTENT_ITEMS = [("Name:", 20, 20), ("Email:", 20, 60), ("Phone:", 20, 100), ("Address:", 20, 140), ("City:", 20, 180), ("Notes:", 20, 220)]

def _content(c, ox, oy, clip, scale=1.0):
    """Draw the fixed-size content (360 x 260, its own 0,0) at ox, oy, clipped to clip=(x, y, w, h)."""
    cx, cy, cw, ch = clip
    layer = C(360, 260, bg=WHITE)
    for label, x, y in CONTENT_ITEMS:
        layer.text(x, y + 2, label); layer.field(x + 80, y, 220, "")
    layer.text(6, 242, "0,0 is the content's own origin", MID, 11)
    crop = layer.img.crop(((cx - ox) * S, (cy - oy) * S, (cx - ox + cw) * S, (cy - oy + ch) * S))
    c.img.paste(crop, (cx * S, cy * S))
    c.rect(cx, cy, cw, ch, fill=None)

def _container(c, x, y, w, h, off_x, off_y):
    """A container at x, y with a w x h frame, showing content from offset (off_x, off_y)."""
    vw, vh = w - 16, h - 16
    _content(c, x - off_x, y - off_y, (x, y, vw, vh))
    c.scrollbar(x + vw, y, vh, True, off_y / max(1, 260 - vh), vh / 260)
    c.scrollbar(x, y + vh, vw, False, off_x / max(1, 360 - vw), vw / 360)
    c.rect(x + vw, y + vh, 16, 16, fill=GRAY)

@sb("5.3", 1)
def container_pan():
    a = C(270, 170); _container(a, 15, 15, 240, 140, 0, 0)
    b = C(270, 170); _container(b, 15, 15, 240, 140, 0, 110)
    d = C(270, 170); _container(d, 15, 15, 240, 140, 120, 110)
    return [("1. Viewport at the content's 0,0", a, None), ("2. Scrolled down", b, "ViewportChanged { offset: (0, 110) }"),
            ("3. Scrolled right", d, "ViewportChanged { offset: (120, 110) }")]

@sb("5.3", 2)
def container_resize_move():
    a = C(400, 300); _container(a, 15, 15, 240, 140, 0, 0)
    b = C(400, 300); _container(b, 15, 15, 376, 276, 0, 0)
    d = C(400, 300); _container(d, 145, 140, 240, 140, 0, 0)
    return [("1. Container 240 x 140", a, None), ("2. Resized to 376 x 276: more content visible, same content size", b, "ViewportChanged"),
            ("3. Moved: content positions unchanged", d, None)]


@sb("9.8")
def drag_drop():
    def base(c, highlight=False, dropped=False):
        c.listbox(14, 12, 130, ["Apples", "Bananas", "Cherries"], selected=set(), scroll=False)
        c.rect(200, 12, 140, 90, fill=WHITE, outline=BLACK, width=2 if highlight else 1)
        c.text(208, 16, "Basket", DARK)
        if dropped: c.text(214, 40, "Bananas")
    a = C(360, 120); base(a); a.rect(15, 33, 128, 20, fill=BLACK, outline=None); a.text(20, 32, "Bananas", WHITE); a.pointer(70, 40)
    b = C(360, 120); base(b, highlight=True); b.rect(230, 52, 80, 20, fill=WHITE, outline=MID); b.text(236, 51, "Bananas", DARK); b.pointer(270, 60)
    d = C(360, 120); base(d, dropped=True)
    return [("1. Drag starts", a, None), ("2. Over an accepting target", b, "target highlighted"), ("3. Dropped", d, "Dropped { payload }")]



def _alert_custom(c):
    c.rect(0, 0, 480, 170, fill=GRAY, outline=None); c.dither(0, 0, 480, 170, MID)
    c.window(30, 14, 420, 140)
    c.poly([(60, 64), (84, 64), (72, 42)], fill=(250, 200, 0), outline=BLACK); c.text(69, 46, "!", BLACK, 13)
    c.text(100, 34, "Delete \"report.txt\"?", (190, 0, 0), 16); c.text(100, 60, "This cannot be undone.", DARK, 13)
    c.button(52, 108, 80, 26, "Help")
    c.button(246, 108, 90, 26, "Delete", default=True); c.button(350, 108, 90, 26, "Cancel")

@sb("2.9", 3)
def alert_custom():
    c = C(480, 170); _alert_custom(c)
    return [("message part: red, larger font;  alert.add_button(\"Help\")", c, "Clicked on the Help part -> on_click(help_id)")]

@sb("5.2")
def composite_parts():
    c = C(480, 170); _alert_custom(c)
    for (x, y, w, h, name) in [(96, 32, 200, 24, "message"), (50, 106, 84, 30, "button:2"), (56, 38, 32, 30, "icon")]:
        c.dotted(x - 2, y - 2, w + 4, h + 4, (200, 0, 0))
    c.text(300, 36, "<- part \"message\"", (200, 0, 0), 12)
    c.text(140, 112, "<- part \"button:2\"", (200, 0, 0), 12)
    return [("Parts are real widgets: fetched by name, restyled, added", c, "parts(), part(\"message\"), add_button(..)")]


TREE = {(): ["Applications", "Library", "System", "Users"],
        ("Users",): ["user", "guest", "shared"],
        ("Users", "user"): ["Desktop", "Documents", "Music", "Projects"],
        ("Users", "user", "Projects"): ["notes-app", "pushrod", "sample-game"],
        ("Users", "user", "Projects", "pushrod"): ["Cargo.toml", "README.md", "src", "examples"]}
LEAVES = {"Cargo.toml", "README.md"}

def _browser(c, x, y, path, ncols_visible=3, colw=140, rows=5, offset=0):
    cols = [()] + [tuple(path[:n + 1]) for n in range(len(path))]
    cols = [p for p in cols if p in TREE]
    shown = cols[offset:offset + ncols_visible]
    total_w = colw * ncols_visible
    c.rect(x, y, total_w + 2, rows * 20 + 4, fill=WHITE)
    for n, p in enumerate(shown):
        cx = x + 1 + n * colw
        items = TREE[p]
        sel = path[len(p)] if len(p) < len(path) else None
        for r, it in enumerate(items[:rows]):
            ry = y + 2 + r * 20
            hot = it == sel
            if hot: c.rect(cx, ry, colw - 2, 20, fill=BLACK, outline=None)
            col = WHITE if hot else BLACK
            c.text(cx + 6, ry, it, col, 13)
            branch = tuple(list(p) + [it]) in TREE or (it not in LEAVES and it not in ("Desktop", "Music", "guest", "shared", "Library", "System", "Applications", "notes-app", "sample-game", "src", "examples"))
            if branch: c.tri(cx + colw - 14, ry + 6, 7, "right", col)
        if n < ncols_visible - 1: c.line(cx + colw - 1, y, cx + colw - 1, y + rows * 20 + 4, MID)
    return total_w

@sb("3.7", 1)
def browser_steps():
    out = []
    for lab, path, note in [("1. Root column", [], None),
                            ("2. 'Users' selected: column 1 appears", ["Users"], "SelectionChanged (path /Users)"),
                            ("3. 'user' selected: column 2 appears", ["Users", "user"], "SelectionChanged (path /Users/user)")]:
        c = C(450, 130); _browser(c, 14, 14, path); out.append((lab, c, note))
    return out

@sb("3.7", 2)
def browser_scroll():
    c = C(450, 150); _browser(c, 14, 14, ["Users", "user", "Projects", "pushrod"], offset=2)
    c.scrollbar(14, 122, 422, False, 1.0, 0.6)
    c2 = C(450, 150); _browser(c2, 14, 14, ["Users", "user", "Projects", "pushrod", "Cargo.toml"], offset=2)
    c2.scrollbar(14, 122, 422, False, 1.0, 0.6)
    return [("Deep path: scrolled to keep the newest column visible", c, None),
            ("Leaf 'Cargo.toml' selected (double-click or Return activates)", c2, "Activated")]


BLUE, DBLUE = (70, 130, 200), (30, 70, 140)

@sb("1.17", 1)
def state_styles():
    out = []
    for lab, bg, fg, ptr, note in [("Normal", WHITE, BLACK, None, None), ("Hovered (style: blue)", BLUE, WHITE, (116, 36), "PointerEntered -> on_hover(true)"),
                                   ("Pressed (style: dark blue)", DBLUE, WHITE, (116, 36), "PointerPressed -> on_press"),
                                   ("Released inside", BLUE, WHITE, (116, 36), "Clicked -> on_click"),
                                   ("Pointer left", WHITE, BLACK, (150, 50), "PointerExited -> on_hover(false)")]:
        c = C(170, 70); c.rect(30, 20, 100, 28, fill=bg); c.ctext(30, 20, 100, 28, "Save", fg)
        if ptr: c.pointer(*ptr)
        out.append((lab, c, note))
    return out

@sb("1.17", 2)
def drag_over_style():
    a = C(300, 110); a.rect(150, 20, 130, 70, fill=WHITE); a.text(158, 24, "Drop here", DARK)
    a.rect(20, 40, 80, 22, fill=WHITE); a.text(26, 40, "Bananas"); a.pointer(70, 50)
    b = C(300, 110); b.rect(150, 20, 130, 70, fill=(230, 240, 255), outline=BLUE, width=2); b.text(158, 24, "Drop here", DBLUE)
    b.rect(180, 50, 80, 22, fill=WHITE, outline=MID); b.text(186, 50, "Bananas", DARK); b.pointer(220, 60)
    return [("Normal", a, None), ("Drag over (style: light blue, blue border)", b, "DRAG_OVER state while a drag hovers")]


def _panel(c, x, y, title="Panel"):
    c.rect(x, y, 150, 90, fill=WHITE); c.rect(x, y, 150, 22, fill=GRAY); c.text(x + 8, y + 1, title)
    c.checkbox(x + 10, y + 32, "Option A", "on"); c.checkbox(x + 10, y + 58, "Option B")

@sb("10.1", 1)
def live_mode():
    out = []
    for n, (x, note) in enumerate([(20, None), (120, "repaint"), (220, "repaint")], 1):
        c = C(400, 130); _panel(c, x, 20); c.pointer(x + 70, 30); out.append((f"{n}. " + ("Drag starts" if n == 1 else "Dragging"), c, note))
    return out

@sb("10.1", 2)
def outline_mode():
    a = C(400, 130); _panel(a, 20, 20); a.pointer(90, 30)
    b = C(400, 130); _panel(b, 20, 20); b.dotted(150, 20, 150, 90); b.dotted(151, 21, 148, 88); b.pointer(220, 30)
    d = C(400, 130); _panel(d, 220, 20); d.pointer(290, 30)
    return [("1. Drag starts", a, None), ("2. Dragging: only the outline moves", b, "no content repaint"), ("3. Released: one repaint", d, "Moved")]


CJK_FONT = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"   # any CJK-capable font works

def _cjk(size=14):
    key = ("cjk", size)
    if key not in _fonts:
        try:
            _fonts[key] = ImageFont.truetype(CJK_FONT, size * S)
        except OSError:
            _fonts[key] = font(size)
    return _fonts[key]

def _ime_field(c, x, y, committed, composing, sel=None, candidates=None, hot=0):
    c.rect(x, y, 230, 26, fill=WHITE); c.rect(x - 2, y - 2, 234, 30, outline=BLACK)
    tx = x + 4
    f = _cjk(14)
    c.d.text((tx * S, (y + 2) * S), committed, fill=BLACK, font=f)
    tx += c.d.textlength(committed, font=f) / S
    if composing:
        w = c.d.textlength(composing, font=f) / S
        if sel:
            a, b = sel
            sx = tx + c.d.textlength(composing[:a], font=f) / S; sw = c.d.textlength(composing[a:b], font=f) / S
            c.line(sx, y + 22, sx + sw, y + 22, BLACK, 2)
        c.d.text((tx * S, (y + 2) * S), composing, fill=BLACK, font=f)
        c.line(tx, y + 23, tx + w, y + 23, BLACK, 1)
        caret = tx + w
    else:
        caret = tx
    c.line(caret, y + 5, caret, y + 21)
    if candidates:
        cx0 = x + 4 + c.d.textlength(committed, font=f) / S
        c.rect(int(cx0), y + 32, 120, 22 * len(candidates) + 4, fill=WHITE)
        for n, cand in enumerate(candidates):
            cy = y + 34 + n * 22
            if n == hot: c.rect(int(cx0) + 1, cy, 118, 22, fill=BLACK, outline=None)
            c.d.text(((cx0 + 6) * S, (cy + 1) * S), f"{n + 1}  {cand}", fill=WHITE if n == hot else BLACK, font=_cjk(13))

@sb("1.19")
def ime():
    out = []
    a = C(312, 120); a.text(12, 18, "Name:"); _ime_field(a, 62, 16, "", "にほんご"); out.append(("1. Composing (underlined, not yet in the text)", a, "CompositionUpdated"))
    b = C(312, 150); b.text(12, 18, "Name:"); _ime_field(b, 62, 16, "", "日本語", sel=(0, 3), candidates=["日本語", "にほんご", "ニホンゴ"]); out.append(("2. Converting: candidates at the caret", b, None))
    d = C(312, 120); d.text(12, 18, "Name:"); _ime_field(d, 62, 16, "日本語", ""); out.append(("3. Committed", d, "TextEntered, TextChanged"))
    return out


def _guide(c, x, y, w, h):
    c.dotted(x, y, w, h, (60, 120, 200))

@sb("5.6")
def row_column():
    out = []
    for lab, W in [("Window 360 px wide", 360), ("Window 520 px wide (fields stretch, buttons stay right)", 520)]:
        c = C(W, 190); c.window(0, 0, W, 190, "Connect")
        fw = W - 40
        c.text(20, 36, "Server"); c.field(20, 58, fw, "pushrod.example.com")
        c.text(20, 88, "Port"); c.field(20, 110, fw, "8080")
        _guide(c, 14, 32, W - 28, 104)
        c.button(W - 196, 150, 84, 26, "Cancel"); c.button(W - 100, 150, 84, 26, "Connect", default=True)
        _guide(c, 14, 144, W - 28, 38); c.text(24, 156, "<- spacer ->", (60, 120, 200), 12)
        out.append((lab, c, None))
    return out

@sb("5.7")
def form_layout():
    rows = [("Name:", "Jane Doe"), ("Email address:", "jane.doe@example.com"), ("City:", "Anytown")]
    a = C(460, 170); a.window(0, 0, 460, 170, "Profile")
    lw = max(a.tw(r[0]) for r in rows)
    for n, (l, v) in enumerate(rows):
        y = 40 + n * 38
        a.text(20 + lw - a.tw(l), y + 2, l); a.field(int(32 + lw), y, int(460 - 52 - lw), v)
    a.line(int(26 + lw), 36, int(26 + lw), 150, (60, 120, 200))
    b = C(250, 250); b.window(0, 0, 250, 250, "Profile")
    for n, (l, v) in enumerate(rows):
        y = 34 + n * 66
        b.text(20, y, l); b.field(20, y + 24, 210, v)
    return [("Wide: labels right-aligned to the widest label", a, None), ("Narrow: labels move above fields", b, "LayoutChanged")]

@sb("5.8")
def grid_layout():
    out = []
    keys = [["7", "8", "9", "/"], ["4", "5", "6", "*"], ["1", "2", "3", "-"], ["0", None, ".", "+"]]
    for lab, W in [("Grid, 220 px", 220), ("Grid, 320 px (cells stretch)", 320)]:
        c = C(W, 230); c.field(12, 12, W - 24, "         42", h=28)
        cw = (W - 24 - 3 * 6) / 4
        for r, row in enumerate(keys):
            for col, k in enumerate(row):
                if k is None: continue
                x = 12 + col * (cw + 6); y = 52 + r * 42
                w = cw * 2 + 6 if k == "0" else cw
                c.button(int(x), y, int(w), 36, k)
        out.append((lab, c, None))
    return out

@sb("5.10")
def flow_layout():
    tags = ["rust", "gui", "sdl2", "atari", "retro", "widgets", "layout", "gpu"]
    out = []
    for lab, W in [("Wide", 420), ("Narrow (wraps)", 220)]:
        c = C(W, 120); x, y = 12, 14
        for t in tags:
            w = int(c.tw(t)) + 20
            if x + w > W - 12: x = 12; y += 32
            c.rect(x, y, w, 24, fill=WHITE); c.ctext(x, y, w, 24, t); x += w + 8
        out.append((lab, c, None))
    return out


import math

def _magnifier(c, x, y, color=DARK):
    c.ellipse(x, y, 10, 10, outline=color, width=1); c.line(x + 9, y + 9, x + 13, y + 13, color, 2)

def _clear_x(c, x, y, color=DARK):
    c.ellipse(x, y, 14, 14, fill=GRAY, outline=None)
    c.line(x + 4, y + 4, x + 10, y + 10, WHITE, 2); c.line(x + 10, y + 4, x + 4, y + 10, WHITE, 2)

def _eye(c, x, y, open_=True):
    c.ellipse(x, y + 2, 16, 10, outline=DARK)
    c.ellipse(x + 5, y + 4, 6, 6, fill=DARK if open_ else None, outline=DARK)
    if not open_: c.line(x, y + 12, x + 16, y, DARK, 1)

@sb("2.6", 3)
def field_accessories():
    a = C(300, 56); a.field(14, 16, 260, "", placeholder="      Search"); _magnifier(a, 20, 21)
    b = C(300, 56); b.field(14, 16, 260, "      widgets", caret=13, focused=True); _magnifier(b, 20, 21); _clear_x(b, 252, 20)
    d = C(300, 56); d.field(14, 16, 260, "", focused=True, placeholder="       Search"); _magnifier(d, 20, 21); d.line(40, 20, 40, 34)
    e = C(300, 56); x = e.labeled(10, 17, "Password:", 78); e.field(x, 16, 190, "•••••••"); _eye(e, x + 166, 21)
    f = C(300, 56); x = f.labeled(10, 17, "Password:", 78); f.field(x, 16, 190, "sample-pass"); _eye(f, x + 166, 21, open_=False)
    return [("1. Search field, empty", a, None), ("2. Typing: clear (×) appears", b, "TextChanged"),
            ("3. × clicked: text cleared", d, "TextChanged"), ("4. Password with reveal", e, None),
            ("5. Reveal clicked", f, "masking toggled")]


@sb("3.9")
def split_view():
    def frame(div, outline=None):
        c = C(470, 170); c.rect(14, 14, 442, 142, fill=WHITE)
        c.listbox(15, 15, div - 15, ["Inbox", "Drafts", "Sent", "Archive"], selected={0}, scroll=False)
        c.rect(div, 15, 6, 140, fill=GRAY, outline=None)
        for yy in (78, 84, 90): c.rect(div + 2, yy, 2, 2, fill=DARK, outline=None)
        c.text(div + 16, 22, "Subject: Weekly report", BLACK, 13); c.text(div + 16, 44, "From: jane.doe@example.com", DARK, 11)
        if outline: c.rect(outline, 15, 2, 140, fill=None, outline=BLACK); c.dotted(outline - 1, 15, 4, 140)
        return c
    a = frame(140); b = frame(140, outline=220); b.pointer(222, 80); d = frame(220)
    return [("1. Divider at 140 px", a, None), ("2. Dragging (outline mode shows a line)", b, None), ("3. Released at 220 px", d, "ValueChanged")]


def _img_icon(c, x, y, s=20, color=(70, 120, 180)):
    c.rect(x, y, s, s, fill=color, outline=None); c.poly([(x + 3, y + s - 3), (x + s / 2, y + 5), (x + s - 3, y + s - 3)], fill=WHITE)

@sb("2.21")
def image_button():
    out = []
    for lab, kind, kw in [("Image only", "only", {}), ("Image left", "left", {}), ("Image above", "above", {}),
                          ("Pressed", "left", dict(pressed=True)), ("Toggled", "only", dict(pressed=True)), ("Disabled", "left", dict(disabled=True))]:
        c = C(130, 80)
        if kind == "only":
            c.button(47, 22, 36, 36, "", pressed=kw.get("pressed", False)); _img_icon(c, 55, 30, 20, WHITE if kw.get("pressed") else (70, 120, 180))
        elif kind == "left":
            c.button(15, 25, 100, 30, "", **kw); _img_icon(c, 22, 30, 20, WHITE if kw.get("pressed") else (70, 120, 180))
            c.text(48, 29, "Upload", WHITE if kw.get("pressed") else BLACK)
            if kw.get("disabled"): c.dither(15, 25, 100, 30, PANEL)
        else:
            c.button(30, 10, 70, 60, ""); _img_icon(c, 55, 18, 20); c.ctext(30, 42, 70, 22, "Upload")
        out.append((lab, c, "Clicked" if lab == "Pressed" else ("Toggled(true)" if lab == "Toggled" else None)))
    return out


LINK = (30, 80, 170)
@sb("2.22")
def link_label():
    out = []
    for lab, color, ptr, focus, note in [("Normal", LINK, False, False, None), ("Hovered", LINK, True, False, None),
                                         ("Focused", LINK, False, True, "Return or Space activates"), ("Visited", (110, 60, 150), False, False, "Activated (opened)")]:
        c = C(190, 50); c.text(14, 14, "Learn more about Pushrod", color, 13)
        w = c.tw("Learn more about Pushrod", 13); c.line(14, 32, 14 + w, 32, color)
        if ptr:
            px, py = 70, 22; c.poly([(px + 4, py), (px + 7, py), (px + 7, py + 7), (px + 13, py + 7), (px + 13, py + 16), (px + 2, py + 16), (px, py + 10), (px + 4, py + 10)], fill=WHITE, outline=BLACK)
        if focus: c.dotted(10, 12, int(w) + 9, 24)
        out.append((lab, c, note))
    return out


def _dial(c, cx, cy, r, frac, disabled=False, focused=False, value=None, notches=11):
    c.ellipse(cx - r, cy - r, 2 * r, 2 * r, fill=WHITE)
    for n in range(notches):
        a = math.radians(225 - 270 * n / (notches - 1))
        c.line(cx + (r + 3) * math.cos(a), cy - (r + 3) * math.sin(a), cx + (r + 7) * math.cos(a), cy - (r + 7) * math.sin(a), DARK)
    a = math.radians(225 - 270 * frac)
    c.line(cx, cy, cx + (r - 5) * math.cos(a), cy - (r - 5) * math.sin(a), BLACK, 3)
    if value is not None: c.ctext(cx - 40, cy + r + 10, 80, 18, str(value), DARK, 12)
    if focused: c.dotted(cx - r - 10, cy - r - 10, 2 * r + 21, 2 * r + 21)
    if disabled: c.dither(cx - r - 8, cy - r - 8, 2 * r + 17, 2 * r + 17, PANEL)

@sb("3.12")
def dial():
    out = []
    for lab, f, kw, ptr, note in [("Idle", 0.3, {}, None, None), ("Dragging up", 0.65, {}, (120, 30), "ValueChanged (continuous)"),
                                  ("Focused (arrows step)", 0.65, dict(focused=True), None, None), ("Disabled", 0.3, dict(disabled=True), None, None)]:
        c = C(150, 130); _dial(c, 75, 58, 34, f, value=f"Volume {int(f * 100)}", **kw)
        if ptr: c.pointer(*ptr)
        out.append((lab, c, note))
    return out


def _spinner(c, cx, cy, r, hot, static=False):
    if static:
        c.poly([(cx - 8, cy - 11), (cx + 8, cy - 11), (cx, cy)], fill=DARK); c.poly([(cx - 8, cy + 11), (cx + 8, cy + 11), (cx, cy)], fill=DARK)
        c.line(cx - 10, cy - 11, cx + 10, cy - 11, BLACK, 2); c.line(cx - 10, cy + 11, cx + 10, cy + 11, BLACK, 2); return
    for n in range(8):
        a = math.radians(90 - n * 45)
        col = BLACK if n == hot else (DARK if (n - hot) % 8 == 7 else GRAY)
        c.line(cx + r * 0.45 * math.cos(a), cy - r * 0.45 * math.sin(a), cx + r * math.cos(a), cy - r * math.sin(a), col, 3)

@sb("4.10")
def spinner():
    out = []
    for n, hot in enumerate([0, 1, 2]):
        c = C(190, 50); _spinner(c, 26, 25, 11, hot); c.text(46, 15, "Connecting…"); out.append((f"Running, frame {n + 1}", c, None))
    c = C(190, 50); c.text(46, 15, "Connected"); out.append(("Stopped (hidden)", c, None))
    c = C(190, 50); _spinner(c, 26, 25, 11, 0, static=True); c.text(46, 15, "Connecting…"); out.append(("Outline mode / vintage: static", c, None))
    return out


# ---- Chassis code editor (Epic 11) -------------------------------------------------------
_MONO = [os.path.join(HERE, "..", "..", "assets", "DejaVuSansMono.ttf"),
         "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
         "/usr/share/fonts/truetype/freefont/FreeMono.ttf"]
def mono(size=13):
    key = ("mono", size)
    if key not in _fonts:
        path = next((p for p in _MONO if os.path.exists(p)), FONT_PATH)
        _fonts[key] = ImageFont.truetype(path, size * S)
    return _fonts[key]

KW, STR, COM, NUM, FN_ = (30, 60, 140), (40, 110, 50), (120, 120, 120), (150, 60, 40), (90, 50, 120)
CODE = [
    [("// Generated by Chassis. Edit inside the function body only.", COM)],
    [("fn ", KW), ("on_save_clicked", FN_), ("(state: &", BLACK), ("mut ", KW), ("AppState, ui: &", BLACK), ("mut ", KW), ("Ui) {", BLACK)],
    [("    ", BLACK), ("let ", KW), ("name = ui.text(ids::NAME_FIELD);", BLACK)],
    [("    ", BLACK), ("if ", KW), ("name.is_empty() {", BLACK)],
    [("        ui.alert(", BLACK), ("\"Please enter a name.\"", STR), (");", BLACK)],
    [("        ", BLACK), ("return", KW), (";", BLACK)],
    [("    }", BLACK)],
    [("    state.retries = ", BLACK), ("0", NUM), (";", BLACK)],
    [("    state.save(name);", BLACK)],
    [("}", BLACK)],
]
LH, CW = 18, 7.8   # line height and character width (13 px DejaVu Sans Mono)

def _code(c, x, y, w, h, lines=CODE, first=1, current=None, caret=None, sel=None, colors=True,
          marks=(), boxes=(), guarded=(), focused=True):
    """Draws an editor: gutter with line numbers, code, current-line band, selection, caret."""
    gw = max(34, len(str(first + len(lines) - 1)) * 7 + 14)
    c.rect(x, y, w, h, fill=WHITE)
    c.rect(x + 1, y + 1, gw, h - 2, fill=PANEL, outline=None)
    c.line(x + gw + 1, y + 1, x + gw + 1, y + h - 2, GRAY)
    tx = x + gw + 8
    for n, toks in enumerate(lines):
        ly = y + 4 + n * LH
        if ly + LH > y + h: break
        ln = first + n
        if ln in guarded:
            c.rect(x + gw + 2, ly, w - gw - 3, LH, fill=(226, 226, 226), outline=None)
        if current == ln:
            c.rect(x + gw + 2, ly, w - gw - 3, LH, fill=(244, 244, 244), outline=None)
        for (sx, sw) in [m for (l, *m) in marks if l == ln]:
            c.rect(tx + sx * CW - 1, ly + 1, sw * CW + 2, LH - 2, fill=None, outline=DARK)
        for (bl, bc) in boxes:
            if bl == ln: c.rect(tx + bc * CW - 1, ly + 1, CW + 2, LH - 2, fill=None, outline=BLACK)
        num = str(ln)
        c.d.text(((x + gw - 4) * S - c.d.textlength(num, font=mono(11)), (ly + 3) * S), num,
                 fill=BLACK if current == ln else MID, font=mono(11))
        col = 0
        text = "".join(t for t, _ in toks)
        sel_range = None
        if sel:
            (l1, c1), (l2, c2) = sel
            if l1 <= ln <= l2:
                a = c1 if ln == l1 else 0
                b = c2 if ln == l2 else len(text) + 1
                sel_range = (a, b)
        if sel_range and sel_range[1] > sel_range[0]:
            a, b = sel_range
            c.rect(tx + a * CW, ly, (b - a) * CW, LH, fill=BLACK, outline=None)
        for t, color in toks:
            for ch in t:
                inside = sel_range and sel_range[0] <= col < sel_range[1]
                fg = WHITE if inside else (color if colors else BLACK)
                c.d.text(((tx + col * CW) * S, (ly + 1) * S), ch, fill=fg, font=mono(13))
                col += 1
    if caret and focused:
        cl, cc = caret
        ly = y + 4 + (cl - first) * LH
        inv = sel and sel[0][0] <= cl <= sel[1][0]
        c.rect(tx + cc * CW - 1, ly + 1, 2, LH - 2, fill=WHITE if inv and cc < len("".join(t for t, _ in lines[cl - first])) else BLACK, outline=None)
    if focused:
        c.rect(x, y, w, h, fill=None, outline=BLACK, width=2)

W_ED, H_ED = 560, 4 + len(CODE) * LH + 6

@sb("11.2")
def code_editor_view():
    a = C(W_ED + 28, H_ED + 28); _code(a, 14, 14, W_ED, H_ED, current=3, caret=(3, 35))
    b = C(W_ED + 28, H_ED + 28)
    big = [[(f"    let row_{i} = table.row(", BLACK), (str(i), NUM), (");", BLACK)] for i in range(4096, 4106)]
    _code(b, 14, 14, W_ED - 16, H_ED, lines=big, first=4096, current=4100, caret=(4100, 4))
    b.scrollbar(14 + W_ED - 16, 14, H_ED, True, 0.82, 0.04)
    return [("1. Gutter with line numbers, flat syntax colors, current line, caret", a, None),
            ("2. Line 4,096 of a 5,000-line file: only visible lines are laid out and drawn", b, None)]

@sb("11.3")
def code_editor_editing():
    a = C(W_ED + 28, H_ED + 28); _code(a, 14, 14, W_ED, H_ED, current=3, caret=(3, 8), sel=((3, 8), (3, 12)))
    a.ibeam(14 + 42 + 10 * CW, 14 + 4 + 2 * LH)
    b = C(W_ED + 28, H_ED + 28); _code(b, 14, 14, W_ED, H_ED, current=9, caret=(9, 0), sel=((7, 4), (9, 0)))
    lines = [list(l) for l in CODE]
    lines[2] = [("    ", BLACK), ("let ", KW), ("user = ui.text(ids::NAME_FIELD);", BLACK)]
    d = C(W_ED + 28, H_ED + 28); _code(d, 14, 14, W_ED, H_ED, lines=lines, current=3, caret=(3, 12))
    return [("1. Double-click selects a word", a, "SelectionChanged"),
            ("2. Drag (or Shift+arrows) selects across lines", b, "SelectionChanged"),
            ("3. Typing replaces the selection", d, "TextChanged { range }")]

@sb("11.5")
def code_editor_highlighting():
    a = C(W_ED + 28, H_ED + 28); _code(a, 14, 14, W_ED, H_ED, colors=False, focused=False)
    b = C(W_ED + 28, H_ED + 28); _code(b, 14, 14, W_ED, H_ED, focused=False)
    return [("Before: plain text", a, None), ("After: tree-sitter highlighting (keywords, strings, numbers, comments)", b, None)]

@sb("11.6")
def code_editor_find():
    a = C(W_ED + 28, H_ED + 28)
    texts = ["".join(t for t, _ in l) for l in CODE]
    hits = [(n + 1, i) for n, t in enumerate(texts) if n > 1 for i in range(len(t)) if t.startswith("name", i) and not t[i - 1].isalnum() and t[i - 1] != "_"]
    first_hit = hits[0]
    _code(a, 14, 14, W_ED, H_ED, current=first_hit[0], marks=[(l, i, 4) for l, i in hits[1:]],
          sel=((first_hit[0], first_hit[1]), (first_hit[0], first_hit[1] + 4)))
    fx, fy = 14 + W_ED - 262, 14 + 6
    a.rect(fx, fy, 254, 30, fill=PANEL)
    a.field(fx + 6, fy + 4, 140, "name", caret=4, focused=True)
    a.text(fx + 152, fy + 7, f"1 of {len(hits)}", DARK, 12)
    a.glyph(fx + 196, fy + 6, 18, "up"); a.glyph(fx + 214, fy + 6, 18, "down")
    a.button(fx + 232, fy + 6, 18, 18, "×")
    return [("Ctrl+F: find bar inside the editor; matches outlined, current match selected", a, "FindChanged")]

@sb("11.7")
def code_editor_indent():
    pre = [list(l) for l in CODE[:4]] + [[("", BLACK)]]
    brace = len("".join(t for t, _ in CODE[3])) - 1
    a = C(W_ED + 28, 4 + 5 * LH + 34); _code(a, 14, 14, W_ED, 4 + 5 * LH + 6, lines=pre, current=4, caret=(4, brace + 1))
    post = [list(l) for l in CODE[:4]] + [[("", BLACK)]]
    b = C(W_ED + 28, 4 + 5 * LH + 34); _code(b, 14, 14, W_ED, 4 + 5 * LH + 6, lines=post, current=5, caret=(5, 8))
    c3 = C(W_ED + 28, H_ED + 28); _code(c3, 14, 14, W_ED, H_ED, current=7, caret=(7, 5), boxes=[(4, brace), (7, 4)])
    return [("1. Caret after '{'", a, None), ("2. Return: new line indented one level", b, "TextChanged"),
            ("3. Caret next to '}': its matching '{' is boxed", c3, None)]

@sb("11.8")
def code_editor_chassis():
    a = C(300, H_ED + 28)
    a.window(14, 14, 272, 150, "Contact")
    x = a.labeled(28, 58, "Name:", 50); a.field(x, 56, 170, "")
    a.button(200, 120, 70, 24, "Save", default=True); a.pointer(236, 132)
    b = C(W_ED + 28, H_ED + 28); _code(b, 14, 14, W_ED, H_ED, current=3, caret=(3, 4), guarded=(1, 2, 10))
    b.text(14 + W_ED - 190, 14 + H_ED - 22, "shaded lines are read-only", DARK, 11)
    return [("1. Chassis: double-click the Save button", a, "designer asks the editor for its handler"),
            ("2. Editor: stub inserted (or found), caret placed in the body; generated lines read-only", b, "insert_function / reveal_range")]


# ---- Chassis application (docs/ROADMAP_CHASSIS.md) ---------------------------------------
CW_W, CW_H = 1200, 760
LEFT_W, RIGHT_W = 250, 280
TOP = 58                     # menu bar (24) + toolbar (34)
STATUS_H = 24

def _ch_menubar(c):
    c.rect(0, 0, CW_W, 24, fill=WHITE, outline=None); c.line(0, 23, CW_W, 23, GRAY)
    x = 10
    for t in ["File", "Edit", "View", "Form", "Project", "Run", "Help"]:
        c.text(x, 3, t, BLACK, 13); x += c.tw(t, 13) + 22

def _ch_toolbar(c):
    c.rect(0, 24, CW_W, 34, fill=PANEL, outline=None); c.line(0, 57, CW_W, 57, GRAY)
    x = 8
    for group in [["New", "Open", "Save"], ["Undo", "Redo"], ["Run   "]]:
        for t in group:
            if t.startswith("Run"):
                w = int(c.tw("Run", 12)) + 36
                c.button(x, 29, w, 24, ""); c.text(x + 9, 32, "Run", BLACK, 12); c.tri(x + w - 18, 36, 9, "right")
            else:
                w = int(c.tw(t, 12)) + 18
                c.button(x, 29, w, 24, ""); c.ctext(x, 29, w, 24, t, BLACK, 12)
            x += w + 4
        c.line(x + 3, 30, x + 3, 52, GRAY); x += 10

def _ch_section(c, x, y, w, title, open_=True, count=None):
    c.tri(x + 8, y + 7, 8, "down" if open_ else "right", DARK)
    c.text(x + 22, y + 1, title + (f"  ({count})" if count is not None else ""), BLACK, 13)
    return y + 22

def _ch_row(c, x, y, w, label, selected=False, icon=None, indent=22):
    if selected: c.rect(x + 1, y, w - 2, 20, fill=BLACK, outline=None)
    fg = WHITE if selected else BLACK
    tx = x + indent
    if icon: icon(c, tx, y + 3, fg); tx += 20
    c.text(tx, y + 1, label, fg, 13)
    return y + 20

def _ico_form(c, x, y, fg):
    c.rect(x, y, 14, 13, fill=None, outline=fg); c.line(x, y + 4, x + 13, y + 4, fg)
def _ico_image(c, x, y, fg):
    c.rect(x, y, 14, 13, fill=None, outline=fg); c.poly([(x + 2, y + 11), (x + 6, y + 5), (x + 10, y + 11)], fill=fg)
def _ico_btn(c, x, y, fg):
    c.rect(x, y + 2, 14, 9, fill=None, outline=fg)
def _ico_lbl(c, x, y, fg):
    c.text(x + 1, y - 3, "A", fg, 13)
def _ico_chk(c, x, y, fg):
    c.rect(x + 1, y + 1, 11, 11, fill=None, outline=fg); c.line(x + 3, y + 6, x + 6, y + 9, fg); c.line(x + 6, y + 9, x + 11, y + 2, fg)
def _ico_rad(c, x, y, fg):
    c.ellipse(x + 1, y + 1, 11, 11, outline=fg); c.ellipse(x + 4, y + 4, 5, 5, fill=fg, outline=None)
def _ico_fld(c, x, y, fg):
    c.rect(x, y + 2, 14, 9, fill=None, outline=fg); c.line(x + 3, y + 4, x + 3, y + 9, fg)
def _ico_sep(c, x, y, fg):
    c.line(x, y + 6, x + 14, y + 6, fg)
def _ico_link(c, x, y, fg):
    c.text(x, y - 3, "a", fg, 13); c.line(x, y + 12, x + 12, y + 12, fg)

PALETTE = [("Label", _ico_lbl), ("Boxed Label", _ico_btn), ("Button", _ico_btn), ("Image Button", _ico_image),
           ("Link Label", _ico_link), ("Check Box", _ico_chk), ("Radio Button", _ico_rad), ("Text Field", _ico_fld),
           ("Separator", _ico_sep)]

def _ch_left(c, palette_pressed=None):
    c.rect(0, TOP, LEFT_W, CW_H - TOP - STATUS_H, fill=WHITE, outline=None)
    c.line(LEFT_W, TOP, LEFT_W, CW_H - STATUS_H, GRAY)
    y = TOP + 8
    c.text(10, y, "PROJECT", DARK, 11); c.text(LEFT_W - 96, y, "contacts.prsc", MID, 11); y += 20
    y = _ch_section(c, 6, y, LEFT_W, "Forms", True, 2)
    y = _ch_row(c, 0, y, LEFT_W, "MainForm", icon=_ico_form, indent=28)
    y = _ch_row(c, 0, y, LEFT_W, "ContactDialog", selected=True, icon=_ico_form, indent=28)
    y = _ch_section(c, 6, y + 2, LEFT_W, "Menus", False, 1)
    y = _ch_section(c, 6, y, LEFT_W, "Strings", False, 12)
    y = _ch_section(c, 6, y, LEFT_W, "Images", True, 1)
    y = _ch_row(c, 0, y, LEFT_W, "logo.png", icon=_ico_image, indent=28)
    y = _ch_section(c, 6, y + 2, LEFT_W, "Icons", False, 4)
    # splitter
    sy = y + 8
    c.rect(0, sy, LEFT_W, 6, fill=PANEL, outline=None)
    for xx in (LEFT_W // 2 - 6, LEFT_W // 2, LEFT_W // 2 + 6): c.rect(xx, sy + 2, 2, 2, fill=DARK, outline=None)
    y = sy + 14
    c.text(10, y, "PALETTE", DARK, 11); y += 18
    c.field(8, y, LEFT_W - 16, "", placeholder="Search widgets"); y += 30
    y = _ch_section(c, 6, y, LEFT_W, "Basic", True)
    for name, ico in PALETTE:
        y = _ch_row(c, 0, y, LEFT_W, name, selected=(name == palette_pressed), icon=ico, indent=28)
    for sec in ["Values", "Containers", "Lists"]:
        y = _ch_section(c, 6, y + 2, LEFT_W, sec, False)

def _ch_tabs(c, active="Designer", title="ContactDialog"):
    x0 = LEFT_W + 1; w = CW_W - LEFT_W - RIGHT_W - 2
    c.rect(x0, TOP, w, 30, fill=PANEL, outline=None); c.line(x0, TOP + 29, x0 + w, TOP + 29, BLACK)
    x = x0 + 10
    for t in ["Designer", "Code"]:
        tw = int(c.tw(t, 13)) + 28
        if t == active:
            c.rect(x, TOP + 4, tw, 26, fill=WHITE, outline=BLACK); c.line(x + 1, TOP + 29, x + tw - 1, TOP + 29, WHITE)
        else:
            c.rect(x, TOP + 6, tw, 23, fill=PANEL, outline=MID)
        c.ctext(x, TOP + 5, tw, 24, t, BLACK if t == active else DARK, 13)
        x += tw + 4
    c.text(x0 + w - c.tw(title + "   F12 switches", 12) - 10, TOP + 9, title + "   F12 switches", MID, 12)
    return x0, TOP + 30, w, CW_H - STATUS_H - TOP - 30

FORM_X, FORM_Y, FORM_W, FORM_H = 330, 140, 440, 300
def _ch_grid(c, x, y, w, h):
    c.rect(x, y, w, h, fill=(226, 226, 226), outline=None)
    for gy in range(y + 8, y + h, 16):
        for gx in range(x + 8, x + w, 16): c.rect(gx, gy, 1, 1, fill=MID, outline=None)

def _ch_form(c, selected="OK", has_code=("OK",), extra=None):
    fx, fy = FORM_X, FORM_Y
    c.rect(fx, fy, FORM_W, FORM_H, fill=PANEL)
    c.rect(fx, fy, FORM_W, 26, fill=WHITE); c.ctext(fx, fy, FORM_W, 26, "Contact", BLACK, 13)
    for i in range(0, FORM_H - 30, 16):
        for j in range(8, FORM_W, 16): c.rect(fx + j, fy + 34 + i, 1, 1, fill=GRAY, outline=None)
    rows = [("Name:", "Jane Doe"), ("E-mail:", "jane.doe@example.com"), ("Phone:", "555-0100")]
    for n, (lab, val) in enumerate(rows):
        yy = fy + 48 + n * 36
        c.text(fx + 24, yy + 3, lab, BLACK, 13); c.field(fx + 100, yy, 300, val)
    c.checkbox(fx + 100, fy + 158, "Send me product updates", state="on")
    c.line(fx + 16, fy + 200, fx + FORM_W - 16, fy + 200, GRAY)
    widgets = {"Cancel": (fx + FORM_W - 210, fy + 230, 90, 28), "OK": (fx + FORM_W - 110, fy + 230, 90, 28)}
    c.button(*widgets["Cancel"], "Cancel")
    c.button(*widgets["OK"], "OK", default=True)
    for name in has_code:
        x, y, w, h = widgets[name]
        c.poly([(x + w - 9, y), (x + w, y), (x + w, y + 9)], fill=BLACK)
    if selected:
        x, y, w, h = widgets[selected]
        c.rect(x - 3, y - 3, w + 6, h + 6, fill=None, outline=BLACK)
        for hx in (x - 6, x + w / 2 - 3, x + w):
            for hy in (y - 6, y + h / 2 - 3, y + h):
                if hx == x + w / 2 - 3 and hy == y + h / 2 - 3: continue
                c.rect(int(hx), int(hy), 6, 6, fill=BLACK, outline=None)
    return widgets

def _prop(c, x, y, w, label, value=None, kind="field", vw=None):
    c.text(x + 10, y + 3, label, DARK, 12)
    vx = x + 98; vw = vw or (w - 108)
    if kind == "field": c.field(vx, y, vw, value, h=22)
    elif kind == "check":
        cx = vx
        for lab, on in value:
            c.checkbox(cx, y + 1, lab, state="on" if on else "off"); cx += int(c.tw(lab)) + 34
    elif kind == "choice":
        c.rect(vx, y, vw, 22, fill=WHITE); c.text(vx + 6, y + 3, value, BLACK, 13); c.tri(vx + vw - 16, y + 8, 8, "down")
    elif kind == "color":
        c.rect(vx, y + 2, 18, 18, fill=value[1]); c.field(vx + 24, y, vw - 24, value[0], h=22)
    return y + 28

def _ch_head(c, x, y, w, t):
    c.rect(x, y, w, 20, fill=PANEL, outline=None); c.text(x + 10, y + 2, t, DARK, 11)
    return y + 26

def _ch_right(c, tab="Properties", events=None):
    x = CW_W - RIGHT_W
    c.rect(x, TOP, RIGHT_W, CW_H - TOP - STATUS_H, fill=WHITE, outline=None)
    c.line(x, TOP, x, CW_H - STATUS_H, GRAY)
    y = TOP + 8
    c.text(x + 10, y, "INSPECTOR", DARK, 11); c.text(x + RIGHT_W - 22, y - 3, "×", DARK, 15); y += 20
    c.text(x + 10, y, "OkButton", BLACK, 17); y += 26
    c.text(x + 10, y, "Button  ·  ID 7  ·  pushrod.button", MID, 12); y += 24
    tx = x + 10
    for t in ["Properties", "Events (2)" if events else "Events (1)"]:
        tw = int(c.tw(t, 13)) + 24
        act = t.startswith(tab)
        c.rect(tx, y, tw, 24, fill=WHITE if act else PANEL, outline=BLACK if act else MID)
        if act: c.line(tx + 1, y + 23, tx + tw - 1, y + 23, WHITE)
        c.ctext(tx, y, tw, 24, t, BLACK if act else DARK, 13); tx += tw + 4
    c.line(x + 1, y + 23, x + RIGHT_W - 1, y + 23, BLACK); c.line(x + 10, y + 23, tx - 4, y + 23, BLACK)
    if tab == "Properties":
        act = [(x + 10, )]
    y += 32
    if tab == "Properties":
        y = _ch_head(c, x, y, RIGHT_W, "IDENTITY")
        y = _prop(c, x, y, RIGHT_W, "Name", "OkButton")
        y = _prop(c, x, y, RIGHT_W, "Text", "OK")
        y = _ch_head(c, x, y + 2, RIGHT_W, "LAYOUT")
        c.text(x + 10, y + 3, "Position", DARK, 12); c.field(x + 98, y, 64, "320"); c.field(x + 168, y, 64, "230"); y += 28
        c.text(x + 10, y + 3, "Size", DARK, 12); c.field(x + 98, y, 64, "90"); c.field(x + 168, y, 64, "28"); y += 28
        y = _ch_head(c, x, y + 2, RIGHT_W, "BEHAVIOR")
        c.checkbox(x + 98, y + 1, "Default", state="on"); c.checkbox(x + 180, y + 1, "Cancel"); y += 24
        c.checkbox(x + 98, y + 1, "Toggle"); c.checkbox(x + 180, y + 1, "Disabled"); y += 24
        c.checkbox(x + 98, y + 1, "Hidden"); y += 30
        y = _ch_head(c, x, y, RIGHT_W, "APPEARANCE")
        y = _prop(c, x, y, RIGHT_W, "Font", "Body (14)", kind="choice")
        y = _prop(c, x, y, RIGHT_W, "Text color", ("#000000", BLACK), kind="color")
        y = _prop(c, x, y, RIGHT_W, "Tooltip", "Save the contact")
    else:
        y = _ch_head(c, x, y, RIGHT_W, "EVENTS   (dot = has code)")
        for ev, handler in events:
            if handler:
                c.ellipse(x + 12, y + 6, 8, 8, fill=BLACK, outline=None)
                c.text(x + 28, y + 2, ev, BLACK, 13); c.text(x + 132, y + 3, handler, DARK, 12)
            else:
                c.text(x + 28, y + 2, ev, DARK, 13); c.text(x + 132, y + 3, "add…", MID, 12)
            c.line(x + 10, y + 24, x + RIGHT_W - 10, y + 24, (230, 230, 230))
            y += 26

def _ch_status(c, text):
    y = CW_H - STATUS_H
    c.rect(0, y, CW_W, STATUS_H, fill=PANEL, outline=None); c.line(0, y, CW_W, y, GRAY)
    x = 10
    for part in text:
        c.text(x, y + 4, part, DARK, 12); x += c.tw(part, 12) + 14
        c.line(x - 7, y + 4, x - 7, y + 20, GRAY)

def _chassis_window(tab="Designer", palette_pressed=None, right_tab="Properties", events=None, status=None):
    c = C(CW_W, CW_H, bg=WHITE)
    _ch_menubar(c); _ch_toolbar(c); _ch_left(c, palette_pressed)
    x0, y0, w, h = _ch_tabs(c, tab)
    _ch_right(c, right_tab, events)
    _ch_status(c, status or ["contacts.prsc", "ContactDialog · 9 widgets", "OkButton (Button)  x 320  y 230  90 × 28", "Grid 8 px"])
    return c, (x0, y0, w, h)

EVENTS_OK = [("Clicked", "on_ok_clicked"), ("FocusGained", None), ("FocusLost", None),
             ("PointerEntered", None), ("PointerExited", None), ("KeyPressed", None)]

@sb("chassis", 1)
def chassis_designer():
    c, (x0, y0, w, h) = _chassis_window()
    _ch_grid(c, x0, y0, w, h); _ch_form(c)
    return [("Designer: OK button selected; its properties in the inspector on the right", c, None)]

@sb("chassis", 2)
def chassis_drag():
    c, (x0, y0, w, h) = _chassis_window(palette_pressed="Text Field",
        status=["contacts.prsc", "ContactDialog · 9 widgets", "Dragging Text Field  x 100  y 194", "Snaps to grid and to neighbors"])
    _ch_grid(c, x0, y0, w, h); _ch_form(c, selected=None)
    dx, dy = FORM_X + 100, FORM_Y + 194
    c.dotted(dx, dy, 300, 22)
    c.dotted(dx, FORM_Y + 40, 1, 180)
    c.pointer(dx + 140, dy + 10)
    return [("Drag from the palette: outline shows where it lands; dotted guide aligns with the fields above", c, "drop adds the widget and selects it")]

@sb("chassis", 3)
def chassis_events_menu():
    c, (x0, y0, w, h) = _chassis_window()
    _ch_grid(c, x0, y0, w, h); wid = _ch_form(c)
    x, y, ww, hh = wid["OK"]
    mx, my = x + 40, y + 16
    items = [("EVENTS", "", {"header"}), ("     Clicked", "on_ok_clicked", {"code"}), ("     FocusGained", "", set()),
             ("     FocusLost", "", set()), ("     PointerEntered", "", set()), ("     PointerExited", "", set()),
             ("     KeyPressed", "", set()), ("", "", {"sep"}), ("Cut", "Ctrl+X", set()), ("Copy", "Ctrl+C", set()),
             ("Delete", "Del", set()), ("", "", {"sep"}), ("Bring to Front", "", set()), ("Send to Back", "", set()),
             ("", "", {"sep"}), ("Inspector", "F4", set())]
    mw = 250
    mh = sum(9 if "sep" in f else 22 for _, _, f in items) + 4
    c.rect(mx, my, mw, mh, fill=WHITE)
    cy = my + 2
    for n, (label, sc, f) in enumerate(items):
        if "sep" in f: c.line(mx + 4, cy + 4, mx + mw - 5, cy + 4, MID); cy += 9; continue
        if "header" in f: c.text(mx + 10, cy + 4, label, MID, 11); cy += 22; continue
        hot = label.strip() == "FocusGained"
        if hot: c.rect(mx + 1, cy, mw - 2, 22, fill=BLACK, outline=None)
        col = WHITE if hot else BLACK
        c.text(mx + 10, cy + 1, label, col, 13)
        if "code" in f: c.ellipse(mx + 12, cy + 7, 8, 8, fill=col, outline=None)
        if sc: c.text(mx + mw - 10 - c.tw(sc, 12), cy + 3, sc, DARK if not hot else WHITE, 12)
        cy += 22
    c.pointer(mx + 120, my + 2 + 22 + 22 + 10)
    return [("Right-click: the widget's events (a dot marks events with code); choosing one opens its handler", c, "choosing FocusGained creates on_ok_focus_gained")]

@sb("chassis", 4)
def chassis_code():
    ev = [("Clicked", "on_ok_clicked"), ("FocusGained", "on_ok_focus_gained"), ("FocusLost", None),
          ("PointerEntered", None), ("PointerExited", None), ("KeyPressed", None)]
    c, (x0, y0, w, h) = _chassis_window(tab="Code", right_tab="Events", events=ev,
        status=["contacts.prsc", "contact_dialog.rs", "Ln 9, Col 5", "Rust", "Generated lines are read-only"])
    lines = [
        [("// ContactDialog event handlers. Chassis generates the signatures;", COM)],
        [("// write code inside the function bodies.", COM)],
        [("fn ", KW), ("on_ok_clicked", FN_), ("(state: &", BLACK), ("mut ", KW), ("AppState, ui: &", BLACK), ("mut ", KW), ("Ui) {", BLACK)],
        [("    state.save(ui.text(ids::NAME_FIELD));", BLACK)],
        [("    ui.close_form();", BLACK)],
        [("}", BLACK)],
        [("", BLACK)],
        [("fn ", KW), ("on_ok_focus_gained", FN_), ("(state: &", BLACK), ("mut ", KW), ("AppState, ui: &", BLACK), ("mut ", KW), ("Ui) {", BLACK)],
        [("    ", BLACK)],
        [("}", BLACK)],
    ]
    c.rect(x0, y0, w, h, fill=WHITE, outline=None)
    _code(c, x0 + 1, y0 + 1, w - 2, h - 2, lines=lines, current=9, caret=(9, 4), guarded=(1, 2, 3, 6, 8, 10))
    return [("Code tab: the new handler is inserted and the caret placed in its body; the inspector lists the events", c, None)]


def main():
    for key, fn in SB.items():
        storyboard(fn(), os.path.join(HERE, f"{key}.png"))
        print("wrote", f"{key}.png")


if __name__ == "__main__":
    main()
