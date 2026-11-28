---
sidebar_position: 1
title: Projects and resources
description: How Chassis models a project, which files it owns and which are yours, and how it finds forms, resources, and handlers without ever overwriting your code.
---

# Projects and resources

A Chassis **project** is an ordinary Cargo crate. Chassis has to know which of its files it may write and which are yours, so it can never overwrite your code. The `chassis` crate's `Project` type holds that knowledge. It has no window and is fully unit-tested.

```mermaid
flowchart LR
    M["Cargo.toml<br/>(yours, read only)"] --> P[Project]
    R["app.prsc<br/>(Chassis, via Pushrod)"] --> P
    H["src/forms/&lt;form&gt;.rs<br/>(yours, parsed)"] --> P
    G["src/generated/<br/>(Chassis)"] --> P
    P --> F[Forms]
    P --> C[Resource counts]
    P --> D[Handlers]
```

## What a project contains

| Path | Owner | What Chassis may do |
|---|---|---|
| `Cargo.toml` | You | Read it. |
| `*.prsc` (the resource file) | Chassis | Read and write it, through Pushrod's form reader and writer. |
| `src/forms/<form>.rs` | You | Create a starter stub once, when the file does not exist. Never overwrite it. |
| `src/generated/**` | Chassis | Write and replace freely; it is regenerated. |
| Everything else | You | Read it. |

The resource file holds the forms, strings, images, icons, files, menus, and alerts. Its name comes from `[package.metadata.chassis] resources` in `Cargo.toml`; without that key, the project's only `*.prsc` file is used. A project with no resource file simply has no forms yet, and several `*.prsc` files without a named choice is an error.

A form's handler file is its name in lowercase words: `MainWindow` is `src/forms/main_window.rs`.

## Loading a project

```rust
use chassis::Project;

let project = Project::load("path/to/project")?;

println!("{} {}", project.package().name, project.package().version);

for form in project.forms() {
    for handler in project.handlers_for(&form.name) {
        println!("{}: {}", form.name, handler.signature);
    }
}
# Ok::<(), chassis::ProjectError>(())
```

`Project::load` reads `Cargo.toml` (with `toml_edit`), the resource file, and every `.rs` file in `src/forms` (with tree-sitter). A **handler** is a top-level `fn` in a handler file: its name, its signature as written, the file, and the line. The model reads the code on disk each time it loads, never a cached copy, so it matches what you wrote. `reload_handlers` re-reads the files after an edit. A handler file with syntax errors is reported as `ProjectError::Handlers`, naming the file.

## Ownership is enforced

`Project::ownership(path)` says who owns a path (`User`, `Handler`, `Resource`, or `Generated`). Chassis writes only through three methods, and each refuses what it must not touch with `ProjectError::Ownership`:

- `write_generated` writes under `src/generated` only; a path that leaves the folder (for example `src/generated/../main.rs`) is refused.
- `create_stub` creates a form's handler file only if it does not exist. If it exists, it is yours and stays untouched.
- `save_resources` and `create_resource_file` write the resource file, after checking it is valid.

## Code generation rules

Chassis follows written rules when it writes code, and tests in `chassis/tests/codegen.rs` enforce them. The functions live in `chassis::codegen`.

```mermaid
flowchart TD
    A[Designer operation] --> B{Which file?}
    B -->|src/generated/| C["Rewrite, then rustfmt<br/>write_generated"]
    B -->|src/main.rs| D["Create once<br/>create_main"]
    B -->|src/forms/&lt;form&gt;.rs| E{Exists?}
    E -->|No| F["Create stub once<br/>create_stub"]
    E -->|Yes| G["Append a stub or rename a signature<br/>insert_handler_stub, rename_handler"]
    B -->|Anything else| H[Never written]
```

| Rule | Function |
|---|---|
| Chassis rewrites only `src/generated/`, and formats what it writes with `rustfmt`. Source that `rustfmt` rejects is not written. | `write_generated` |
| `src/main.rs` and `src/forms/<form>.rs` are created once. A file that exists is yours and is never overwritten. | `create_main`, `create_stub` |
| After that, Chassis only appends a handler stub. A function with the same name is never replaced. | `insert_handler_stub` |
| Or it renames a handler in its signature. Only the bytes of the name change; the new name must be a valid identifier that the file does not already use. | `rename_handler` |
| Chassis never edits a function body. Comments, spacing, and code outside the changed name are kept exactly. | all of the above |

A call to the old name inside a body (a recursive call, for example) is not updated; that code is yours. Every edit is parsed again before it is written, and a refused request leaves the file untouched.

## Event wiring

You never write registration code. Name a handler `on_<widget>_<event>` in snake case in the form's handler file, and Chassis wires it. `on_ok_button_clicked` handles the `Clicked` event of the widget whose ID symbol is `OK_BUTTON`; `on_accept_toggled` handles `Toggled` on `ACCEPT`. The widget types and their events come from Pushrod's widget registry (its event metadata), so the generator knows which engine method registers each event.

```mermaid
flowchart LR
    H["src/forms/&lt;form&gt;.rs<br/>on_ok_button_clicked"] --> M[match_handlers]
    R["Widget registry<br/>events per widget type"] --> M
    F["Resource file<br/>ID symbols"] --> M
    M --> G["generate_wiring"]
    G --> W["src/generated/&lt;form&gt;_wiring.rs<br/>wire(engine, form)"]
```

`write_wiring` writes `src/generated/<form>_wiring.rs`, for example `main_window_wiring.rs`:

```rust
pub fn wire(engine: &mut Engine<AppState>, form: &FormHandle) {
    if let Some(id) = form.widget(ids::main_window::OK_BUTTON) {
        engine.on_click(id, on_ok_button_clicked);
    }
}
```

Call `wire` once after loading the form. The file starts with a do-not-edit header, is rewritten in full, and is formatted like every generated file. It uses the typed ID constants (`ids`) and imports your handlers from `crate::forms::<form>`, and it expects an `AppState` type at the crate root. The output is deterministic: entries are sorted by handler name, so the same form file, handlers, and registry always give the same file.

| Situation | Result |
|---|---|
| A function whose name matches no widget and event (a helper) | Ignored. |
| A matching handler that is not `pub` | Refused, naming the handler. |
| An event with no engine registration method (`KeyReleased`, for example) | Refused; handle it with `Engine::on_event`. |
| Two handlers on one widget that use the same registration method (`on_ok_button_focus_gained` and `on_ok_button_focus_lost` both use `on_focus_change`) | Refused; keep one, which receives both. |
| A form with no matching handlers | `wire` is generated empty. |

## Try it

```bash
cargo run -p chassis --example project -- path/to/project
```

prints the project's name, its resource counts, each form with its handler file, and the handlers found in it.

To see the rules work on a copy of your project, add a handler stub to a form with:

```bash
cargo run -p chassis --example codegen -- path/to/project MainWindow "pub fn on_close() {}"
```

The example appends the stub to the form's handler file and lists the handlers. Running it again with the same function is refused, because the function already exists.

To see which handlers wire to which widgets, and write the wiring file:

```bash
cargo run -p chassis --example wiring -- path/to/project MainWindow
```

The example prints each wired handler with its widget, event, and engine method, then writes `src/generated/main_window_wiring.rs`.

## The command line

The `chassis` command regenerates the generated code without opening the designer, and checks a project for problems. It calls the same function the designer does (`chassis::cli::render`), so the files are byte for byte the same.

```mermaid
flowchart LR
    R["app.prsc"] --> X["chassis::cli::render"]
    H["src/forms/*.rs"] --> X
    X --> G["chassis generate<br/>writes src/generated/"]
    X --> C["chassis check<br/>compares, writes nothing"]
```

```bash
cargo run -p chassis -- generate path/to/project
cargo run -p chassis -- check path/to/project
```

The project defaults to the current directory. `generate` writes `src/generated/ids.rs`, one `<form>_wiring.rs` per form, and `src/generated/mod.rs` (the module list), and prints each path. Nothing is written if any file cannot be produced.

`check` writes nothing. It prints one line per problem and exits non-zero if there are any:

| Kind | Meaning |
|---|---|
| `invalid` | A name that is not a usable Rust identifier, a handler that is not `pub`, a handler that could mean two widgets or events, or a project with no resource file. |
| `orphaned` | A function named `on_...` that matches no widget and event of its form (for example, its widget was deleted), or a file in `src/forms` that belongs to no form. |
| `missing` | A generated file that does not exist or is out of date, so a handler is not wired. Run `chassis generate`. |

| Exit code | Meaning |
|---|---|
| `0` | Success; `check` found nothing. |
| `1` | `check` found problems. |
| `2` | Bad arguments, or the project could not be loaded or written. |

Use `chassis check` in CI to fail a build whose generated code is out of date. A `build.rs` can call `chassis::cli::generate` (or run the command) to regenerate before compiling.
