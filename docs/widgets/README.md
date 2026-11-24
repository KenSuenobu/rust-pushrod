# Widget roadmaps

One roadmap per widget: what it is, what it does, how it looks, and how it responds. Each file is the specification for its widget; change a widget by editing its file in a pull request. The [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) covers the core, events, forms, and release work, and the order everything is built in. Every widget follows the [design philosophy](../../DESIGN_PHILOSOPHY.md): simple, flat, and predictable.

## Core dialog widgets

| Widget | What it is | Issue | Release |
|---|---|---|---|
| [Boxed Label](boxed-label.md) | Text with background and border. | [#27](https://github.com/KenSuenobu/rust-pushrod/issues/27) | MVP |
| [Glyph Button](glyph-button.md) | A small box with an arrow or symbol, used for scroll arrows, close boxes, and spin buttons. | [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) | MVP |
| [Button](button.md) | A push button that runs an action when clicked; it can also be a dialog's default or cancel button, a toggle, or a button that closes a dialog. | [#29](https://github.com/KenSuenobu/rust-pushrod/issues/29) | MVP |
| [Check Box](check-box.md) | Box with a check mark and label; toggles checked; optional mixed state. | [#30](https://github.com/KenSuenobu/rust-pushrod/issues/30) | MVP |
| [Radio Button](radio-button.md) | One-of-many selection among sibling widgets. | [#31](https://github.com/KenSuenobu/rust-pushrod/issues/31) | MVP |
| [Text Field](text-field.md) | Single-line text entry with caret, selection, clipboard, input mask, validator, and numeric mode. | [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) | MVP |
| [Image](image.md) | Display a PNG/BMP image centered, stretched, scaled to fit, or tiled. | [#33](https://github.com/KenSuenobu/rust-pushrod/issues/33) | MVP |
| [Icon](icon.md) | Picture with a caption, selectable and double-clickable, like desktop icons. | [#34](https://github.com/KenSuenobu/rust-pushrod/issues/34) | later |
| [Alert](alert.md) | One-call modal alert with icon, message, and up to three buttons. | [#35](https://github.com/KenSuenobu/rust-pushrod/issues/35) | MVP |
| [Custom Paint](custom-paint.md) | One-off custom drawing without writing a widget struct. | [#36](https://github.com/KenSuenobu/rust-pushrod/issues/36) | MVP |
| [Menu Bar and Menus](menu-bar-and-menus.md) | Menu bar with drop-down menus, separators, check marks, submenus, shortcuts, and mnemonics. | [#37](https://github.com/KenSuenobu/rust-pushrod/issues/37) | later |
| [File Chooser](file-chooser.md) | Choose a file or directory with a consistent look on every platform. | [#38](https://github.com/KenSuenobu/rust-pushrod/issues/38) | later |
| [Tooltip](tooltip.md) | Short help text shown after the pointer rests on a widget. | [#77](https://github.com/KenSuenobu/rust-pushrod/issues/77) | later |
| [Separator](separator.md) | Horizontal or vertical divider line. | [#83](https://github.com/KenSuenobu/rust-pushrod/issues/83) | MVP |
| [Toolbar](toolbar.md) | A row of icon/text buttons bound to actions. | [#84](https://github.com/KenSuenobu/rust-pushrod/issues/84) | later |
| [Status Bar](status-bar.md) | A strip at the bottom of the window for messages and status fields. | [#85](https://github.com/KenSuenobu/rust-pushrod/issues/85) | later |
| [Switch](switch.md) | A sliding on/off switch for settings that take effect immediately. | [#86](https://github.com/KenSuenobu/rust-pushrod/issues/86) | later |
| [Text Area](text-area.md) | Multi-line plain-text editing with scrolling, selection, clipboard, and undo. | [#87](https://github.com/KenSuenobu/rust-pushrod/issues/87) | later |
| [Context Menu](context-menu.md) | Right-click (or long-press) pop-up menus for a widget. | [#88](https://github.com/KenSuenobu/rust-pushrod/issues/88) | later |
| [Message Bar](message-bar.md) | A dismissible banner for warnings and information that doesn't block the window. | [#89](https://github.com/KenSuenobu/rust-pushrod/issues/89) | later |
| [Image Button](image-button.md) | A button that shows an image, or an image with a text label, with the full button behavior. | [#125](https://github.com/KenSuenobu/rust-pushrod/issues/125) | later |
| [Link Label](link-label.md) | Clickable text that looks like a link and opens a URL or runs a callback. | [#126](https://github.com/KenSuenobu/rust-pushrod/issues/126) | later |

## Value and choice controls

| Widget | What it is | Issue | Release |
|---|---|---|---|
| [Scroll Bar](scroll-bar.md) | Vertical/horizontal scroll bar with arrows and a proportional thumb. | [#39](https://github.com/KenSuenobu/rust-pushrod/issues/39) | MVP |
| [Slider](slider.md) | Horizontal/vertical slider with range, optional ticks, and value label. | [#40](https://github.com/KenSuenobu/rust-pushrod/issues/40) | MVP |
| [Group Box](group-box.md) | A frame with a title in its top border, grouping related controls. | [#41](https://github.com/KenSuenobu/rust-pushrod/issues/41) | MVP |
| [Choice Button](choice-button.md) | Pick one of several options compactly, as a pop-up list or a click-to-cycle button. | [#44](https://github.com/KenSuenobu/rust-pushrod/issues/44) | later |
| [Column Browser](column-browser.md) | Columns of lists where selecting an item in one column fills the next, as in the NeXTSTEP browser and the Mac Finder's column view. | [#45](https://github.com/KenSuenobu/rust-pushrod/issues/45) | later |
| [Split View](split-view.md) | Two or more panes side by side or stacked, separated by dividers the user drags to resize them. | [#47](https://github.com/KenSuenobu/rust-pushrod/issues/47) | later |
| [Expander](expander.md) | A titled section that shows or hides its contents when clicked. | [#90](https://github.com/KenSuenobu/rust-pushrod/issues/90) | later |
| [Date Picker](date-picker.md) | A date field with a drop-down month calendar. | [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) | later |
| [Dial](dial.md) | A round control turned by dragging, the wheel, or the keyboard, for values like volume, pan, or angle. | [#127](https://github.com/KenSuenobu/rust-pushrod/issues/127) | later |

## Lists, tables, progress, spin boxes, and tabs

| Widget | What it is | Issue | Release |
|---|---|---|---|
| [Progress Bar](progress-bar.md) | A bar showing how far along a task is; with no value it shows a busy animation (like HTML's progress element). | [#48](https://github.com/KenSuenobu/rust-pushrod/issues/48) | MVP |
| [Spin Box](spin-box.md) | Numeric field with up/down arrows, range, and step. | [#49](https://github.com/KenSuenobu/rust-pushrod/issues/49) | later |
| [Tab View](tab-view.md) | Tabs that switch between pages of controls. | [#51](https://github.com/KenSuenobu/rust-pushrod/issues/51) | later |
| [List View](list-view.md) | Scrolling list with single/multiple selection, backed by a data source. | [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) | later |
| [Combo Box](combo-box.md) | Text field with a drop-down list of suggestions. | [#53](https://github.com/KenSuenobu/rust-pushrod/issues/53) | later |
| [Table View](table-view.md) | Multi-column list with headers, column resizing, and sorting. | [#92](https://github.com/KenSuenobu/rust-pushrod/issues/92) | later |
| [Spinner](spinner.md) | A small animated indicator for work of unknown length, for use inline or on buttons. | [#128](https://github.com/KenSuenobu/rust-pushrod/issues/128) | later |

## Composition, clipping, and layout managers

| Widget | What it is | Issue | Release |
|---|---|---|---|
| [Container](container.md) | A widget whose children live on a content area with its own size and 0,0 origin; the container's frame is a viewport panned by scroll bars. | [#58](https://github.com/KenSuenobu/rust-pushrod/issues/58) | MVP |

## Parked

Considered and not planned. Reopen the issue if real applications need one.

| Widget | Why parked | Issue |
|---|---|---|
| Matrix of cells (grid of buttons with selection modes) | Radio groups ([2.5 Radio Button](radio-button.md)) and the choice button ([3.6 Choice Button](choice-button.md)) cover the common cases; palettes can be built from buttons. | [#42](https://github.com/KenSuenobu/rust-pushrod/issues/42) |
| Labeled field rows | Text fields with built-in labels ([2.6 Text Field](text-field.md)) cover this. | [#43](https://github.com/KenSuenobu/rust-pushrod/issues/43) |
| Color well and picker | Rarely needed in ordinary dialogs; revisit if the Chassis inspector needs it. | [#46](https://github.com/KenSuenobu/rust-pushrod/issues/46) |
| Segmented control | Radio groups and the choice button's cycle mode cover the same choices. | [#50](https://github.com/KenSuenobu/rust-pushrod/issues/50) |
| Level indicator | The progress bar covers capacity displays. | [#54](https://github.com/KenSuenobu/rust-pushrod/issues/54) |
| Outline view (expandable tree list) | Specialized; revisit once the list view is in use. | [#55](https://github.com/KenSuenobu/rust-pushrod/issues/55) |
| Font chooser | Specialized; most applications don't let users pick fonts. Revisit alongside the color picker. | [#129](https://github.com/KenSuenobu/rust-pushrod/issues/129) |
