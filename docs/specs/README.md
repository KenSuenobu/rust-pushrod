# Pushrod specifications

Short written contracts for the parts of Pushrod that a widget library or a backend must honor.
Each guarantee is checked by a conformance test in [`tests/conformance/`](../../tests/conformance/),
which uses only the public API of the `pushrod` crate. An alternative widget library or backend
copies that directory and runs it to prove it keeps the same contracts.

| Spec | What it fixes | Conformance tests |
|---|---|---|
| [modules.md](modules.md) | The engine, the design layer, and the crates, and which may depend on which | `tests/conformance/boundaries.rs` |
| [widget.md](widget.md) | The widget core, identity, behavior, invalidation, and parts | `tests/conformance/widget.rs` |
| [events.md](events.md) | The event vocabulary, phases, masks, engine events, and ordering | `tests/conformance/events.rs` |
| [paint.md](paint.md) | The `Painter` trait and the cache policy | `tests/conformance/paint.rs` |
| [forms.md](forms.md) | Forms, modal dialogs, and the form file format | `tests/conformance/forms.rs` |

Run the suite with `cargo test --test conformance`; CI runs it with `cargo test --workspace`.

Words such as **must** and **may** are meant literally: a behavior marked **must** has a test.
Specs and tests use fictional sample data only.

When a change alters a guarantee, change the spec and its conformance test in the same pull
request. A guarantee without a test is marked *(not yet tested)*.
