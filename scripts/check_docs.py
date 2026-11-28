#!/usr/bin/env python3
"""Checks that every non-private Rust item in the library crates has a full doc header.

Rules (see CONTRIBUTING.md, "Code documentation"):

* Every non-private item (`pub`, `pub(crate)`, `pub(super)`, ...) -- function, struct, enum,
  trait, type alias, constant, static -- has a `///` doc comment.
* Every function declared in a trait, and every function in a trait implementation, has one too.
* A documented function lists each parameter (other than `self`) under `# Arguments` as
  ``* `name` - ...``, and has a `# Returns` section if it returns a value.

Private items (no `pub`, outside trait blocks) are not checked.  Run from the repository root:

    python3 scripts/check_docs.py
"""

import pathlib
import re
import sys

ROOTS = ["src", "pushrod-widgets/src", "chassis/src"]
ITEM = re.compile(
    r"^\s*pub(\([^)]*\))?\s+(?:const\s+|unsafe\s+|async\s+|extern\s+\"[^\"]*\"\s+)*"
    r"(fn|struct|enum|trait|type|const|static|union)\b"
)
FN = re.compile(r"^\s*(?:pub(\([^)]*\))?\s+)?(?:const\s+|unsafe\s+|async\s+)*fn\s+(\w+)")
TRAIT_OPEN = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(unsafe\s+)?trait\s+\w+")
IMPL_OPEN = re.compile(r"^\s*(unsafe\s+)?impl\b.*\bfor\b")


def strip_code(line):
    """Removes comments, string literals, and char literals so braces can be counted."""
    line = re.sub(r'"(\\.|[^"\\])*"', '""', line)
    line = re.sub(r"'(\\.|[^'\\])'", "''", line)
    return line.split("//", 1)[0]


def doc_block(lines, n):
    """Returns the /// lines directly above line n, skipping attributes."""
    i = n - 1
    while i >= 0 and lines[i].strip().startswith("#["):
        i -= 1
    block = []
    while i >= 0 and lines[i].strip().startswith("///"):
        block.insert(0, lines[i].strip()[3:].strip())
        i -= 1
    return block


def signature(lines, n):
    """Joins a possibly multi-line item signature up to its opening brace or semicolon."""
    sig = ""
    for line in lines[n:n + 40]:
        code = strip_code(line)
        sig += " " + code.strip()
        if "{" in code or code.rstrip().endswith(";"):
            break
    return sig.split("{", 1)[0]


def params(sig):
    """Returns the parameter names of a function signature, excluding self."""
    start = sig.find("(", re.search(r"\bfn\s+\w+", sig).end())
    depth, end = 0, None
    for i in range(start, len(sig)):
        if sig[i] in "(<[":
            depth += 1
        elif sig[i] in ")>]":
            depth -= 1
            if depth == 0 and sig[i] == ")":
                end = i
                break
    inner = sig[start + 1:end]
    parts, depth, cur = [], 0, ""
    for ch in inner:
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    parts.append(cur)
    names = []
    for p in parts:
        p = p.strip()
        if not p or re.match(r"^(&\s*('\w+\s+)?)?(mut\s+)?self\b", p):
            continue
        name = p.split(":", 1)[0].replace("mut ", "").strip()
        names.append(name)
    return names, sig[end + 1:] if end else ""


def check_file(path):
    problems = []
    lines = path.read_text().split("\n")
    stack = []  # (kind, depth) for trait / trait-impl blocks
    depth = 0
    for n, line in enumerate(lines):
        code = strip_code(line)
        stripped = line.strip()
        in_block = stack and stack[-1][1] == depth
        is_fn = FN.match(line)
        needs_doc = bool(ITEM.match(line)) or (is_fn and in_block and stack[-1][0] in ("trait", "impl"))
        if needs_doc:
            block = doc_block(lines, n)
            where = f"{path}:{n + 1}"
            if not block:
                problems.append(f"{where}: missing doc comment: {stripped}")
            elif is_fn:
                text = "\n".join(block)
                names, rest = params(signature(lines, n))
                if names and "# Arguments" not in text:
                    problems.append(f"{where}: no `# Arguments` section for {', '.join(names)}")
                for name in names:
                    if "# Arguments" in text and f"`{name}`" not in text:
                        problems.append(f"{where}: parameter `{name}` not described under `# Arguments`")
                if "->" in rest and "# Returns" not in text:
                    problems.append(f"{where}: returns a value but has no `# Returns` section")
        opens, closes = code.count("{"), code.count("}")
        if opens:
            if TRAIT_OPEN.match(line) and (stripped.startswith("pub") or "trait" in stripped.split()[0:2]):
                is_pub = stripped.startswith("pub")
                stack.append(("trait" if is_pub else "private-trait", depth + 1))
            elif IMPL_OPEN.match(line):
                stack.append(("impl", depth + 1))
        depth += opens - closes
        while stack and depth < stack[-1][1]:
            stack.pop()
    return problems


def main():
    problems = []
    for root in ROOTS:
        for path in sorted(pathlib.Path(root).rglob("*.rs")):
            problems += check_file(path)
    for p in problems:
        print(p)
    if problems:
        print(f"\n{len(problems)} documentation problem(s). See CONTRIBUTING.md, \"Code documentation\".")
        return 1
    print("Documentation headers: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
