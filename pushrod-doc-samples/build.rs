//! Build script: extracts every documentation sample marked `rust checked` into `OUT_DIR`.
//!
//! A sample is a fenced code block in `pushrod-docs/docs/**/*.md(x)` whose info string starts
//! with `rust checked`, for example ```` ```rust checked title="src/main.rs" ````.  Each one
//! becomes a module in the generated `samples.rs`, so a sample that no longer compiles against
//! the library fails `cargo test`.

use std::fs;
use std::path::{Path, PathBuf};

/// Collects every `.md` and `.mdx` file below `dir`, in sorted order.
///
/// # Arguments
///
/// * `dir` - The directory to search.
/// * `out` - Receives the paths found.
fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("md" | "mdx")
        ) {
            out.push(path);
        }
    }
}

/// Returns the bodies of the `rust checked` fences in `text`.
///
/// # Arguments
///
/// * `text` - The contents of one documentation page.
///
/// # Returns
///
/// The code of each checked sample, in page order.
fn checked_samples(text: &str) -> Vec<String> {
    let mut samples = Vec::new();
    let mut current: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim_start();

        match current.as_mut() {
            Some(code) if trimmed.starts_with("```") => {
                samples.push(std::mem::take(code));
                current = None;
            }
            Some(code) => {
                code.push_str(line);
                code.push('\n');
            }
            None if trimmed.starts_with("```rust checked") => current = Some(String::new()),
            None => {}
        }
    }

    samples
}

fn main() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../pushrod-docs/docs");
    println!("cargo:rerun-if-changed={}", docs.display());

    let mut files = Vec::new();
    collect(&docs, &mut files);

    let mut generated = String::new();
    let mut count = 0;

    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        let text = fs::read_to_string(&file).unwrap_or_default();

        for sample in checked_samples(&text) {
            count += 1;
            generated.push_str(&format!(
                "/// Sample {count} from `{}`.\n#[allow(dead_code, unused)]\npub mod sample_{count} {{\n{sample}}}\n\n",
                file.file_name().and_then(|n| n.to_str()).unwrap_or("?")
            ));
        }
    }

    generated.push_str(&format!(
        "/// How many samples were checked.\npub const CHECKED_SAMPLES: usize = {count};\n"
    ));
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("samples.rs");
    fs::write(out, generated).unwrap();
}
