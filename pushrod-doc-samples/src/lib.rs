//! Compiles the documentation samples marked `rust checked` in `pushrod-docs/docs`.
//!
//! Nothing here is public API: the crate exists so `cargo test --workspace` fails when a checked
//! sample on the documentation site stops compiling.  See "Writing documentation" on the site.

include!(concat!(env!("OUT_DIR"), "/samples.rs"));

#[cfg(test)]
mod tests {
    /// The documentation must contain at least one checked sample, or the check is not running.
    #[test]
    fn docs_have_checked_samples() {
        assert!(
            super::CHECKED_SAMPLES > 0,
            "no `rust checked` samples found in pushrod-docs/docs"
        );
    }
}
