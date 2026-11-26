---
sidebar_position: 8
title: Text
---

# Text

Pushrod separates **laying out** text from **drawing** it.

- A **layout engine** works out how big the text is, where each line breaks, where each glyph goes, and where the caret and selection sit. It never draws.
- The **painter** draws: each glyph is rasterized once into a texture atlas on the GPU, and every later frame copies rectangles from that atlas.

```mermaid
flowchart LR
    A[Text + TextStyle + width] --> B[LayoutCache]
    B -- text, font or width changed --> C[TextLayout engine]
    B -- unchanged --> D[cached LaidOut]
    C --> D
    D --> E[Glyph runs]
    E -- first time only --> F[swash rasterizes glyph]
    F --> G[Atlas texture]
    E -- every frame --> H[Canvas::copy from atlas]
    G --> H
    D --> I[Size hint, caret, selection]
```

Steady-state text drawing does no CPU rasterization: once the text on screen has been drawn once, a frame is texture copies and nothing else.

## The `TextLayout` trait

`pushrod::text::TextLayout` takes a string, a `TextStyle` and a maximum width, and returns a `LaidOut`:

| Field or method | Meaning |
| --- | --- |
| `size` | Width and height of the text in pixels. |
| `min_width` | The narrowest width the text can wrap to (its longest word). |
| `size_hint()` | `(min_width, size)`, for layout managers. |
| `lines` | Each line's byte range in the text and its bounds. |
| `runs` | Glyph runs to draw (font, size, glyph ids and positions). |
| `geometry.caret_rect(index)` | The caret rectangle for a byte index, for text fields. |
| `geometry.selection_rects(range)` | The rectangles to fill for a selection (several per line in mixed-direction text). |
| `geometry.hit_test(point)` | The byte index nearest a point, for mouse clicks. |

Two engines ship with Pushrod:

| Engine | Cargo feature | What it does |
| --- | --- | --- |
| `TtfLayout` | none (default) | Measures with SDL2_ttf: left-to-right text wrapped at spaces, with caret and selection geometry. It returns no glyph runs because the SDL2 painter draws its text with SDL2_ttf. |
| `ParleyLayout` | `parley` (off by default) | Uses [Parley](https://github.com/linebender/parley): shaping, right-to-left and mixed-direction text, line breaking, and font fallback for glyphs the font lacks (CJK, emoji). It returns glyph runs. |

Applications can implement `TextLayout` for another engine, such as Cosmic Text or a bitmap font.

## Layout caching

Laying text out is the expensive part, so give each widget a `LayoutCache`. It keeps the last `LaidOut` and lays out again only when the text, font name, font size, font style or width changes. Changing only the color or the justification does not.

```rust checked
use pushrod::color::{Color, FontStyle};
use pushrod::geometry::{Point, Rect, Size};
use pushrod::paint::{TextJustify, TextStyle};
use pushrod::text::{LaidOut, LayoutCache, Line, TextGeometry, TextLayout};
use std::ops::Range;

struct Fixed;

impl TextGeometry for Fixed {
    fn caret_rect(&self, index: usize) -> Rect {
        Rect { x: index as i32 * 8, y: 0, w: 1, h: 16 }
    }
    fn selection_rects(&self, range: Range<usize>) -> Vec<Rect> {
        vec![Rect { x: range.start as i32 * 8, y: 0, w: range.len() as u32 * 8, h: 16 }]
    }
    fn hit_test(&self, point: Point) -> usize {
        (point.x.max(0) as usize + 4) / 8
    }
}

/// A made-up engine: every character is 8 pixels wide and 16 tall.
struct FixedWidth;

impl TextLayout for FixedWidth {
    fn layout(&mut self, text: &str, _style: &TextStyle, _max_width: Option<u32>) -> LaidOut {
        let size = Size { w: text.chars().count() as u32 * 8, h: 16 };
        LaidOut {
            size,
            min_width: 8,
            lines: vec![Line { text_range: 0..text.len(), bounds: Rect { x: 0, y: 0, w: size.w, h: 16 } }],
            runs: Vec::new(),
            geometry: Box::new(Fixed),
        }
    }
}

fn lay_out_twice() {
    let style = TextStyle {
        font_name: "assets/OpenSans-Regular.ttf".to_string(),
        font_style: FontStyle::NORMAL,
        font_size: 16,
        font_color: Color::BLACK,
        justification: TextJustify::Left,
    };

    let mut cache = LayoutCache::new();
    let mut engine = FixedWidth;

    cache.get(&mut engine, "Jane Example", &style, Some(200));
    cache.get(&mut engine, "Jane Example", &style, Some(200));
    assert_eq!(cache.layouts(), 1); // the second call reused the layout

    cache.get(&mut engine, "Jane Example", &style, Some(100));
    assert_eq!(cache.layouts(), 2); // a new width lays out again
}
```

## Truncation

Text that is wider than its space is shortened and marked with an ellipsis (`…`), instead of being clipped mid-letter. `paint::text` takes a `Truncate` mode:

| Mode | Result for `Quarterly report.txt` in ten characters |
| --- | --- |
| `Truncate::None` | Not shortened; the backend clips it as before. |
| `Truncate::End` (default) | `Quarterly…` |
| `Truncate::Start` | `…eport.txt` |
| `Truncate::Middle` | `Quart….txt`, for paths and file names. |

The cut falls on a grapheme boundary, so an accent, flag or emoji is never split, and the ellipsis is measured in the same font as the text, so the result never overflows. Text that fits is drawn unchanged. Truncation applies to single-line text; text that contains a newline is left alone. If not even the ellipsis fits, nothing is drawn.

```mermaid
flowchart LR
    A[Text] --> B{Fits the width?}
    B -- yes --> C[Draw unchanged]
    B -- "no, mode None" --> D[Backend clips]
    B -- "no, End / Start / Middle" --> E[fit_text cuts on a grapheme<br/>and adds the ellipsis]
    E --> F[Draw shortened text,<br/>truncated = true]
```

`TextWidget` and `ButtonWidget` have a `truncate` property (`set_truncate`, `get_truncate`) that defaults to `Truncate::End`, and `is_truncated()` reports whether the last paint shortened the text, so a tooltip can show the full text. Every widget that draws text through `paint::text` behaves the same way. A widget paints into its cached texture only when invalidated, so the shortened text is computed once per change, not once per frame.

`fit_text` is the pure helper behind it. Give it a function that measures a string:

```rust checked
use pushrod::paint::{fit_text, Truncate};

fn main() {
    // Pretend every character is 5 pixels wide; a painter measures with the real font.
    let mut measure = |s: &str| s.chars().count() as u32 * 5;
    let name = "Quarterly report.txt";

    assert_eq!(fit_text(name, 50, Truncate::End, &mut measure).as_deref(), Some("Quarterly…"));
    assert_eq!(fit_text(name, 50, Truncate::Start, &mut measure).as_deref(), Some("…eport.txt"));
    assert_eq!(fit_text(name, 50, Truncate::Middle, &mut measure).as_deref(), Some("Quart….txt"));
    // Text that fits comes back as `None`: draw it unchanged.
    assert_eq!(fit_text("Short", 50, Truncate::End, &mut measure), None);
}
```

## Wrapping

A label draws one line unless you turn wrapping on. `TextWidget::set_wrap` takes a `Wrap` mode, and `set_max_lines` limits the lines.

| Mode | Lines break |
| --- | --- |
| `Wrap::None` (default) | Never; one line, shortened by `Truncate` if it doesn't fit. |
| `Wrap::Word` | At spaces. A word wider than the widget is broken between graphemes. |
| `Wrap::Anywhere` | Between any two graphemes, as late as possible. |

- An explicit `\n` always starts a new line, and a blank line stays blank.
- Each line is aligned on its own by the widget's justification (left, center or right).
- The lines are recomputed whenever the widget is painted, so resizing the widget reflows the text.
- If `max_lines` is set and the text needs more lines, the last line ends in an ellipsis; `is_truncated()` reports it. While wrapping is on, `truncate` is not used.
- `height_for_width(painter, width)` returns the height the text needs at a width (lines times line height), so a layout can give a wrapped label the room it needs.

```mermaid
flowchart TD
    A[Text] --> B[Split at explicit newlines]
    B --> C{Wrap mode}
    C -- None --> D[One line per paragraph]
    C -- Word --> E[Greedy lines at spaces]
    C -- Anywhere --> F[Greedy lines at graphemes]
    D --> G{More than max_lines?}
    E --> G
    F --> G
    G -- no --> H[Draw each line, aligned]
    G -- yes --> I[Cut the last line with an ellipsis] --> H
```

`wrap_text` is the pure helper behind it:

```rust checked
use pushrod::paint::{wrap_text, Wrap};

fn main() {
    // Pretend every character is 5 pixels wide; a painter measures with the real font.
    let mut measure = |s: &str| s.chars().count() as u32 * 5;
    let msg = "the quick brown fox jumps";

    let (lines, cut) = wrap_text(msg, 50, Wrap::Word, None, &mut measure);
    assert_eq!(lines, ["the quick", "brown fox", "jumps"]);
    assert!(!cut);

    // Limited to two lines, the last one ends in an ellipsis.
    let (lines, cut) = wrap_text(msg, 50, Wrap::Word, Some(2), &mut measure);
    assert_eq!(lines, ["the quick", "brown fox…"]);
    assert!(cut);
}
```

## Using Parley

Enable the feature in your `Cargo.toml`:

```toml
pushrod = { version = "0.0.9", features = ["parley"] }
```

Then lay out with `ParleyLayout` and draw the glyph runs. The `parley_spike` example does this end to end: Parley computes the runs, swash rasterizes each glyph once into a CPU bitmap, the bitmap is uploaded into an SDL2 atlas texture, and glyphs are drawn with `Canvas::copy` and color modulation (color glyphs such as emoji are copied unmodulated).

```bash
cargo run --release -p pushrod-sdl2 --example parley_spike --features parley
```

![The spike: Latin, Hebrew, Japanese and emoji text, and a text field with a selection and caret](/img/screenshots/parley-international-text.png)

The example prints the three measurements of the compatibility gate:

1. After the first frame, **0 glyphs are rasterized or uploaded** over the measured frames.
2. Glyphs are drawn with `Canvas::copy` from an atlas texture, the same texture and blit path as every other widget.
3. Binary size, the time to create the engine (which discovers system fonts), and the time of the first and of a steady frame.

### Why `parley` is off by default

The layout engine and the rasterizer pass the gate, but Parley adds a sizable dependency tree and discovers system fonts at start-up, so `ParleyLayout` is behind the `parley` feature and `TtfLayout` stays the default until the glyph atlas ([#3](https://github.com/KenSuenobu/rust-pushrod/issues/3)) moves the SDL2 painter onto glyph runs.

## See also

- The widget gallery shows the truncation and wrapping modes on `TextWidget` (`cargo run -p pushrod-sdl2 --example gallery`).
- [Drawing](drawing.md) for how widgets paint into cached textures.
- [Backend-neutral paint API](../platforms/backend-neutral-paint-api.md) for the `Painter` trait.
