# Form file format (version 1)

A form file stores one or more forms (widget trees) compactly so programs refer to every object
by its **object number**. It is little-endian throughout and is read and written with the
standard library only by `pushrod::form` (`FormFile::to_bytes`, `FormFile::from_bytes`).

## Layout

```text
+---------------------------+
| Header  'PRSC' v1 offsets |  56 bytes
+---------------------------+
| Form table   name -> root |
| Object table (per widget) |  48 bytes per object
| Property blocks (TLV)     |
| String pool               |
| Image pool                |
| Symbols (ID and action)   |
| Resource tables (optional)|
+---------------------------+
```

Sections follow the header in the order above. A reader finds them only through the header's
offsets, so writers may add padding or new sections after a future version bump.

## Header (56 or 64 bytes)

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 4 | Magic, the bytes `PRSC` |
| 4 | 2 | Format version (`u16`, currently `1`) |
| 6 | 2 | Header length in bytes (`u16`, at least `56`) |
| 8 | 8 | Form table: offset (`u32`), size (`u32`) |
| 16 | 8 | Object table: offset, size |
| 24 | 8 | Property section: offset, size |
| 32 | 8 | String pool: offset, size |
| 40 | 8 | Image pool: offset, size |
| 48 | 8 | Symbol table: offset, size |
| 56 | 8 | Resource section: offset, size (only when the header length is `64`) |

Offsets are from the start of the file. Every section must lie inside the file. A file with no
resources has a 56-byte header and is exactly what a reader written before resources existed
expects; a file with resources has a 64-byte header, which such a reader accepts and ignores (it
only requires the length to be at least `56`).

## Versioning

- The version is bumped when a change would make an old reader misread a file.
- A reader accepts its own version and older, and rejects a newer one (`UnsupportedVersion`).
- New properties do **not** need a bump: they use new tags (see below) that old readers skip.

## Strings

The string pool is `count: u32`, then each string as `length: u32` and that many UTF-8 bytes.
Everywhere else a string is a `u32` index into the pool. The writer stores each distinct string
once.

## Form table

`count: u32`, then per form: `name: u32` (string index), `root: u32` (object number).

## Object table

`count: u32`, then one 48-byte record per object; the **object number is the record's index**.

| Field | Type | Meaning |
| --- | --- | --- |
| type | `u32` | String index of the registered type name (`pushrod.button`) |
| parent | `u32` | Object number, or `0xFFFFFFFF` for a root |
| first child | `u32` | Object number or `0xFFFFFFFF` |
| next sibling | `u32` | Object number or `0xFFFFFFFF` |
| x, y | `i32` each | Origin relative to the parent |
| width, height | `u32` each | Size |
| behavior | `u32` | Behavior flags |
| state | `u32` | State flags |
| anchors | `u32` | Which edges follow the parent |
| properties | `u32` | Offset of the object's block in the property section, or `0xFFFFFFFF` for none |

Children form a linked list: the parent names the first child, and each child names the next
sibling and its parent. A reader checks every link, rejects an object that is linked twice or
missing from its parent's list, and rejects parent cycles.

## Property blocks

A block is `length: u32`, then entries until `length` bytes are consumed. An entry is
**tag-length-value**: `tag: u16`, `length: u32`, then `length` bytes. A reader skips any tag it
does not know, using the length.

| Tag | Name | Value |
| --- | --- | --- |
| 1 | Named property | `name: u32` (string index), `kind: u8`, then the value |
| 2 | Layout | `kind: u8`, `count: u32`, then `count` `i32` parameters: how a container lays out its children |
| 3 | Child layout | `stretch: u16`, `horizontal: u8`, `vertical: u8`, margins (4 × `u32`: left, top, right, bottom), grid cell (4 × `u16`: column, row, column span, row span) |

Named property value kinds (an unknown kind is skipped like an unknown tag):

| Kind | Value |
| --- | --- |
| 0 | Bool: one byte |
| 1 | Int: `i64` |
| 2 | Float: `f64` |
| 3 | Text: `u32` string index |
| 4 | Color: four bytes, red, green, blue, alpha |
| 5 | Image: `u32` index into the image pool |

The layout kind and alignment codes are stored as written and interpreted by the loader.

## Image pool

`count: u32`, then per image: `name: u32` (string index), `length: u32`, and `length` bytes of
encoded image data (for example a PNG file), stored untouched.

## Symbol table

`count: u32`, then per symbol 9 bytes: `kind: u8` (`0` ID, `1` action), `id: u32`, `name: u32`
(string index). An ID symbol names an object number (`OK_BUTTON` → 2); an action symbol names an
action number a program binds to by name.

## Resource section

Resources are the data a program ships next to its forms: strings, images, icons, files, menus,
and alerts. The section is a list of **tables** in tag-length-value form: `tag: u16`,
`length: u32`, then `length` bytes. A reader skips a tag it does not know, so tables added later
do not break older readers. A table that is empty is not written, and a known tag may appear once.

| Tag | Table | Value of an entry |
| --- | --- | --- |
| 1 | Strings | `text` |
| 2 | Images | `data` (for example a PNG file) |
| 3 | Icons | `image: u32`, `mask: u32`, `selected: u32` (image IDs, `0` for none), `caption` |
| 4 | Files | `mime`, `data` |
| 5 | Menus | `kind: u8` (`0` menu bar, `1` context menu), then an item list |
| 6 | Alerts | `kind: u8` (`0` info, `1` warning, `2` error, `3` question), `title: u32`, `message: u32`, `count: u32`, then `count` button `u32`s (all string IDs) |

Here `text`, `mime`, and names are a `u32` length and that many UTF-8 bytes (stored inline, not in
the string pool), and `data` is a `u32` length and that many bytes.

### Table body and stable IDs

A table body is `next_id: u32`, `count: u32`, then per entry `id: u32`, `name` (a string), and the
kind's value. IDs are assigned once, starting at `1`, and **never reused or renumbered**:

- Adding an entry gives it `next_id`, then `next_id` moves up by one.
- Removing or reordering entries changes no other entry's ID, and `next_id` is saved, so the ID of
  a removed entry stays retired even when it was the last one.
- A reader rejects `0`, a repeated ID, an ID at or above `next_id`, and a repeated name within a
  table.

Names are unique within a table, so one name finds one resource (`SAVE_FAILED`).

### Menu items

An item list is `count: u32`, then per item `kind: u8`, `length: u32`, and `length` bytes, so a
reader skips an item kind it does not know.

| Kind | Item | Body |
| --- | --- | --- |
| 0 | Command | `label`, `action` (the action name a program binds to), `flags: u8` (bit 0: enabled, bit 1: a shortcut follows), then `shortcut` when flagged |
| 1 | Separator | empty |
| 2 | Submenu | `label`, then an item list |

Menus nest at most 16 levels; a reader and a writer reject deeper ones.

### References

An icon's image IDs name entries of the image table, and an alert's title, message, and button IDs
name entries of the string table. A reader and a writer reject an ID that names nothing. Alerts
refer to strings, rather than holding text, so a string table per language can replace the strings
later without touching the alerts.

## Reading safely

Readers check every offset, length, count, string index, and object number, and report a
`FormError` instead of panicking. Entry counts are checked against the bytes that remain, so a
damaged count cannot force a huge allocation.
