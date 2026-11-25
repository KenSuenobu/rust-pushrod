# Extra widget roadmaps

One roadmap per widget in the `pushrod-extra-widgets` crate: what it is, what it does, how it looks, and how it responds. These are larger or compound widgets kept out of the deliberately simple [standard widgets](../widgets/README.md). The [extra widgets roadmap](../ROADMAP_EXTRA_WIDGETS.md) explains why they are separate, which toolkits they come from, and the order they are built in. They follow the same [design philosophy](../../DESIGN_PHILOSOPHY.md): simple, flat, and predictable.

## Crate and foundations

| Widget | What it is | Issue | Milestone |
|---|---|---|---|
| [Create the pushrod-extra-widgets crate](crate.md) | The crate, its registration function, its docs section, and its place in CI. | [#252](https://github.com/KenSuenobu/rust-pushrod/issues/252) | X1 (MVP) |
| [Extra widgets gallery example](gallery.md) | A runnable gallery showing every extra widget, used for golden screenshots. | [#253](https://github.com/KenSuenobu/rust-pushrod/issues/253) | X1 (MVP) |
| [Popover: a floating panel anchored to a control](popover.md) | A borderless panel that opens next to a control, holds any widgets, and closes on Escape or a click outside. | [#254](https://github.com/KenSuenobu/rust-pushrod/issues/254) | X1 (MVP) |

## Buttons and value controls

| Widget | What it is | Issue | Milestone |
|---|---|---|---|
| [Menu button and split button](menu-button.md) | A button that opens a menu, and a button with a separate arrow part that opens one. | [#258](https://github.com/KenSuenobu/rust-pushrod/issues/258) | X2 (MVP) |
| [Segmented control](segmented-control.md) | A row of joined buttons: pick one, or toggle several. | [#50](https://github.com/KenSuenobu/rust-pushrod/issues/50) | X2 (MVP) |
| [Stepper (up and down arrows)](stepper.md) | A pair of small up/down arrows that step a value, attachable to any field. | [#259](https://github.com/KenSuenobu/rust-pushrod/issues/259) | X2 (MVP) |
| [Dial (rotary knob)](dial.md) | A round control turned by dragging, the wheel, or the keyboard, for values like volume, pan, or angle. | [#127](https://github.com/KenSuenobu/rust-pushrod/issues/127) | X2 (MVP) |
| [Level indicator (meter)](level-indicator.md) | A bar showing a level against warning and critical thresholds, continuous or in discrete segments. | [#54](https://github.com/KenSuenobu/rust-pushrod/issues/54) | X2 (MVP) |
| [Tool palette (grid of tool buttons)](tool-palette.md) | A grid of icon buttons with one-of-many selection and optional drag out, for tools, swatches, or components. | [#42](https://github.com/KenSuenobu/rust-pushrod/issues/42) | X1 (MVP) |
| [Ruler](ruler.md) | A horizontal or vertical scale in pixels, points, or units, with markers that can be dragged. | [#270](https://github.com/KenSuenobu/rust-pushrod/issues/270) | X3 |
| [Shortcut field (key combination recorder)](shortcut-field.md) | A field that records the key combination pressed while it has focus. | [#271](https://github.com/KenSuenobu/rust-pushrod/issues/271) | X3 |

## Pickers and special fields

| Widget | What it is | Issue | Milestone |
|---|---|---|---|
| [Calendar (month view)](calendar.md) | A month grid for choosing a day or a range of days, usable on its own or in the date picker. | [#260](https://github.com/KenSuenobu/rust-pushrod/issues/260) | X2 (MVP) |
| [Date picker](date-picker.md) | A date field with a drop-down month calendar. | [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) | X2 (MVP) |
| [Time field and date-time field](time-field.md) | A time entry with steppable parts, and a combined date-and-time field. | [#261](https://github.com/KenSuenobu/rust-pushrod/issues/261) | X2 (MVP) |
| [Color well and color picker](color-picker.md) | A swatch button that opens a color picker with swatches, sliders, and a hex field. | [#46](https://github.com/KenSuenobu/rust-pushrod/issues/46) | X2 (MVP) |
| [Font chooser](font-chooser.md) | A font button that opens a chooser with family, style, size, and a preview. | [#129](https://github.com/KenSuenobu/rust-pushrod/issues/129) | X2 (MVP) |
| [Path field (file or folder with Browse)](path-field.md) | A text field for a file or folder path, with a Browse button and drop support. | [#262](https://github.com/KenSuenobu/rust-pushrod/issues/262) | X2 (MVP) |
| [Image well](image-well.md) | A framed area that shows an image and accepts a new one by drop, paste, or Choose. | [#272](https://github.com/KenSuenobu/rust-pushrod/issues/272) | X3 |
| [New password (enter, confirm, strength)](new-password.md) | Two password fields with a strength meter and a match check. | [#273](https://github.com/KenSuenobu/rust-pushrod/issues/273) | X3 |
| [Character picker](character-picker.md) | A searchable grid of characters by block, with a preview and code point. | [#274](https://github.com/KenSuenobu/rust-pushrod/issues/274) | X3 |

## Data views

| Widget | What it is | Issue | Milestone |
|---|---|---|---|
| [Tree view (outline)](tree-view.md) | A hierarchical list with expand/collapse triangles, backed by a tree data source. | [#55](https://github.com/KenSuenobu/rust-pushrod/issues/55) | X2 (MVP) |
| [Column browser](column-browser.md) | Columns of lists where selecting an item in one column fills the next, as in NeXTSTEP and the Mac Finder. | [#45](https://github.com/KenSuenobu/rust-pushrod/issues/45) | X3 |
| [Icon grid view](icon-grid.md) | A scrolling grid of icons with captions, rubber-band selection, and drag and drop. | [#275](https://github.com/KenSuenobu/rust-pushrod/issues/275) | X3 |
| [Data grid (editable table)](data-grid.md) | A table whose cells edit in place with typed editors, with row add and delete. | [#276](https://github.com/KenSuenobu/rust-pushrod/issues/276) | X3 |
| [Property grid (name and value inspector)](property-grid.md) | Categorized name/value rows with an editor per value type. | [#255](https://github.com/KenSuenobu/rust-pushrod/issues/255) | X1 (MVP) |
| [Check list](check-list.md) | A list with a check box on every row, separate from the selection. | [#263](https://github.com/KenSuenobu/rust-pushrod/issues/263) | X2 (MVP) |
| [Editable list (add, remove, reorder)](editable-list.md) | A list with + and - buttons underneath, in-place editing, and reordering. | [#264](https://github.com/KenSuenobu/rust-pushrod/issues/264) | X2 (MVP) |
| [Dual list (list builder)](dual-list.md) | Available and chosen lists side by side, with buttons to move items between them and order the chosen list. | [#278](https://github.com/KenSuenobu/rust-pushrod/issues/278) | X3 |
| [Breadcrumb bar (path control)](breadcrumb-bar.md) | A row of clickable path segments, with a menu of siblings on each separator. | [#256](https://github.com/KenSuenobu/rust-pushrod/issues/256) | X1 (MVP) |
| [Filter field (live filter for lists, tables, and trees)](filter-field.md) | A search field bound to a list, table, or tree that hides non-matching rows as you type. | [#277](https://github.com/KenSuenobu/rust-pushrod/issues/277) | X3 |
| [Rule editor (query builder)](rule-editor.md) | Rows of "field, operator, value" with add and remove buttons, joined by All or Any. | [#279](https://github.com/KenSuenobu/rust-pushrod/issues/279) | X3 |
| [Image viewer (zoom and pan)](image-viewer.md) | An image in a scrollable view with zoom to fit, actual size, zoom by wheel, and drag to pan. | [#280](https://github.com/KenSuenobu/rust-pushrod/issues/280) | X3 |

## Panels and navigation

| Widget | What it is | Issue | Milestone |
|---|---|---|---|
| [Drawer (collapsible side panel)](drawer.md) | A side panel with a title bar that collapses to a thin strip and remembers its width. | [#257](https://github.com/KenSuenobu/rust-pushrod/issues/257) | X1 (MVP) |
| [Accordion (one section open at a time)](accordion.md) | A stack of titled sections where opening one closes the others. | [#265](https://github.com/KenSuenobu/rust-pushrod/issues/265) | X2 (MVP) |
| [Sidebar (navigation list)](sidebar.md) | A list of sections with icons and headings that switches the page beside it. | [#266](https://github.com/KenSuenobu/rust-pushrod/issues/266) | X2 (MVP) |
| [Document tabs (closable and reorderable)](document-tabs.md) | A tab bar for open documents: close buttons, a modified mark, drag to reorder, and overflow. | [#281](https://github.com/KenSuenobu/rust-pushrod/issues/281) | X3 |

## Dialogs and assistants

| Widget | What it is | Issue | Milestone |
|---|---|---|---|
| [Input dialog (ask for one value)](input-dialog.md) | A one-call modal dialog asking for a single text, number, or choice. | [#267](https://github.com/KenSuenobu/rust-pushrod/issues/267) | X2 (MVP) |
| [Progress dialog](progress-dialog.md) | A modal dialog with a message, a progress bar, and Cancel, for long tasks. | [#268](https://github.com/KenSuenobu/rust-pushrod/issues/268) | X2 (MVP) |
| [Task dialog (alert with details and options)](task-dialog.md) | An alert with a main instruction, expandable details, a "don't ask again" box, and command choices. | [#269](https://github.com/KenSuenobu/rust-pushrod/issues/269) | X2 (MVP) |
| [Wizard (step-by-step assistant)](wizard.md) | Pages shown one at a time with Back, Next, and Finish, and a list of steps. | [#282](https://github.com/KenSuenobu/rust-pushrod/issues/282) | X3 |
| [Settings dialog](settings-dialog.md) | A sidebar of categories, a page per category, and OK, Cancel, and Apply. | [#283](https://github.com/KenSuenobu/rust-pushrod/issues/283) | X3 |
| [About dialog](about-dialog.md) | The application's icon, name, version, copyright, website, and credits and license pages. | [#284](https://github.com/KenSuenobu/rust-pushrod/issues/284) | X3 |
| [Find and replace bar](find-bar.md) | A bar with find and replace fields, match options, a match count, and next and previous. | [#285](https://github.com/KenSuenobu/rust-pushrod/issues/285) | X3 |
| [Keyboard shortcuts editor](shortcuts-editor.md) | A filterable table of actions and their shortcuts, edited with the shortcut field. | [#286](https://github.com/KenSuenobu/rust-pushrod/issues/286) | X3 |

