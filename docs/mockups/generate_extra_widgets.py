#!/usr/bin/env python3
"""
Generates the mockups for the pushrod-extra-widgets roadmap (docs/mockups/xw-*.png).

Same rules as generate.py, whose drawing helpers this script reuses: flat fills, 1 px black
borders, Open Sans text, inverted colors when pressed or selected, a 50% dither when disabled,
a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners.
Each image is a storyboard: numbered frames showing one interaction step each, with the event
emitted (if any) underneath.

Usage:  python3 docs/mockups/generate_extra_widgets.py
Needs:  Pillow  (pip install pillow)

Sample data must be fictional: no real names, e-mail addresses, places, or paths
(use placeholders such as "Jane Doe", "jane.doe@example.com", "Anytown", "/home/user").
"""
import math
import os

from generate import (C, storyboard, _picture, _icon, _choice, _tabs, _browser, _dial,
                      PANEL, BLACK, WHITE, GRAY, MID, DARK, HERE)

XW = {}


def xw(key):
    """Registers a storyboard for extra-widgets issue `key` (written to xw-<key>-1.png)."""
    def deco(fn):
        XW[key] = fn
        return fn
    return deco


# ---- small helpers ---------------------------------------------------------
def popover(c, x, y, w, h, anchor_x=None):
    """A popover panel: one-pixel border, no shadow, a small notch toward its anchor."""
    c.rect(x, y, w, h, fill=WHITE)
    if anchor_x is not None:
        c.poly([(anchor_x - 7, y + 1), (anchor_x + 7, y + 1), (anchor_x, y - 7)], fill=WHITE)
        c.line(anchor_x - 7, y, anchor_x, y - 7); c.line(anchor_x, y - 7, anchor_x + 7, y)


def mini_calendar(c, x, y, sel=24, today=25, month="November 2026", rng=None, cell=26, marks=()):
    """A month grid for November 2026 (starts on Sunday)."""
    c.glyph(x, y, 18, "left"); c.ctext(x, y - 2, cell * 7, 22, month, BLACK, 13); c.glyph(x + cell * 7 - 18, y, 18, "right")
    for n, d in enumerate("SMTWTFS"):
        c.ctext(x + n * cell, y + 22, cell, 18, d, DARK, 11)
    day = 1
    for row in range(5):
        for col in range(7):
            if day > 30:
                break
            dx, dy = x + col * cell, y + 42 + row * 22
            inside = rng and rng[0] <= day <= rng[1]
            if day == sel or inside:
                c.rect(dx + 1, dy, cell - 2, 21, fill=BLACK, outline=None)
                c.ctext(dx, dy, cell, 21, str(day), WHITE, 12)
            else:
                c.ctext(dx, dy, cell, 21, str(day), BLACK, 12)
            if day == today and day != sel and not inside:
                c.rect(dx + 1, dy, cell - 2, 21, fill=None)
            if day in marks:
                c.rect(dx + cell // 2 - 1, dy + 18, 3, 2, fill=BLACK if day != sel else WHITE, outline=None)
            day += 1
    return 42 + 5 * 22


def swatch(c, x, y, s, color, selected=False):
    c.rect(x, y, s, s, fill=color)
    if selected:
        c.rect(x - 3, y - 3, s + 6, s + 6, fill=None, width=2)


def tool_icon(c, x, y, s, kind, inverted=False):
    """Simple pictograms for palette cells."""
    fg = WHITE if inverted else BLACK
    m = s // 4
    if kind == "arrow":
        c.poly([(x + m, y + m), (x + m, y + s - m), (x + s // 2, y + s // 2 + 2), (x + s - m, y + s - m - 2)], fill=fg)
    elif kind == "pencil":
        c.line(x + m, y + s - m, x + s - m, y + m, fg, 3)
    elif kind == "rect":
        c.rect(x + m, y + m + 2, s - 2 * m, s - 2 * m - 4, outline=fg)
    elif kind == "oval":
        c.ellipse(x + m, y + m + 2, s - 2 * m, s - 2 * m - 4, outline=fg)
    elif kind == "text":
        c.ctext(x, y, s, s, "A", fg, 15)
    elif kind == "fill":
        c.rect(x + m, y + m + 3, s - 2 * m, s - 2 * m - 6, fill=fg, outline=None)
    elif kind == "line":
        c.line(x + m, y + s // 2, x + s - m, y + s // 2, fg, 2)
    elif kind == "button":
        c.rect(x + m - 2, y + m + 3, s - 2 * m + 4, s - 2 * m - 6, outline=fg); c.line(x + m + 2, y + s // 2, x + s - m - 2, y + s // 2, fg)
    elif kind == "check":
        c.rect(x + m, y + m + 1, s // 3, s // 3, outline=fg); c.line(x + m + s // 3 + 3, y + m + 6, x + s - m, y + m + 6, fg)
    elif kind == "field":
        c.rect(x + m - 2, y + s // 2 - 5, s - 2 * m + 4, 10, outline=fg); c.line(x + m + 1, y + s // 2 - 3, x + m + 1, y + s // 2 + 3, fg)
    elif kind == "list":
        c.rect(x + m - 2, y + m, s - 2 * m + 4, s - 2 * m, outline=fg)
        for n in range(3):
            c.line(x + m + 1, y + m + 4 + n * 5, x + s - m - 2, y + m + 4 + n * 5, fg)
    elif kind == "label":
        c.ctext(x, y, s, s, "Ab", fg, 12)


def checkmark(c, x, y, color=BLACK):
    """A check mark drawn as two strokes (Open Sans has no check-mark glyph)."""
    c.line(x, y + 6, x + 4, y + 10, color, 2); c.line(x + 4, y + 10, x + 11, y + 1, color, 2)


def arrow(c, cx, cy, angle, color=BLACK, double=False, both=False, size=11):
    """An arrow character drawn with lines (Open Sans has no arrow glyphs)."""
    a = math.radians(angle); dx, dy = math.cos(a), -math.sin(a)
    x1, y1, x2, y2 = cx - dx * size, cy - dy * size, cx + dx * size, cy + dy * size
    if double:
        px, py = -dy * 2, dx * 2
        c.line(x1 + px, y1 + py, x2 - dx * 3 + px, y2 - dy * 3 + py, color); c.line(x1 - px, y1 - py, x2 - dx * 3 - px, y2 - dy * 3 - py, color)
    else:
        c.line(x1, y1, x2, y2, color, 2)
    for ex, ey, sgn in ([(x2, y2, 1)] + ([(x1, y1, -1)] if both else [])):
        for t in (150, -150):
            b = math.radians(angle + t) if sgn == 1 else math.radians(angle + 180 + t)
            c.line(ex, ey, ex + math.cos(b) * 6, ey - math.sin(b) * 6, color, 2)


def tree_rows(c, x, y, w, rows, sel=None, row_h=20, check=None):
    """rows: (depth, label, state) state in {'open','closed','leaf'}."""
    c.rect(x, y, w, row_h * len(rows) + 2, fill=WHITE)
    for n, (depth, label, state) in enumerate(rows):
        ry = y + 1 + n * row_h
        hot = n == sel
        if hot:
            c.rect(x + 1, ry, w - 2, row_h, fill=BLACK, outline=None)
        col = WHITE if hot else BLACK
        tx = x + 6 + depth * 16
        if state == "closed":
            c.tri(tx, ry + 6, 8, "right", col)
        elif state == "open":
            c.tri(tx, ry + 7, 8, "down", col)
        c.text(tx + 14, ry, label, col, 13)


def window(c, x, y, w, h, title):
    c.rect(x, y, w, h, fill=PANEL); c.rect(x, y, w, 24, fill=WHITE); c.ctext(x, y, w, 24, title, BLACK, 13)
    return x, y + 24


# ============================================================================
# Epic 1: Crate and foundations
# ============================================================================
@xw("1.1")
def crate():
    c = C(470, 170)
    boxes = [("pushrod (core)", 160, 130), ("pushrod-widgets", 20, 70), ("pushrod-extra-widgets", 250, 70), ("your application", 135, 12)]
    for t, x, y in boxes:
        w = int(c.tw(t, 13)) + 24
        c.rect(x, y, w, 30, fill=WHITE if "extra" not in t else BLACK)
        c.ctext(x, y, w, 30, t, BLACK if "extra" not in t else WHITE, 13)
    c.line(90, 100, 200, 130); c.line(330, 100, 240, 130); c.line(170, 85, 250, 85)
    c.line(200, 42, 90, 70); c.line(220, 42, 320, 70)
    c.text(176, 70, "uses", DARK, 11)
    r = C(330, 170)
    r.text(12, 10, "WidgetRegistry", BLACK, 13)
    rows = ["pushrod.button", "pushrod.list_view", "extra.tree_view", "extra.property_grid", "extra.drawer"]
    r.listbox(12, 34, 300, rows, selected={2}, scroll=False)
    return [("Crate layout: depends only on public APIs", c, None),
            ("register() adds types under the extra. prefix", r, "forms and Chassis load them by type name")]


@xw("1.2")
def gallery():
    c = C(560, 240); x, y = window(c, 0, 0, 560, 240, "Extra widgets gallery")
    _tabs(c, 10, y + 8, 540, 170, ["Buttons", "Pickers", "Data views", "Panels", "Dialogs"], 1)
    c.text(26, y + 46, "Date:", BLACK, 13); c.field(80, y + 44, 110, "11/24/2026"); c.glyph(190, y + 44, 22, "down")
    c.text(26, y + 84, "Color:", BLACK, 13); c.rect(80, y + 80, 44, 26, fill=WHITE); c.rect(84, y + 84, 36, 18, fill=(70, 120, 70), outline=None)
    c.text(26, y + 122, "Path:", BLACK, 13); c.field(80, y + 120, 200, "/home/user/notes"); c.button(282, y + 120, 30, 22, "…")
    c.rect(0, 216, 560, 24, fill=WHITE); c.text(8, 218, "Last event: ColorChanged { id: 12 }", DARK, 12)
    return [("Gallery: one tab per epic, events logged at the bottom", c, "golden screenshots per tab")]


@xw("1.3")
def popover_sb():
    a = C(250, 170); a.text(12, 16, "Due:", BLACK, 13); a.field(54, 14, 110, "11/24/2026"); a.glyph(164, 14, 22, "down")
    b = C(250, 170); b.text(12, 16, "Due:", BLACK, 13); b.field(54, 14, 110, "11/24/2026"); b.glyph(164, 14, 22, "down", pressed=True)
    popover(b, 40, 46, 196, 112, anchor_x=175); b.text(52, 56, "Any content:", DARK, 12)
    b.checkbox(52, 78, "Remind me", "on"); b.button(52, 112, 80, 24, "Done")
    d = C(250, 170); d.text(12, 16, "Due:", BLACK, 13); d.field(54, 14, 110, "11/24/2026"); d.glyph(164, 14, 22, "down")
    popover(d, 40, 46, 196, 112, anchor_x=175); d.text(52, 56, "Any content:", DARK, 12)
    d.checkbox(52, 78, "Remind me", "on"); d.button(52, 112, 80, 24, "Done"); d.pointer(20, 140)
    e = C(250, 170); e.text(12, 16, "Due:", BLACK, 13); e.field(54, 14, 110, "11/24/2026", focused=True); e.glyph(164, 14, 22, "down")
    f = C(250, 170); f.text(12, 132, "Near the bottom:", DARK, 12); f.field(120, 130, 90, "Medium"); f.glyph(210, 130, 22, "down", pressed=True)
    popover(f, 70, 20, 170, 96); f.poly([(214, 115), (228, 115), (221, 123)], fill=WHITE); f.text(82, 30, "Flipped above", DARK, 12)
    return [("1. Anchor control", a, None), ("2. Opened: notch points at the anchor", b, "PopoverOpened"),
            ("3. Click outside", d, "PopoverClosed(outside)"), ("4. Closed: focus back on the anchor", e, None),
            ("Placement flips to stay inside the window", f, None)]


# ============================================================================
# Epic 2: Buttons and value controls
# ============================================================================
def _menu_btn(c, x, y, label, pressed=False):
    w = int(c.tw(label)) + 40
    c.rect(x, y, w, 26, fill=BLACK if pressed else WHITE); c.text(x + 10, y + 2, label, WHITE if pressed else BLACK)
    c.tri(x + w - 18, y + 10, 8, "down", WHITE if pressed else BLACK)
    return w


def _split_btn(c, x, y, label, main=False, arrow=False):
    w = int(c.tw(label)) + 20
    c.rect(x, y, w, 26, fill=BLACK if main else WHITE); c.ctext(x, y, w, 26, label, WHITE if main else BLACK)
    c.rect(x + w - 1, y, 24, 26, fill=BLACK if arrow else WHITE); c.tri(x + w + 7, y + 10, 8, "down", WHITE if arrow else BLACK)
    return w + 23


@xw("2.1")
def menu_button():
    a = C(220, 150); _menu_btn(a, 14, 14, "Sort by")
    b = C(220, 150); _menu_btn(b, 14, 14, "Sort by", pressed=True)
    b.menu(14, 40, 150, [("Name", "", ""), ("Date", "", ""), ("Size", "", ""), ("", "", "sep"), ("Descending", "", "")], hover=1)
    c = C(230, 150); _split_btn(c, 14, 14, "Save", main=True); c.pointer(40, 26)
    d = C(230, 150); _split_btn(d, 14, 14, "Save", arrow=True)
    d.menu(14, 40, 220, [("Save", "Ctrl+S", ""), ("Save As…", "Ctrl+Shift+S", ""), ("Save a Copy…", "", "")], hover=1)
    return [("1. Menu button", a, None), ("2. Clicked: menu opens", b, "MenuOpened; item runs its action"),
            ("3. Split button: main part", c, "Clicked (default action)"), ("4. Arrow part opens the variants", d, "MenuOpened")]


def _segments(c, x, y, labels, sel, w=None, disabled=()):
    cx = x
    for n, t in enumerate(labels):
        sw = w or int(c.tw(t)) + 24
        on = n in sel
        c.rect(cx, y, sw, 26, fill=BLACK if on else WHITE); c.ctext(cx, y, sw, 26, t, WHITE if on else BLACK, 13)
        if n in disabled:
            c.dither(cx, y, sw, 26, PANEL)
        cx += sw - 1
    return cx - x


@xw("2.2")
def segmented():
    a = C(250, 60); _segments(a, 14, 16, ["Day", "Week", "Month"], {1}, 70)
    b = C(250, 60); _segments(b, 14, 16, ["Day", "Week", "Month"], {2}, 70); b.pointer(190, 30)
    d = C(210, 60); _segments(d, 14, 16, ["B", "I", "U", "S"], {0, 2}, 40)
    e = C(210, 60); _segments(e, 14, 16, ["B", "I", "U", "S"], {0, 1, 2}, 40); e.pointer(76, 30)
    f = C(250, 60); _segments(f, 14, 16, ["Day", "Week", "Month"], {0}, 70, disabled={2})
    return [("1. One mode: Week", a, None), ("2. Month clicked", b, "SelectionChanged(2)"),
            ("3. Many mode: B and U on", d, None), ("4. I toggled on", e, "Toggled(1, on)"), ("Segment disabled", f, None)]


def _stepper(c, x, y, up=False, down=False, up_dis=False, down_dis=False):
    c.glyph(x, y, 14, "up", up, up_dis); c.glyph(x, y + 13, 14, "down", down, down_dis)


@xw("2.3")
def stepper():
    out = []
    for lab, v, kw, note in [("1. Idle", 4, {}, None), ("2. Up held: repeats", 7, dict(up=True), "ValueChanged (repeating)"),
                             ("3. At max 10", 10, dict(up_dis=True), "up arrow disabled"), ("Buddy: page number", None, {}, None)]:
        c = C(190, 56)
        if v is None:
            c.text(12, 16, "Page", BLACK, 13); c.rect(54, 13, 40, 26, fill=WHITE); c.ctext(54, 13, 40, 26, "3", BLACK, 13)
            c.text(100, 16, "of 12", BLACK, 13); _stepper(c, 140, 13)
        else:
            c.text(12, 16, "Level:", BLACK, 13); c.rect(64, 13, 40, 26, fill=WHITE); c.ctext(64, 13, 40, 26, str(v), BLACK, 13)
            _stepper(c, 108, 13, **kw)
        out.append((lab, c, note))
    return out


@xw("2.4")
def dial_sb():
    out = []
    for lab, f, kw, ptr, note in [("1. Idle", 0.3, {}, None, None), ("2. Dragging up", 0.65, {}, (120, 30), "ValueChanged (continuous)"),
                                  ("3. Released", 0.65, {}, None, "EditingFinished"),
                                  ("Focused (arrows step)", 0.65, dict(focused=True), None, None), ("Disabled", 0.3, dict(disabled=True), None, None)]:
        c = C(150, 130); _dial(c, 75, 58, 34, f, value=f"Volume {int(f * 100)}", **kw)
        if ptr:
            c.pointer(*ptr)
        out.append((lab, c, note))
    return out


def _level(c, x, y, w, frac, warn=0.7, crit=0.9, seg=0, text=None):
    c.rect(x, y, w, 18, fill=WHITE)
    if seg:
        sw = (w - 4) / seg
        for n in range(seg):
            sx = x + 2 + n * sw
            on = (n + 1) / seg <= frac + 1e-6
            c.rect(int(sx) + 1, y + 2, int(sw) - 2, 14, fill=BLACK if on else WHITE, outline=MID if not on else None)
            if on and (n + 1) / seg > warn:
                c.dither(int(sx) + 1, y + 2, int(sw) - 2, 14, WHITE)
        return
    fw = int((w - 4) * frac)
    c.rect(x + 2, y + 2, fw, 14, fill=BLACK, outline=None)
    wx = x + 2 + int((w - 4) * warn)
    if fw > wx - x - 2:
        for hx in range(wx, x + 2 + fw, 4):
            c.line(hx, y + 2, hx, y + 15, WHITE)
    cx = x + 2 + int((w - 4) * crit)
    if fw > cx - x - 2:
        c.dither(cx, y + 2, x + 2 + fw - cx, 14, WHITE)
    for t in (warn, crit):
        tx = x + 2 + int((w - 4) * t); c.line(tx, y + 18, tx, y + 23, DARK)
    if text:
        c.text(x, y + 24, text, DARK, 11)


@xw("2.5")
def level():
    out = []
    for lab, kw in [("Normal: 45%", dict(frac=0.45, text="45 GB of 100 GB")), ("Past warning: hatched", dict(frac=0.8, text="80 GB of 100 GB")),
                    ("Past critical: dithered", dict(frac=0.96, text="96 GB of 100 GB")), ("Discrete: 4 of 5", dict(frac=0.8, seg=5))]:
        c = C(230, 64); _level(c, 14, 12, 200, **kw); out.append((lab, c, None))
    c = C(230, 64); _level(c, 14, 12, 200, 0.6, seg=5); c.pointer(150, 20); out.append(("Editable: click sets 3 of 5", c, "ValueChanged"))
    return out


TOOLS = ["arrow", "pencil", "line", "rect", "oval", "fill", "text", "label"]
WIDGETS = ["label", "button", "check", "field", "list", "rect", "text", "line"]


def _palette(c, x, y, kinds, cols, sel=None, s=30, header=None):
    if header:
        c.rect(x, y, cols * s + 1, 20, fill=GRAY); c.text(x + 6, y, header, BLACK, 12); y += 20
    for n, k in enumerate(kinds):
        cx, cy = x + (n % cols) * s, y + (n // cols) * s
        on = n == sel
        c.rect(cx, cy, s + 1, s + 1, fill=BLACK if on else WHITE)
        tool_icon(c, cx, cy, s, k, on)
    return y + ((len(kinds) + cols - 1) // cols) * s


@xw("2.6")
def tool_palette():
    a = C(130, 150); _palette(a, 18, 14, TOOLS, 2, sel=0)
    b = C(130, 150); _palette(b, 18, 14, TOOLS, 2, sel=3); b.pointer(70, 64)
    d = C(170, 150); _palette(d, 18, 14, TOOLS, 2, sel=1); d.tooltip(60, 50, "Pencil")
    e = C(220, 150); _palette(e, 12, 12, TOOLS, 4, sel=None, header="Shapes")
    f = C(330, 150)
    f.rect(12, 12, 140, 20, fill=GRAY); f.text(18, 12, "Basic", BLACK, 12)
    for n, (k, t) in enumerate([("label", "Label"), ("button", "Button"), ("check", "Check Box"), ("field", "Text Field")]):
        ry = 32 + n * 26; f.rect(12, ry, 140, 27, fill=BLACK if n == 1 else WHITE)
        tool_icon(f, 14, ry, 26, k, n == 1); f.text(44, ry + 3, t, WHITE if n == 1 else BLACK, 13)
    f.dotted(190, 70, 110, 40); f.text(200, 80, "Button1", MID, 12); f.pointer(220, 92)
    return [("1. Select mode: arrow tool", a, None), ("2. Rectangle clicked", b, "SelectionChanged(3)"), ("3. Name as a tooltip", d, None),
            ("Reflows wider, with a group header", e, None), ("List layout, Drag mode (Chassis widget palette)", f, "DragStarted: outline preview")]


@xw("2.7")
def ruler():
    c = C(460, 150)
    c.rect(40, 12, 400, 22, fill=WHITE)
    for n in range(0, 41):
        tx = 40 + n * 10
        h = 10 if n % 10 == 0 else (6 if n % 5 == 0 else 3)
        c.line(tx, 34 - h, tx, 33, BLACK)
        if n % 10 == 0 and n < 40:
            c.text(tx + 3, 12, str(n // 10), DARK, 11)
    c.rect(12, 34, 28, 104, fill=WHITE)
    for n in range(0, 11):
        ty = 34 + n * 10
        c.line(40 - (10 if n % 10 == 0 else (6 if n % 5 == 0 else 3)), ty, 39, ty)
    c.rect(40, 34, 400, 104, fill=WHITE)
    c.line(230, 12, 230, 34, BLACK); c.pointer(230, 80)
    c.tri(96, 26, 8, "down"); c.tri(380, 26, 8, "down")
    d = C(460, 70); d.rect(40, 12, 400, 22, fill=WHITE)
    for n in range(0, 41):
        tx = 40 + n * 10
        d.line(tx, 33 - (10 if n % 10 == 0 else 3), tx, 33)
    d.tri(150, 26, 8, "down"); d.line(150, 34, 150, 60, MID); d.pointer(152, 30)
    return [("Ruler: units, pointer-tracking line, two markers", c, None), ("Dragging the left marker", d, "MarkerMoved(0, 1.1 in)")]


@xw("2.8")
def shortcut_field():
    out = []
    for lab, text, kw, note in [("1. Idle", "Ctrl+S", {}, None), ("2. Clicked: recording", "Press keys…", dict(focused=True), None),
                                ("3. Ctrl+Shift held", "Ctrl+Shift+…", dict(focused=True), None), ("4. S pressed", "Ctrl+Shift+S", {}, "ShortcutChanged"),
                                ("Conflict", "Ctrl+O", dict(focused=True), "rejected")]:
        c = C(240, 70); c.text(12, 14, "Save:", BLACK, 13); c.field(60, 12, 160, text, focused=kw.get("focused", False))
        if text == "Press keys…":
            c.field(60, 12, 160, "", placeholder="Press keys…", focused=True)
        if lab == "Conflict":
            c.text(60, 40, "Used by Open", DARK, 12)
        out.append((lab, c, note))
    return out


# ============================================================================
# Epic 3: Pickers and special fields
# ============================================================================
@xw("3.1")
def calendar():
    a = C(220, 190); mini_calendar(a, 14, 12, sel=24, marks=(3, 18))
    b = C(220, 190); mini_calendar(b, 14, 12, sel=12, marks=(3, 18)); b.pointer(120, 104)
    d = C(220, 190); mini_calendar(d, 14, 12, sel=None, rng=(9, 13))
    e = C(220, 190); mini_calendar(e, 14, 12, sel=24, month="December 2026", today=0); e.text(14, 170, "Page Down: next month", DARK, 11)
    return [("1. Today outlined, 24 selected, dots mark days", a, None), ("2. Click the 12th", b, "SelectionChanged"),
            ("Range mode: 9 to 13", d, "SelectionChanged(range)"), ("Next month", e, "MonthChanged")]


@xw("3.2")
def date_picker():
    a = C(270, 230); a.text(12, 14, "Due:", BLACK, 13); a.field(52, 12, 120, "11/24/2026"); a.glyph(172, 12, 22, "down")
    b = C(270, 230); b.text(12, 14, "Due:", BLACK, 13); b.field(52, 12, 120, "11/24/2026"); b.glyph(172, 12, 22, "down", pressed=True)
    popover(b, 40, 44, 212, 172, anchor_x=183); mini_calendar(b, 52, 52, sel=24, cell=27)
    d = C(270, 230); d.text(12, 14, "Due:", BLACK, 13); d.field(52, 12, 120, "11/30/2026"); d.glyph(172, 12, 22, "down")
    e = C(270, 230); e.text(12, 14, "Due:", BLACK, 13); e.field(52, 12, 120, "11/30/2026", sel=(3, 5), focused=True); e.glyph(172, 12, 22, "down")
    e.text(52, 42, "Up/Down steps the day", DARK, 11)
    return [("1. Idle", a, None), ("2. Button: calendar in a popover", b, "PopoverOpened"), ("3. 30 chosen: popover closes", d, "ValueChanged"),
            ("Typing: the part under the caret steps", e, None)]


@xw("3.3")
def time_field():
    a = C(220, 60); a.text(12, 16, "Start:", BLACK, 13); a.field(64, 14, 100, "09:30 AM"); _stepper(a, 166, 13)
    b = C(220, 60); b.text(12, 16, "Start:", BLACK, 13); b.field(64, 14, 100, "09:30 AM", sel=(3, 5), focused=True); _stepper(b, 166, 13)
    d = C(220, 60); d.text(12, 16, "Start:", BLACK, 13); d.field(64, 14, 100, "09:45 AM", sel=(3, 5), focused=True); _stepper(d, 166, 13, up=True)
    e = C(340, 60); e.text(12, 16, "When:", BLACK, 13); e.field(64, 14, 100, "11/24/2026"); e.glyph(164, 14, 22, "down")
    e.field(196, 14, 80, "09:45 AM"); _stepper(e, 278, 13)
    return [("1. Idle", a, None), ("2. Minutes clicked: part selected", b, None), ("3. Up arrow: +15 (step)", d, "ValueChanged"),
            ("Date-time field: one value", e, None)]


COLORS = [(0, 0, 0), (90, 90, 90), (150, 150, 150), (255, 255, 255), (170, 40, 40), (200, 120, 40), (210, 190, 60), (70, 120, 70),
          (60, 140, 160), (50, 80, 160), (110, 70, 150), (160, 70, 120)]


@xw("3.4")
def color_picker():
    a = C(150, 60); a.text(12, 16, "Fill:", BLACK, 13); a.rect(54, 12, 46, 28, fill=WHITE); a.rect(58, 16, 38, 20, fill=COLORS[7], outline=None)
    b = C(330, 290); b.text(12, 16, "Fill:", BLACK, 13); b.rect(54, 12, 46, 28, fill=BLACK); b.rect(58, 16, 38, 20, fill=COLORS[7], outline=None)
    popover(b, 20, 50, 296, 228, anchor_x=77)
    for n, col in enumerate(COLORS):
        swatch(b, 34 + (n % 6) * 30, 62 + (n // 6) * 30, 22, col, selected=(n == 9))
    b.text(220, 62, "Recent", DARK, 11); swatch(b, 220, 80, 18, COLORS[4]); swatch(b, 244, 80, 18, COLORS[9])
    for n, (ch, v) in enumerate([("R", 50), ("G", 80), ("B", 160)]):
        b.text(34, 130 + n * 30, ch, BLACK, 13); b.slider(54, 130 + n * 30, 150, v / 255); b.field(240, 128 + n * 30, 50, str(v))
    b.text(34, 222, "Hex", BLACK, 13); b.field(70, 220, 90, "#3250A0"); b.rect(180, 220, 50, 24, fill=COLORS[7]); b.rect(229, 220, 50, 24, fill=COLORS[9])
    b.text(180, 248, "before", DARK, 11); b.text(229, 248, "after", DARK, 11)
    d = C(150, 60); d.text(12, 16, "Fill:", BLACK, 13); d.rect(54, 12, 46, 28, fill=WHITE); d.rect(58, 16, 38, 20, fill=COLORS[9], outline=None)
    return [("1. Color well", a, None), ("2. Clicked: picker in a popover, blue chosen", b, "ColorChanged (live)"), ("3. Closed", d, None)]


@xw("3.5")
def font_chooser():
    a = C(220, 60); a.text(12, 16, "Font:", BLACK, 13); a.rect(56, 12, 150, 28, fill=WHITE); a.text(64, 15, "Sans 11", BLACK, 13)
    b = C(470, 290); b.text(12, 16, "Font:", BLACK, 13); b.rect(56, 12, 150, 28, fill=BLACK); b.text(64, 15, "Sans 11", WHITE, 13)
    popover(b, 20, 50, 436, 228, anchor_x=130)
    b.field(34, 62, 200, "", placeholder="Filter families")
    b.listbox(34, 92, 200, ["Mono", "Sans", "Serif", "Slab"], selected={2}, scroll=True)
    b.text(250, 62, "Style", DARK, 11); b.listbox(250, 78, 110, ["Regular", "Italic", "Bold"], selected={0}, scroll=False)
    b.text(372, 62, "Size", DARK, 11); b.field(372, 78, 70, "14"); b.listbox(372, 104, 70, ["10", "11", "12", "14"], selected={3}, scroll=False)
    b.rect(34, 190, 408, 72, fill=WHITE); b.text(44, 206, "The quick brown fox", BLACK, 20)
    d = C(220, 60); d.text(12, 16, "Font:", BLACK, 13); d.rect(56, 12, 150, 28, fill=WHITE); d.text(64, 15, "Serif 14", BLACK, 13)
    return [("1. Font button", a, None), ("2. Chooser: family, style, size, preview", b, "FontChanged"), ("3. Serif 14 chosen", d, None)]


@xw("3.6")
def path_field():
    a = C(330, 60); a.text(12, 16, "Folder:", BLACK, 13); a.field(70, 14, 210, "/home/user/projects"); a.button(282, 14, 30, 22, "…")
    b = C(330, 60); b.text(12, 16, "Folder:", BLACK, 13); b.field(70, 14, 210, "/home/user/projects"); b.button(282, 14, 30, 22, "…", pressed=True)
    b.text(70, 40, "opens the file chooser here", DARK, 11)
    d = C(330, 60); d.text(12, 16, "Folder:", BLACK, 13); d.field(70, 14, 210, "/home/user/notes"); d.button(282, 14, 30, 22, "…")
    d.dotted(68, 12, 214, 26); d.pointer(200, 30)
    e = C(330, 60); e.text(12, 16, "Folder:", BLACK, 13); e.field(70, 14, 210, "/home/user/nope"); e.button(282, 14, 30, 22, "…")
    e.text(70, 40, "Folder not found", DARK, 11)
    return [("1. Idle", a, None), ("2. Browse", b, "PathChanged when chosen"), ("3. Folder dropped on the field", d, "PathChanged"),
            ("Missing path (must_exist)", e, None)]


@xw("3.7")
def image_well():
    a = C(150, 150); a.rect(20, 12, 110, 110, fill=WHITE); a.ctext(20, 12, 110, 110, "No image", MID, 12)
    b = C(150, 150); b.rect(20, 12, 110, 110, fill=WHITE); b.dotted(16, 8, 118, 118); b.pointer(80, 70)
    d = C(150, 150); d.rect(20, 12, 110, 110, fill=WHITE); _picture(d, 22, 14, 106, 106)
    e = C(150, 150); e.rect(20, 12, 110, 110, fill=WHITE); _picture(e, 22, 14, 106, 106); e.rect(36, 28, 70, 70, fill=None, width=2)
    return [("1. Empty", a, None), ("2. Image file dragged over", b, None), ("3. Dropped", d, "ImageChanged"), ("Optional square crop", e, None)]


@xw("3.8")
def new_password():
    out = []
    for lab, p1, p2, frac, msg, ok in [("1. Too short", "••••", "", 0.2, "At least 10 characters", False),
                                        ("2. Strong, not confirmed", "••••••••••••", "••••", 0.85, "Passwords don't match", False),
                                        ("3. Matches: OK enabled", "••••••••••••", "••••••••••••", 0.85, "", True)]:
        c = C(360, 150)
        c.text(12, 14, "Password:", BLACK, 13); c.field(100, 12, 180, p1); c.checkbox(286, 12, "Show")
        c.text(12, 46, "Confirm:", BLACK, 13); c.field(100, 44, 180, p2)
        _level(c, 100, 76, 180, frac, warn=0.99, crit=1.0); c.text(100, 98, msg, DARK, 11)
        c.button(230, 118, 80, 24, "OK", default=ok, disabled=not ok)
        out.append((lab, c, "PasswordChanged(acceptable)" if ok else None))
    return out


@xw("3.9")
def character_picker():
    c = C(480, 250)
    _choice(c, 12, 12, 150, "Arrows"); c.field(172, 12, 170, "", placeholder="Search names")
    chars = [(180, 0, 0), (90, 0, 0), (0, 0, 0), (270, 0, 0), (0, 0, 1), (90, 0, 1), (135, 0, 0), (45, 0, 0),
             (315, 0, 0), (225, 0, 0), (180, 1, 0), (90, 1, 0), (0, 1, 0), (270, 1, 0), (0, 1, 1), (90, 1, 1)]
    for n, (ang, dbl, both) in enumerate(chars):
        x, y = 12 + (n % 8) * 34, 48 + (n // 8) * 34
        on = n == 2
        c.rect(x, y, 35, 35, fill=BLACK if on else WHITE); arrow(c, x + 17, y + 17, ang, WHITE if on else BLACK, bool(dbl), bool(both), 9)
    c.rect(300, 48, 166, 140, fill=WHITE); arrow(c, 383, 86, 0, BLACK, size=26)
    c.ctext(300, 124, 166, 20, "RIGHTWARDS ARROW", DARK, 11); c.ctext(300, 146, 166, 20, "U+2192", DARK, 11)
    c.text(12, 130, "Recent:", DARK, 11)
    for n, ch in enumerate("—…•"):
        c.rect(70 + n * 30, 124, 28, 28, fill=WHITE); c.ctext(70 + n * 30, 124, 28, 28, ch, BLACK, 14)
    c.button(386, 210, 80, 26, "Insert", default=True)
    return [("Block, search, grid, detail, recent; double-click or Insert", c, "CharacterChosen(U+2192)")]


# ============================================================================
# Epic 4: Data views
# ============================================================================
TREE_OPEN = [(0, "Documents", "open"), (1, "Letters", "closed"), (1, "Reports", "open"), (2, "q3.txt", "leaf"), (2, "q4.txt", "leaf"), (0, "Pictures", "closed")]


@xw("4.1")
def tree_view():
    a = C(220, 150); tree_rows(a, 12, 12, 196, [(0, "Documents", "closed"), (0, "Pictures", "closed")])
    b = C(220, 150); tree_rows(b, 12, 12, 196, [(0, "Documents", "open"), (1, "Letters", "closed"), (1, "Reports", "closed"), (0, "Pictures", "closed")]); b.pointer(22, 22)
    d = C(220, 150); tree_rows(d, 12, 12, 196, TREE_OPEN, sel=3)
    e = C(220, 150); tree_rows(e, 12, 12, 196, TREE_OPEN, sel=4); e.line(32, 34, 200, 34, BLACK, 2); e.pointer(120, 30)
    return [("1. Collapsed", a, None), ("2. Triangle clicked", b, "Expanded(Documents)"), ("3. Expanded, q3.txt selected", d, "SelectionChanged"),
            ("Drag to reparent: insertion line", e, None)]


@xw("4.2")
def column_browser():
    out = []
    for lab, path, note in [("1. Root column", [], None), ("2. 'Users' selected: next column appears", ["Users"], "SelectionChanged"),
                            ("3. 'user' selected", ["Users", "user"], "SelectionChanged")]:
        c = C(450, 130); _browser(c, 14, 14, path); out.append((lab, c, note))
    c = C(450, 150); _browser(c, 14, 14, ["Users", "user", "Projects", "pushrod", "Cargo.toml"], offset=2)
    c.scrollbar(14, 122, 422, False, 1.0, 0.6)
    out.append(("Deep path: scrolled to the newest column; leaf activated", c, "Activated"))
    return out


def _grid(c, x, y, cols, rows, sel=(), names=None):
    names = names or ["Report", "Notes", "Budget", "Photo", "Draft", "Plan", "Ideas", "Todo"]
    n = 0
    for r in range(rows):
        for col in range(cols):
            if n >= len(names):
                return
            _icon(c, x + col * 80, y + r * 66, names[n], n in sel); n += 1


@xw("4.3")
def icon_grid():
    a = C(350, 160); a.rect(10, 8, 330, 144, fill=WHITE); _grid(a, 14, 14, 4, 2)
    b = C(350, 160); b.rect(10, 8, 330, 144, fill=WHITE); _grid(b, 14, 14, 4, 2, sel={1, 2, 5, 6}); b.dotted(100, 16, 160, 120); b.pointer(258, 134)
    d = C(350, 160); d.rect(10, 8, 330, 144, fill=WHITE); _grid(d, 14, 14, 4, 2, sel={6})
    d.field(88 + 80 - 4, 118, 64, "Ideas 2", caret=7)
    return [("1. Grid", a, None), ("2. Rubber band selects four", b, "SelectionChanged"), ("3. Rename in place", d, "Renamed")]


def _grid_table(c, x, y, rows, edit=None, sel_row=None, bad=None):
    cols = [("Item", 140), ("Qty", 60), ("Price", 80), ("Paid", 50)]
    c.rect(x, y, 330, 22, fill=WHITE); cx = x
    for t, w in cols:
        c.text(cx + 6, y + 2, t, BLACK, 12); c.line(cx + w, y, cx + w, y + 22 + 22 * (len(rows) + 1)); cx += w
    c.rect(x, y + 22, 330, 22 * (len(rows) + 1), fill=WHITE)
    for r, row in enumerate(rows):
        ry = y + 22 + r * 22; cx = x
        for ci, (val, (t, w)) in enumerate(zip(row, cols)):
            if (r, ci) == edit:
                c.field(cx + 1, ry + 1, w - 2, val, caret=len(val), h=20)
            elif ci == 3:
                c.checkbox(cx + 16, ry, "", "on" if val else "off")
            else:
                c.text(cx + 6, ry + 2, val, BLACK, 12)
            if (r, ci) == bad:
                c.rect(cx + 1, ry + 1, w - 2, 20, fill=None, width=2)
            cx += w
        c.line(x, ry + 22, x + 330, ry + 22, GRAY)
    c.text(x + 6, y + 24 + 22 * len(rows), "+ new row", MID, 12)


ROWS = [("Paper", "4", "12.00", True), ("Pens", "10", "8.50", False), ("Stapler", "1", "15.25", True)]


@xw("4.4")
def data_grid():
    a = C(360, 130); _grid_table(a, 14, 12, ROWS)
    b = C(360, 130); _grid_table(b, 14, 12, ROWS, edit=(1, 1))
    d = C(360, 130); _grid_table(d, 14, 12, [ROWS[0], ("Pens", "12", "8.50", False), ROWS[2]]); d.rect(214, 56, 80, 22, fill=None, width=2)
    e = C(360, 150); _grid_table(e, 14, 12, [ROWS[0], ("Pens", "-3", "8.50", False), ROWS[2]], bad=(1, 1)); e.text(14, 126, "Qty must be positive", DARK, 11)
    return [("1. Grid", a, None), ("2. F2 on Qty: editing", b, None), ("3. Return: committed, moves right with Tab", d, "CellEdited"),
            ("Validation rejects -3", e, None)]


def _prop_grid(c, x, y, w, rows, sel=None, div=110):
    h = 20 * len(rows) + 2
    c.rect(x, y, w, h, fill=WHITE)
    for n, (kind, name, val) in enumerate(rows):
        ry = y + 1 + n * 20
        if kind == "cat":
            c.rect(x + 1, ry, w - 2, 20, fill=GRAY, outline=None); c.tri(x + 6, ry + 7, 8, "down"); c.text(x + 20, ry + 1, name, BLACK, 12)
            continue
        if n == sel:
            c.rect(x + 1, ry, div - 1, 20, fill=BLACK, outline=None)
        c.text(x + 18, ry + 1, name, WHITE if n == sel else BLACK, 12)
        c.line(x + div, ry, x + div, ry + 20, GRAY)
        if kind == "bool":
            c.checkbox(x + div + 6, ry - 1, "", "on" if val else "off")
        elif kind == "choice":
            c.text(x + div + 6, ry + 1, val, BLACK, 12); c.tri(x + w - 16, ry + 7, 8, "down")
        elif kind == "color":
            c.rect(x + div + 6, ry + 4, 14, 12, fill=val); c.text(x + div + 26, ry + 1, "#3250A0", BLACK, 12)
        elif kind == "edit":
            c.field(x + div + 2, ry + 1, w - div - 4, val, caret=len(val), h=18)
        else:
            c.text(x + div + 6, ry + 1, val, BLACK, 12, ) if not str(name).startswith("*") else None
        if kind == "bold":
            c.text(x + div + 6, ry + 1, val, BLACK, 12); c.text(x + div + 6.5, ry + 1, val, BLACK, 12)
        c.line(x, ry + 20, x + w, ry + 20, (225, 225, 225))


PROPS = [("cat", "Identity", ""), ("text", "Name", "SaveButton"), ("cat", "Appearance", ""), ("bold", "Text", "Save"),
         ("color", "Background", (50, 80, 160)), ("choice", "Align", "Center"), ("cat", "Behavior", ""), ("bool", "Default", True), ("bool", "Disabled", False)]


@xw("4.5")
def property_grid():
    a = C(290, 210); _prop_grid(a, 12, 12, 266, PROPS)
    rows = list(PROPS); rows[3] = ("edit", "Text", "Save all")
    b = C(290, 210); _prop_grid(b, 12, 12, 266, rows, sel=3)
    rows2 = list(PROPS); rows2[3] = ("bold", "Text", "Save all")
    d = C(290, 210); _prop_grid(d, 12, 12, 266, rows2, sel=3)
    e = C(290, 210); _prop_grid(e, 12, 12, 266, PROPS, div=150); e.line(162, 12, 162, 194, BLACK, 2); e.pointer(162, 100)
    return [("1. Categories; bold = changed from default", a, None), ("2. Text clicked: editor in place", b, None),
            ("3. Return commits", d, "PropertyChanged(Text, old, new)"), ("Divider dragged", e, None)]


@xw("4.6")
def check_list():
    def draw(c, checks, sel=(), header="mixed"):
        c.rect(12, 12, 220, 22, fill=GRAY); c.checkbox(18, 12, "Include all", header)
        c.rect(12, 34, 220, 102, fill=WHITE)
        for n, (t, on) in enumerate(zip(["Documents", "Pictures", "Music", "Videos", "Downloads"], checks)):
            ry = 35 + n * 20
            if n in sel:
                c.rect(13, ry, 218, 20, fill=BLACK, outline=None)
            c.checkbox(18, ry - 1, "", "on" if on else "off"); c.text(40, ry, t, WHITE if n in sel else BLACK, 13)
    a = C(250, 150); draw(a, [True, False, True, False, False])
    b = C(250, 150); draw(b, [True, True, True, False, False]); b.pointer(24, 64)
    d = C(250, 150); draw(d, [True, True, True, True, True], sel={3, 4}, header="on")
    return [("1. Mixed: some checked", a, None), ("2. Box clicked", b, "CheckChanged(1, true)"),
            ("3. Space on two selected rows: all checked", d, "CheckChanged")]


def _edit_list(c, items, sel=None, edit=None):
    c.listbox(12, 12, 220, items, selected={sel} if sel is not None else set(), scroll=False)
    if edit is not None:
        c.field(13, 13 + edit * 20, 218, items[edit], caret=len(items[edit]), h=20)
    by = 14 + 20 * len(items)
    for n, g in enumerate(["+", "-"]):
        c.rect(12 + n * 25, by, 26, 22, fill=WHITE); c.ctext(12 + n * 25, by, 26, 22, g, BLACK, 14)
        if g == "-" and sel is None:
            c.dither(12 + n * 25, by, 26, 22, PANEL)
    c.glyph(180, by, 22, "up", disabled=sel in (None, 0)); c.glyph(203, by, 22, "down", disabled=sel in (None, len(items) - 1))


@xw("4.7")
def editable_list():
    a = C(250, 130); _edit_list(a, ["/usr/share/fonts", "/home/user/fonts"])
    b = C(250, 130); _edit_list(b, ["/usr/share/fonts", "/home/user/fonts", "/opt/fonts"], sel=2, edit=2)
    d = C(250, 130); _edit_list(d, ["/usr/share/fonts", "/opt/fonts", "/home/user/fonts"], sel=1)
    return [("1. Two items; - disabled with no selection", a, None), ("2. + adds a row and edits it", b, "ItemsChanged"),
            ("3. Up moves it", d, "ItemsChanged")]


@xw("4.8")
def dual_list():
    def draw(c, avail, chosen, sa=(), sc=()):
        c.text(12, 8, "Available", DARK, 12); c.listbox(12, 26, 150, avail, selected=set(sa), scroll=False)
        for n, g in enumerate([">", ">>", "<", "<<"]):
            c.button(176, 30 + n * 28, 40, 22, g, disabled=(g == ">" and not sa) or (g == "<" and not sc))
        c.text(230, 8, "Shown columns", DARK, 12); c.listbox(230, 26, 150, chosen, selected=set(sc), scroll=False)
        c.glyph(386, 30, 20, "up", disabled=not sc); c.glyph(386, 52, 20, "down", disabled=not sc)
    a = C(420, 150); draw(a, ["Size", "Type", "Owner"], ["Name", "Modified"], sa={1})
    b = C(420, 150); draw(b, ["Size", "Owner"], ["Name", "Modified", "Type"], sc={2})
    d = C(420, 150); draw(d, ["Size", "Owner"], ["Name", "Type", "Modified"], sc={1})
    return [("1. Type selected", a, None), ("2. > moves it across", b, "ChosenChanged"), ("3. Up reorders", d, "ChosenChanged")]


def _crumbs(c, x, y, parts, hot=None):
    cx = x
    for n, t in enumerate(parts):
        last = n == len(parts) - 1
        w = int(c.tw(t, 13)) + 12
        if n == hot:
            c.rect(cx, y, w, 24, fill=BLACK, outline=None)
        c.text(cx + 6, y + 3, t, WHITE if n == hot else (BLACK if last else (50, 80, 160)), 13)
        cx += w
        if not last:
            c.tri(cx + 2, y + 8, 8, "right", DARK); cx += 14
    return cx


@xw("4.9")
def breadcrumb():
    a = C(430, 150); _crumbs(a, 12, 12, ["ContactDialog", "Column", "Row", "Spacer1"])
    b = C(430, 150); _crumbs(b, 12, 12, ["ContactDialog", "Column", "Row", "Spacer1"], hot=1); b.pointer(150, 24)
    d = C(430, 150); x = _crumbs(d, 12, 12, ["ContactDialog", "Column", "Row", "Spacer1"])
    d.menu(170, 38, 140, [("Row", "", ""), ("Buttons", "", ""), ("Footer", "", "")], hover=1)
    e = C(430, 150); e.rect(12, 12, 24, 24, fill=WHITE); e.ctext(12, 12, 24, 24, "…", BLACK, 13); e.tri(42, 20, 8, "right", DARK)
    _crumbs(e, 56, 12, ["Row", "Spacer1"])
    return [("1. Path to the selection", a, None), ("2. Column clicked", b, "SegmentActivated(1)"),
            ("3. Separator: sibling menu", d, "SiblingChosen"), ("Narrow: leading parts in a … menu", e, None)]


@xw("4.10")
def filter_field():
    def field(c, text):
        c.field(12, 12, 220, "")
        if text:
            c.text(34, 13, text); cx = 34 + c.tw(text); c.line(cx, 16, cx, 30)
        c.ellipse(16, 17, 10, 10); c.line(24, 25, 28, 29)
        if text:
            c.ctext(212, 12, 20, 22, "×", DARK, 14)
    a = C(250, 160); field(a, ""); a.text(34, 13, "Filter", MID)
    tree_rows(a, 12, 42, 220, TREE_OPEN[:5])
    b = C(250, 160); field(b, "q4"); tree_rows(b, 12, 42, 220, [(0, "Documents", "open"), (1, "Reports", "open"), (2, "q4.txt", "leaf")])
    d = C(250, 160); field(d, "zz"); d.rect(12, 42, 220, 60, fill=WHITE); d.ctext(12, 42, 220, 60, "No matches", MID, 12)
    return [("1. Bound to a tree", a, None), ("2. 'q4': matches keep their parents", b, "FilterChanged(1 match)"), ("3. Nothing matches", d, None)]


@xw("4.11")
def rule_editor():
    def rows(c, rules):
        c.text(12, 14, "Match", BLACK, 13); _choice(c, 62, 12, 70, "All"); c.text(140, 14, "of the following:", BLACK, 13)
        for n, (f, op, v) in enumerate(rules):
            y = 46 + n * 32
            _choice(c, 12, y, 110, f); _choice(c, 128, y, 120, op)
            if v is not None:
                c.field(254, y + 1, 120, v)
            for k, g in enumerate("-+"):
                c.rect(382 + k * 25, y, 26, 24, fill=WHITE); c.ctext(382 + k * 25, y, 26, 24, g, BLACK, 14)
    a = C(450, 120); rows(a, [("Subject", "contains", "invoice")])
    b = C(450, 120); rows(b, [("Subject", "contains", "invoice"), ("Received", "is before", "11/01/2026")])
    return [("1. One rule", a, None), ("2. + adds a row; a date field gets date operators", b, "RulesChanged")]


@xw("4.12")
def image_viewer():
    def view(c, scale, ox=0, oy=0, bar="Fit"):
        c.rect(12, 12, 300, 160, fill=WHITE)
        w, h = int(280 * scale), int(150 * scale)
        x0 = 12 + (300 - w) // 2 - ox
        y0 = 12 + (160 - h) // 2 - oy
        _picture(c, x0, y0, w, h)
        c.rect(0, 0, 12, c.h, fill=PANEL, outline=None); c.rect(312, 0, c.w - 312, c.h, fill=PANEL, outline=None)
        c.rect(0, 0, c.w, 12, fill=PANEL, outline=None); c.rect(0, 172, c.w, c.h - 172, fill=PANEL, outline=None)
        c.rect(12, 12, 300, 160, fill=None)
        c.rect(12, 172, 300, 24, fill=WHITE); _choice(c, 14, 172, 80, bar); c.text(110, 176, f"{int(scale * 100)}%", DARK, 12)
    a = C(330, 210); view(a, 1.0)
    b = C(330, 210); view(b, 2.0, bar="200%"); b.pointer(160, 90)
    d = C(330, 210); view(d, 2.0, ox=120, bar="200%"); d.pointer(160, 90)
    return [("1. Fit", a, None), ("2. Ctrl+wheel: zoom around the pointer", b, "ZoomChanged(2.0)"), ("3. Drag pans (hand cursor)", d, None)]


# ============================================================================
# Epic 5: Panels and navigation
# ============================================================================
@xw("5.1")
def drawer():
    def win(c, collapsed, width=150):
        c.rect(0, 0, c.w, c.h, fill=PANEL)
        if collapsed:
            c.rect(8, 8, 24, c.h - 16, fill=WHITE); c.glyph(10, 12, 18, "right")
            for n, ch in enumerate("Project"):
                c.ctext(8, 40 + n * 16, 24, 16, ch, BLACK, 12)
            lx = 40
        else:
            c.rect(8, 8, width, c.h - 16, fill=WHITE); c.rect(8, 8, width, 24, fill=GRAY); c.text(14, 10, "Project", BLACK, 13); c.glyph(width - 14, 11, 18, "left")
            c.listbox(14, 38, width - 12, ["Forms", "Strings", "Images"], scroll=False)
            c.line(width + 10, 8, width + 10, c.h - 8, DARK, 2); lx = width + 18
        c.rect(lx, 8, c.w - lx - 8, c.h - 16, fill=WHITE); c.ctext(lx, 8, c.w - lx - 8, c.h - 16, "Work area", MID, 13)
    a = C(380, 170); win(a, False)
    b = C(380, 170); win(b, True)
    d = C(380, 170); win(d, False, 200); d.pointer(210, 90)
    return [("1. Drawer open", a, None), ("2. Collapse glyph: thin strip", b, "Collapsed"), ("3. Expanded and widened", d, "Expanded; Resized")]


@xw("5.2")
def accordion():
    def draw(c, open_i, titles=("Shapes", "Text", "Colors", "Layers")):
        y = 10
        for n, t in enumerate(titles):
            c.rect(12, y, 220, 24, fill=WHITE if n != open_i else GRAY)
            c.tri(18, y + 8, 8, "down" if n == open_i else "right"); c.text(32, y + 2, t, BLACK, 13)
            y += 24
            if n == open_i:
                c.rect(12, y, 220, 80, fill=PANEL); _palette(c, 20, y + 8, TOOLS[:6], 6, s=30); y += 80
    a = C(250, 200); draw(a, 0)
    b = C(250, 200); draw(b, 2); b.pointer(80, 70)
    return [("1. Shapes open", a, None), ("2. Colors clicked: Shapes closes", b, "SectionChanged(2)")]


@xw("5.3")
def sidebar():
    def draw(c, sel):
        c.rect(8, 8, 150, c.h - 16, fill=WHITE)
        items = [("h", "Personal"), ("i", "Appearance"), ("i", "Notifications"), ("h", "System"), ("i", "Storage"), ("i", "Updates")]
        y = 10
        for n, (k, t) in enumerate(items):
            if k == "h":
                c.text(14, y + 2, t.upper(), DARK, 10); y += 20; continue
            if n == sel:
                c.rect(9, y, 148, 22, fill=BLACK, outline=None)
            col = WHITE if n == sel else BLACK
            c.rect(16, y + 5, 12, 12, outline=col); c.text(34, y + 1, t, col, 13)
            if t == "Updates":
                c.text(130, y + 2, "3", col, 11)
            y += 22
        c.rect(164, 8, c.w - 172, c.h - 16, fill=PANEL)
        title = items[sel][1]; c.text(176, 16, title, BLACK, 15)
        if title == "Appearance":
            c.radio(176, 48, "Light", on=True); c.radio(176, 72, "Dark")
        else:
            c.text(176, 48, "Used: 62 GB", BLACK, 13); _level(c, 176, 72, 160, 0.62)
    a = C(380, 170); draw(a, 1)
    b = C(380, 170); draw(b, 4); b.pointer(70, 100)
    return [("1. Appearance page", a, None), ("2. Storage chosen: page switches", b, "SelectionChanged")]


def _doc_tabs(c, x, y, tabs, active, hover=None, mod=(), overflow=False):
    cx = x
    for n, t in enumerate(tabs):
        w = int(c.tw(t, 13)) + 44
        c.rect(cx, y, w, 26, fill=PANEL if n == active else GRAY); c.text(cx + 10, y + 3, t, BLACK, 13)
        if n in mod:
            c.ellipse(cx + w - 22, y + 9, 8, 8, fill=BLACK, outline=None)
        elif n == active or n == hover:
            c.ctext(cx + w - 26, y, 20, 26, "×", BLACK, 14)
        cx += w - 1
    if overflow:
        c.glyph(c.w - 62, y + 4, 18, "left"); c.glyph(c.w - 44, y + 4, 18, "right"); c.glyph(c.w - 26, y + 4, 18, "down")
    c.rect(x, y + 25, c.w - 2 * x, 50, fill=PANEL)


@xw("5.4")
def document_tabs():
    a = C(420, 90); _doc_tabs(a, 10, 10, ["main.rs", "lib.rs", "notes.md"], 0, mod={1})
    b = C(420, 90); _doc_tabs(b, 10, 10, ["main.rs", "lib.rs", "notes.md"], 0, hover=2, mod={1}); b.pointer(290, 22)
    d = C(420, 90); _doc_tabs(d, 10, 10, ["lib.rs", "main.rs", "notes.md"], 1, mod={0}); d.line(10, 10, 10, 36, BLACK, 2); d.pointer(60, 22)
    e = C(420, 90); _doc_tabs(e, 10, 10, ["main.rs", "lib.rs", "notes.md", "plan.md"], 0, overflow=True)
    return [("1. Modified mark on lib.rs", a, None), ("2. Hover shows close", b, "CloseRequested(2) on click"),
            ("3. Dragged to the front", d, "TabMoved(0 to 1)"), ("Overflow: arrows and tab list", e, None)]


# ============================================================================
# Epic 6: Dialogs and assistants
# ============================================================================
@xw("6.1")
def input_dialog():
    a = C(340, 150); x, y = window(a, 0, 0, 340, 150, "New Folder")
    a.text(16, y + 14, "Name for the new folder:", BLACK, 13); a.field(16, y + 40, 306, "", caret=0, focused=True)
    a.button(146, y + 84, 80, 26, "Cancel"); a.button(240, y + 84, 80, 26, "OK", default=True, disabled=True)
    b = C(340, 150); x, y = window(b, 0, 0, 340, 150, "New Folder")
    b.text(16, y + 14, "Name for the new folder:", BLACK, 13); b.field(16, y + 40, 306, "Invoices", caret=8, focused=True)
    b.button(146, y + 84, 80, 26, "Cancel"); b.button(240, y + 84, 80, 26, "OK", default=True)
    d = C(340, 150); x, y = window(d, 0, 0, 340, 150, "Print")
    d.text(16, y + 14, "Copies:", BLACK, 13); d.field(80, y + 12, 60, "   2"); d.glyph(140, y + 11, 12, "up"); d.glyph(140, y + 23, 12, "down")
    d.button(146, y + 84, 80, 26, "Cancel"); d.button(240, y + 84, 80, 26, "OK", default=True)
    return [("1. Empty: OK disabled by validation", a, None), ("2. Typed: Return or OK", b, "completion(Some(\"Invoices\"))"),
            ("Number form", d, None)]


@xw("6.2")
def progress_dialog():
    a = C(360, 140); x, y = window(a, 0, 0, 360, 140, "Exporting")
    a.text(16, y + 14, "Writing page 12 of 40…", BLACK, 13); a.progress(16, y + 42, 326, frac=0.3)
    a.text(16, y + 66, "About 1 minute left", DARK, 11); a.button(262, y + 80, 80, 26, "Cancel")
    b = C(360, 140); x, y = window(b, 0, 0, 360, 140, "Exporting")
    b.text(16, y + 14, "Preparing…", BLACK, 13); b.progress(16, y + 42, 326, busy_at=0.4); b.button(262, y + 80, 80, 26, "Cancel")
    d = C(360, 140); x, y = window(d, 0, 0, 360, 140, "Exporting")
    d.text(16, y + 14, "Cancelling…", BLACK, 13); d.progress(16, y + 42, 326, frac=0.3); d.button(262, y + 80, 80, 26, "Cancel", pressed=True, disabled=True)
    return [("Shown only after 0.5 s; determinate", a, None), ("Unknown length: busy", b, None), ("Cancel pressed", d, "CancelRequested; task stops")]


def _cmd(c, x, y, w, title, desc, hot=False):
    c.rect(x, y, w, 46, fill=BLACK if hot else WHITE)
    c.tri(x + 10, y + 10, 10, "right", WHITE if hot else BLACK)
    c.text(x + 28, y + 4, title, WHITE if hot else BLACK, 14); c.text(x + 28, y + 24, desc, WHITE if hot else DARK, 11)


@xw("6.3")
def task_dialog():
    a = C(420, 260); x, y = window(a, 0, 0, 420, 260, "Sample App")
    a.ellipse(16, y + 14, 32, 32); a.ctext(16, y + 14, 32, 32, "!", BLACK, 18)
    a.text(60, y + 12, "Save changes to “notes.md”?", BLACK, 16); a.text(60, y + 38, "Your changes will be lost if you don't save them.", DARK, 12)
    _cmd(a, 60, y + 62, 340, "Save", "Write the changes to the file", hot=True); _cmd(a, 60, y + 112, 340, "Don't save", "Close and discard the changes")
    a.checkbox(16, y + 172, "Don't ask again"); a.button(320, y + 200, 80, 26, "Cancel")
    b = C(420, 260); x, y = window(b, 0, 0, 420, 260, "Sample App")
    b.ellipse(16, y + 14, 32, 32); b.ctext(16, y + 14, 32, 32, "×", BLACK, 18)
    b.text(60, y + 12, "The file couldn't be opened.", BLACK, 16)
    b.tri(60, y + 46, 9, "down"); b.text(76, y + 40, "Details", BLACK, 13)
    b.rect(60, y + 64, 340, 90, fill=WHITE); b.text(66, y + 68, "Error: permission denied", BLACK, 12); b.text(66, y + 86, "path: /home/user/notes.md", BLACK, 12)
    b.button(60, y + 162, 70, 24, "Copy"); b.button(320, y + 200, 80, 26, "OK", default=True)
    return [("Command choices with descriptions, don't-ask-again", a, "completion(Save, dont_ask=false)"), ("Expandable, copyable details", b, None)]


@xw("6.4")
def wizard():
    def draw(c, step, title, body, can_next=True, last=False):
        x, y = window(c, 0, 0, c.w, c.h, "Import Contacts")
        c.rect(0, y, 150, c.h - y, fill=WHITE)
        for n, s in enumerate(["Choose file", "Map fields", "Review", "Finish"]):
            sy = y + 14 + n * 26
            if n < step:
                checkmark(c, 14, sy + 4)
            c.text(32, sy, s, BLACK if n <= step else MID, 13)
            if n == step:
                c.text(32.6, sy, s, BLACK, 13)
        c.text(166, y + 12, title, BLACK, 15); body(c, 166, y + 44)
        c.button(c.w - 290, c.h - 40, 80, 26, "Back", disabled=step == 0)
        c.button(c.w - 200, c.h - 40, 90, 26, "Finish" if last else "Next", default=True, disabled=not can_next)
        c.button(c.w - 100, c.h - 40, 80, 26, "Cancel")
    a = C(500, 230); draw(a, 0, "Choose a file", lambda c, x, y: (c.field(x, y, 220, "", placeholder="contacts.csv"), c.button(x + 224, y, 30, 22, "…")), can_next=False)
    b = C(500, 230); draw(b, 1, "Map fields", lambda c, x, y: [(c.text(x, y + n * 30, a_, BLACK, 13), _choice(c, x + 90, y - 2 + n * 30, 150, b_)) for n, (a_, b_) in enumerate([("Name", "Full name"), ("E-mail", "Email")])])
    d = C(500, 230); draw(d, 3, "Ready to import", lambda c, x, y: c.text(x, y, "12 contacts from contacts.csv", BLACK, 13), last=True)
    return [("1. Next disabled until a file is chosen", a, None), ("2. Map fields", b, "PageChanged(1)"), ("3. Last page: Finish", d, "completion(finished)")]


@xw("6.5")
def settings_dialog():
    c = C(520, 260); x, y = window(c, 0, 0, 520, 260, "Settings")
    c.field(8, y + 8, 150, "", placeholder="Search settings")
    items = ["General", "Editor", "Fonts", "Shortcuts"]
    c.listbox(8, y + 36, 150, items, selected={1}, scroll=False)
    c.text(172, y + 10, "Editor", BLACK, 15)
    c.text(172, y + 46, "Tab width:", BLACK, 13); c.field(270, y + 44, 50, "4")
    c.checkbox(172, y + 76, "Show line numbers", "on"); c.checkbox(172, y + 102, "Wrap long lines")
    c.button(172, y + 140, 140, 24, "Restore Defaults")
    c.button(242, 222, 80, 26, "Apply"); c.button(330, 222, 80, 26, "Cancel"); c.button(418, 222, 80, 26, "OK", default=True)
    return [("Sidebar of categories; Apply enabled after a change", c, "Applied on OK or Apply")]


@xw("6.6")
def about_dialog():
    a = C(320, 270); x, y = window(a, 0, 0, 320, 270, "About Sample App")
    a.rect(136, y + 14, 48, 48, fill=WHITE); a.rect(146, y + 24, 28, 28, fill=GRAY)
    a.ctext(0, y + 70, 320, 24, "Sample App", BLACK, 17); a.ctext(0, y + 96, 320, 20, "Version 1.2.0", DARK, 12)
    a.ctext(0, y + 116, 320, 20, "© 2026 Example Corp.", DARK, 12); a.ctext(0, y + 138, 320, 20, "www.example.com", (50, 80, 160), 12)
    a.button(20, y + 200, 80, 26, "Credits"); a.button(108, y + 200, 80, 26, "License"); a.button(220, y + 200, 80, 26, "OK", default=True)
    b = C(320, 270); x, y = window(b, 0, 0, 320, 270, "About Sample App")
    b.text(16, y + 10, "Credits", BLACK, 15); b.rect(16, y + 38, 288, 150, fill=WHITE)
    for n, t in enumerate(["Jane Doe: design", "John Doe: code", "Alex Example: testing"]):
        b.text(24, y + 46 + n * 22, t, BLACK, 13)
    b.button(20, y + 200, 80, 26, "Back"); b.button(220, y + 200, 80, 26, "OK", default=True)
    return [("About box", a, None), ("Credits page", b, None)]


@xw("6.7")
def find_bar():
    def bar(c, text, count, replace=False):
        c.rect(0, 0, c.w, c.h, fill=PANEL)
        c.rect(8, 8, c.w - 16, 70, fill=WHITE)
        lines = ["let total = items.len();", "for item in items {", "    total += item.price;"]
        for n, l in enumerate(lines):
            c.text(14, 10 + n * 20, l, BLACK, 12)
        if text == "item":
            c.rect(76, 30, 30, 18, fill=BLACK, outline=None); c.text(76, 30, "item", WHITE, 12)
        y = 86
        c.text(10, y + 2, "Find:", BLACK, 12); c.field(60, y, 160, text, caret=len(text), focused=True)
        c.glyph(224, y + 2, 18, "up"); c.glyph(244, y + 2, 18, "down"); c.text(270, y + 2, count, DARK, 12)
        c.checkbox(330, y, "Aa"); c.checkbox(390, y, "Word"); c.checkbox(460, y, ".*")
        c.ctext(c.w - 30, y, 20, 22, "×", DARK, 14)
        if replace:
            c.text(10, y + 32, "Replace:", BLACK, 12); c.field(60, y + 30, 160, "entry")
            c.button(224, y + 30, 70, 22, "Replace"); c.button(300, y + 30, 90, 22, "Replace All")
    a = C(540, 120); bar(a, "item", "2 of 3")
    b = C(540, 150); bar(b, "item", "2 of 3", replace=True)
    return [("Ctrl+F: find bar, match count; Return goes next", a, "SearchChanged(3)"), ("Ctrl+H: replace row", b, "Replaced(n)")]


@xw("6.8")
def shortcuts_editor():
    c = C(480, 230)
    c.field(12, 12, 220, "save", caret=4)
    cols = [("Category", 110), ("Action", 160), ("Shortcut", 140)]
    c.rect(12, 42, 410, 22, fill=WHITE); cx = 12
    for t, w in cols:
        c.text(cx + 6, 44, t, BLACK, 12); cx += w
    c.rect(12, 64, 410, 66, fill=WHITE)
    rows = [("File", "Save", "Ctrl+S"), ("File", "Save As…", "Ctrl+Shift+S"), ("File", "Save All", "")]
    for n, r in enumerate(rows):
        ry = 64 + n * 22
        if n == 2:
            c.rect(13, ry, 408, 22, fill=BLACK, outline=None)
        cx = 12
        for v, (t, w) in zip(r, cols):
            c.text(cx + 6, ry + 2, v, WHITE if n == 2 else BLACK, 12); cx += w
    c.text(12, 144, "Shortcut for Save All:", BLACK, 13); c.field(170, 142, 150, "Ctrl+Shift+S", focused=True)
    c.text(170, 170, "Used by Save As…", DARK, 11); c.button(326, 140, 96, 24, "Reassign")
    c.button(12, 196, 80, 26, "Reset"); c.button(100, 196, 90, 26, "Reset All")
    return [("Filtered table; shortcut field reports a conflict", c, "ShortcutsChanged on Reassign")]


def main():
    for key, fn in XW.items():
        storyboard(fn(), os.path.join(HERE, f"xw-{key}-1.png"))
        print("wrote", f"xw-{key}-1.png")


if __name__ == "__main__":
    main()
