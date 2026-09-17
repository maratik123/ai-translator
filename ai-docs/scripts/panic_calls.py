"""The panic gate: a panicking call in shipped code is marked, or it is refused.

WHAT COUNTS AS PANICKING: the panicking macros, and the two conversions that
panic on the value they are given. They are refused in compiled non-test source
because the paths this project ships — a translation request, a pipeline stage,
a database write — must degrade into an error the caller can render or retry. A
panic there loses a chapter's work in flight and takes the process with it.

THE ESCAPE IS A MARKER, NOT A SUPPRESSION. A call that genuinely cannot fire
carries `PANIC:` and its reason on its own line or on the line above, and the
reason is the justification itself — never a pointer at the index, which would
be the outward reference the comment convention bans. The index is the second
half and is kept by review: one row per surviving call, with the same
justification in prose.

WHAT THIS GATE DOES NOT SEE. Test code is excluded by position: a module behind
the test-configuration attribute, and every file under a test, benchmark or
example directory. A panicking call reached only from a test but written outside
one is refused like any other, which is the cheap direction — the marker is one
line.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from scan_sources import rust_code  # noqa: E402  (path is set up above)

PANICKING = (
    (re.compile(r"\bpanic!\s*\("), "panic!"),
    (re.compile(r"\btodo!\s*\("), "todo!"),
    (re.compile(r"\bunimplemented!\s*\("), "unimplemented!"),
    (re.compile(r"\bunreachable!\s*\("), "unreachable!"),
    (re.compile(r"\.unwrap\s*\("), ".unwrap()"),
    (re.compile(r"\.expect\s*\("), ".expect()"),
    (re.compile(r"\.unwrap_err\s*\("), ".unwrap_err()"),
    (re.compile(r"\.expect_err\s*\("), ".expect_err()"),
)

MARKER = re.compile(r"//[^\n]*\bPANIC:\s*\S")
EXCLUDED_DIRS = ("tests/", "benches/", "examples/")


def usage() -> None:
    print(
        "Usage:\n"
        "  panic_calls.py                 every tracked non-test Rust source\n"
        "  panic_calls.py --staged        the staged Rust sources\n"
        "  panic_calls.py <path> [...]    the named files"
    )


def is_excluded(path: str) -> bool:
    if os.path.basename(path) == "build.rs":
        return True
    return any(part in path.split("/") for part in ("tests", "benches", "examples"))


def test_module_spans(code: str) -> list[tuple[int, int]]:
    """Return the character spans of every module behind the test attribute."""
    spans: list[tuple[int, int]] = []
    for m in re.finditer(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", code):
        brace = code.find("{", m.end())
        if brace == -1:
            continue
        depth, i = 0, brace
        while i < len(code):
            if code[i] == "{":
                depth += 1
            elif code[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        spans.append((m.start(), i))
    return spans


def findings_for(path: str, text: str) -> list[tuple[int, str]]:
    code = rust_code(text)
    spans = test_module_spans(code)
    lines = text.split("\n")
    out: list[tuple[int, str]] = []
    for rx, name in PANICKING:
        for m in rx.finditer(code):
            if any(start <= m.start() <= end for start, end in spans):
                continue
            line = code.count("\n", 0, m.start()) + 1
            own = lines[line - 1] if line - 1 < len(lines) else ""
            above = lines[line - 2] if line >= 2 else ""
            if MARKER.search(own) or MARKER.search(above):
                continue
            out.append((line, name))
    return sorted(out)


def git_files(args: list[str]) -> list[str]:
    if args and args[0] == "--staged":
        out = subprocess.run(
            ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    elif args:
        return args
    else:
        out = subprocess.run(
            ["git", "ls-files", "*.rs"], capture_output=True, text=True, check=True
        ).stdout
    return [line for line in out.splitlines() if line.endswith(".rs")]


def main(argv: list[str]) -> int:
    if argv and argv[0] in ("-h", "--help"):
        usage()
        return 0
    root = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True
    ).stdout.strip()
    if root:
        os.chdir(root)

    total = 0
    for path in git_files(argv):
        if is_excluded(path) or not os.path.isfile(path) or os.path.islink(path):
            continue
        with open(path, encoding="utf-8", errors="replace") as fh:
            text = fh.read()
        for line, name in findings_for(path, text):
            print(f"{path}:{line}: {name} in shipped code")
            total += 1

    if total:
        print(
            f"\npanic-gate: {total} panicking call(s) outside test code. Return an error the "
            "caller can render or retry. A call that genuinely cannot fire carries PANIC: and its "
            "reason on its own line or the line above, states the justification in prose rather "
            "than pointing at the index, and gets its row in the index in the same commit.",
            file=sys.stderr,
        )
        return 1
    print("panic-gate: no unmarked panicking call in shipped code")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
