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
+---------------------------+
```

Sections follow the header in the order above. A reader finds them only through the header's
offsets, so writers may add padding or new sections after a future version bump.

## Header (56 bytes)

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

Offsets are from the start of the file. Every section must lie inside the file.

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

## Reading safely

Readers check every offset, length, count, string index, and object number, and report a
`FormError` instead of panicking. Entry counts are checked against the bytes that remain, so a
damaged count cannot force a huge allocation.
