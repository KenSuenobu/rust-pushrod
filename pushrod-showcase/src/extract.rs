// Pushrod
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Finds a page's `build` function in its source file.  The build script (`build.rs`) and the
//! tests share this file, so the snippet the showcase shows is exactly what the tests check.

/// Returns the text of the first `fn build(` function in a Rust source file, from the `fn`
/// keyword through its closing brace, with the indentation of the surrounding `impl` removed.
///
/// Braces inside strings, character literals, and comments are not counted.
///
/// # Arguments
///
/// * `source` - The text of a Rust source file.
///
/// # Returns
///
/// The function's text, or `None` when the file has no `build` function or its braces do not
/// balance.
pub fn build_fn(source: &str) -> Option<String> {
    let start = source.find("fn build(")?;
    let line_start = source[..start].rfind('\n').map_or(0, |at| at + 1);
    let indent = start - line_start;
    let end = closing_brace(source, start)?;
    let text = &source[start..=end];

    Some(
        text.lines()
            .enumerate()
            .map(|(n, line)| {
                // The first line starts at `fn`; the others still carry the impl's indent.
                if n == 0 {
                    line
                } else {
                    let spaces = line.len() - line.trim_start_matches(' ').len();

                    &line[spaces.min(indent)..]
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Returns the index of the `}` that closes the first `{` at or after `from`.
fn closing_brace(source: &str, from: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut i = from;

    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }

                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = source[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |at| i + 2 + at + 2);

                continue;
            }
            b'"' => {
                i += 1;

                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
            }
            // A character literal (`'{'`, `'\\n'`); a lifetime (`'_`) has no closing quote.
            b'\'' => {
                if bytes.get(i + 1) == Some(&b'\\') {
                    i = source[i + 2..]
                        .find('\'')
                        .map_or(bytes.len(), |at| i + 2 + at);
                } else if let Some(c) = source[i + 1..].chars().next()
                    && source[i + 1 + c.len_utf8()..].starts_with('\'')
                {
                    i += 1 + c.len_utf8() + 1;

                    continue;
                }
            }
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;

                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }

        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_build_function_and_removes_the_impl_indent() {
        let source = "impl Page for P {\n    fn name(&self) {}\n\n    fn build(&self, ui: &mut Ui<'_>) -> u32 {\n        let a = 1;\n        a\n    }\n\n    fn source(&self) {}\n}\n";

        assert_eq!(
            build_fn(source).unwrap(),
            "fn build(&self, ui: &mut Ui<'_>) -> u32 {\n    let a = 1;\n    a\n}"
        );
    }

    #[test]
    fn braces_in_strings_chars_and_comments_are_not_counted() {
        let source = "fn build() {\n    let s = \"}\\\"{\"; // }\n    let c = '}';\n    /* { */\n}\nfn after() {}";

        assert_eq!(build_fn(source).unwrap().lines().count(), 5);
        assert!(build_fn(source).unwrap().ends_with("*/\n}"));
    }

    #[test]
    fn a_missing_or_unbalanced_function_is_none() {
        assert!(build_fn("fn other() {}").is_none());
        assert!(build_fn("fn build() { {").is_none());
    }
}
