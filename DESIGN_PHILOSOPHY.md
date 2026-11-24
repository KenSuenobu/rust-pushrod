# Pushrod design philosophy

Pushrod is meant to be small, simple, lightweight, and easy to use and maintain. This document
explains the rule behind every widget, event, and API decision, so contributors and application
developers know what to expect and what to propose.

## Keep It Stupid Simple

> Everything in Pushrod is simple, straightforward, and uncomplicated. While we could make
> buttons, sliders, and everything else look fancy, that's up to the developer to implement. We
> may add hooks to make this easier in future releases, but the current core design philosophy is
> the KISS method: **Keep It Stupid Simple.**
>
> Why? Performance, a clean and consistent look, usability, and predictable behavior.
>
> No glass. No beveled edges. No rounded corners.

## Why

**Performance.** A flat widget is a fill, a border, and some text. That is cheap on any GPU and
on SDL2's software renderer, and it stays cheap on slow and vintage hardware (the outline drawing
mode and the possible Atari ST target depend on it). Each widget paints into its own texture only
when it changes, and frames are composed by blitting those textures. There is no per-frame
shading, blurring, or blending to pay for.

**A clean, consistent look.** Every widget is drawn from the same few primitives, so a dialog
looks like one piece no matter who wrote its widgets. Pushrod draws everything itself, with no
OS-native controls, so an application looks the same on every platform.

**Usability.** State is shown by contrast, not by subtle shading: pressed and selected items
are inverted, disabled items are dithered, and the focused item has a dotted outline. These
read instantly and at any size or resolution.

**Predictable behavior.** Every widget follows the same rules: the same visual states, the same
keyboard handling, the same events and callbacks. Once you know one Pushrod widget, you know how
the rest respond.

## What a Pushrod widget looks like

| Element | How it's drawn |
|---|---|
| Body | A rectangle with a flat fill |
| Frame | A one-pixel border (the default button gets a two-pixel ring) |
| Text | Plain text in the widget's font, clipped to the widget |
| Symbols | Simple glyphs: arrows, check marks, the × of a clear button, a magnifier |
| Pressed or selected | Inverted colors |
| Disabled | Dithered |
| Keyboard focus | A dotted outline |
| Pop-up menus | The same white body and one-pixel border as any other widget, drawn above what they cover |

Widget frames are rectangles. A shape may be round only when the roundness *is* the control:
the radio button's circle, the dial, the busy spinner.

Each widget's exact look and behavior is specified in its own roadmap file under
[`docs/widgets/`](docs/widgets/README.md), with mockups.

## What Pushrod doesn't do

- No glass, translucency, or blur.
- No beveled edges, 3D shading, or embossing.
- No rounded corners on widgets.
- No gradients and no shadows, including under menus and pop-ups. (Pop-up shadows may be
  reconsidered after the MVP; they aren't needed for the look.)
- No decorative animation. Animation only where it carries information (a progress bar's busy
  mode, the busy spinner), and it stops when nothing is happening.
- No OS-native menus, dialogs, or controls.
- No widget added just because another toolkit has it. If ordinary applications don't need it,
  it is parked.

## If you want it fancier

That's up to the application, and Pushrod stays out of the way:

- **Draw it yourself.** A widget's `paint` function is yours: write your own widget, or use the
  Custom Paint widget for one-off drawing.
- **Change colors, border, and font per state.** Per-state appearance lets an application restyle
  a widget within the flat style.
- **Theming hooks may come later.** Themes and skins are a distant-future epic and are not
  scheduled. Until then, the core stays simple.

## The same rule beyond looks

KISS applies to the whole library, not just to drawing:

- **Few dependencies.** Nothing new at run time without a strong reason.
- **Self-contained.** SDL2 for graphics and input; Pushrod draws everything else.
- **One event model.** Every internal action emits an event. Handle events with callbacks
  registered by widget ID, or with your own event loop; both use the same events.
- **IDs are the contract.** Widgets are addressed by stable IDs, so forms built in a designer
  and code written by hand work the same way.
- **Fictional sample data.** Demos, examples, tests, mockups, and documentation never contain
  real names or personal information (see [CONTRIBUTING.md](CONTRIBUTING.md)).

## Checking a proposal

Before proposing a widget, a feature, or a visual change, ask:

1. Can it be drawn with fills, one-pixel borders, text, and simple glyphs?
2. Does it behave like the other widgets: same states, keyboard handling, events, and callbacks?
3. Does it work in outline drawing mode and on slow hardware?
4. Would ordinary applications use it, or is it there because another toolkit has it?
5. Does it add a dependency? If so, why is that worth it?

If the answers point toward "fancier", the feature probably belongs in the application, not in
Pushrod.
