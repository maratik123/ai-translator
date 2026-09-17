"""The comment-reference gate: no comment in a gated file points outward.

The rule and its exemptions live in the documentation convention's fourth
section; this is the mechanical half of it, and it decides only what a lexical
pass can decide. A comment says what the thing is, states its call contract, and
carries what the toolchain requires. It names nothing outside itself that moves
when this tree moves.

Every class below is decided on the comment text alone, after the machine-read
directives are removed. A finding names the class and quotes what matched, so
the fix is always visible from the report: rewrite the sentence without the
outward pointer, or drop the sentence when the pointer was the whole of it.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from scan_sources import (  # noqa: E402  (path is set up above)
    UnsupportedSource,
    extract_comments,
    language_of,
)

RE_TODO_ISSUE = re.compile(r"\bTODO\(#[0-9]+\)")
RE_URL = re.compile(r"\bhttps?://\S+")
RE_LOCATOR = re.compile(r"\b[A-Za-z0-9_./-]*[./][A-Za-z0-9_./-]*:[0-9]+\b")
RE_MARKDOWN = re.compile(r"\S*\.md\b")
RE_AC_ID = re.compile(r"\bAC[0-9]+\b")
RE_DECISION = re.compile(r"\b(?:KD-[0-9]+|D[0-9]+)\b")
RE_REGISTER_ID = re.compile(r"\bS?R[0-9]+-[0-9]+\b")
RE_SECTION = re.compile(r"§")
RE_ISSUE = re.compile(r"#[0-9]+")
RE_PATH_TOKEN = re.compile(r"[A-Za-z0-9_./-]+")
RE_CRATE_SYMBOL = re.compile(r"\b([a-z][a-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*)")

GATED_SOURCE_EXTS = (".rs", ".sh", ".sql", ".yml", ".yaml")
GATED_ROOT_NAMES = {"Makefile", ".gitignore"}
REPO_TOP_DIRS = {
    "crates",
    "docs",
    "ai-docs",
    "examples",
    "models",
    "tools",
    ".claude",
    ".github",
    ".githooks",
}

# A directive the toolchain reads is exempt in full, because it is addressed to
# a program and not to a reader. The human reason text a lint suppression
# carries is classified like any other prose, since it rots like any other
# prose.
DIRECTIVE_PREFIXES = ("shellcheck", "!", "yamllint", "cargo:", "SPDX-License-Identifier")


def usage() -> None:
    print(
        "Usage:\n"
        "  comment_refs.py                 every tracked file of the gated set\n"
        "  comment_refs.py --staged        the staged files of the gated set\n"
        "  comment_refs.py <path> [...]    the named files"
    )


def strip_directive(text: str) -> str | None:
    """Return the text to classify, or None when the comment is exempt in full."""
    trimmed = text.lstrip(" \t")
    if trimmed.startswith("nolint:") or trimmed.startswith("allow("):
        rest = trimmed.split(":", 1)[-1]
        parts = rest.split(None, 1)
        return parts[1] if len(parts) > 1 else None
    for prefix in DIRECTIVE_PREFIXES:
        if trimmed.startswith(prefix):
            return None
    return text


def blank_out(match: re.Match) -> str:
    return " " * len(match.group(0))


def is_repo_path_token(token: str) -> bool:
    if token in GATED_ROOT_NAMES:
        return True
    if token.endswith(GATED_SOURCE_EXTS):
        return True
    head = token.split("/", 1)[0]
    return "/" in token and head in REPO_TOP_DIRS


def workspace_crates(root: str) -> set[str]:
    """The crate names this workspace declares, read off the tree.

    While no crate exists the set is empty and the crate-symbol class is inert,
    which is the honest state: there is no symbol of this workspace to point at.
    """
    names: set[str] = set()
    crates_dir = os.path.join(root, "crates")
    if not os.path.isdir(crates_dir):
        return names
    for entry in sorted(os.listdir(crates_dir)):
        manifest = os.path.join(crates_dir, entry, "Cargo.toml")
        if not os.path.isfile(manifest):
            continue
        with open(manifest, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                m = re.match(r'\s*name\s*=\s*"([^"]+)"', line)
                if m:
                    names.add(m.group(1).replace("-", "_"))
                    break
        names.add(entry.replace("-", "_"))
    return names


def own_crate(path: str) -> str:
    parts = path.split("/")
    if len(parts) > 2 and parts[0] == "crates":
        return parts[1].replace("-", "_")
    return ""


def classify(text: str, own: str, crates: set[str]) -> list[tuple[str, str]]:
    body = strip_directive(text)
    if body is None:
        return []
    body = RE_TODO_ISSUE.sub(blank_out, body)

    findings: list[tuple[str, str]] = []
    for cls, rx in (("url", RE_URL), ("locator", RE_LOCATOR), ("markdown-path", RE_MARKDOWN)):
        for m in rx.findall(body):
            findings.append((cls, m))
        body = rx.sub(blank_out, body)

    for cls, rx in (
        ("ac-id", RE_AC_ID),
        ("decision-anchor", RE_DECISION),
        ("review-register-id", RE_REGISTER_ID),
        ("section", RE_SECTION),
        ("issue", RE_ISSUE),
    ):
        for m in rx.findall(body):
            findings.append((cls, m))

    for token in RE_PATH_TOKEN.findall(body):
        if is_repo_path_token(token):
            findings.append(("repo-path", token))

    for crate, symbol in RE_CRATE_SYMBOL.findall(body):
        if crate != own and crate in crates:
            findings.append(("crate-symbol", f"{crate}::{symbol}"))

    return findings


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
            ["git", "ls-files"], capture_output=True, text=True, check=True
        ).stdout
    return [line for line in out.splitlines() if line]


def main(argv: list[str]) -> int:
    if argv and argv[0] in ("-h", "--help"):
        usage()
        return 0
    root = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True
    ).stdout.strip()
    if root:
        os.chdir(root)
    crates = workspace_crates(root or ".")

    findings = 0
    unreadable = 0
    for path in git_files(argv):
        if language_of(path) is None:
            continue
        if os.path.islink(path) or not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8", errors="replace") as fh:
            text = fh.read()
        try:
            comments = extract_comments(path, text)
        except UnsupportedSource as exc:
            print(f"{path}: cannot be scanned: {exc}", file=sys.stderr)
            unreadable += 1
            continue
        own = own_crate(path)
        for line, comment in comments:
            for cls, matched in classify(comment, own, crates):
                print(f"{path}:{line}: {cls}: {matched}")
                findings += 1

    if unreadable:
        print(
            f"comment-refs: {unreadable} file(s) could not be scanned; that is an instrument "
            "failure, not a clean result",
            file=sys.stderr,
        )
        return 1
    if findings:
        print(
            f"\ncomment-refs: {findings} outward reference(s). A comment says what the thing is "
            "and names nothing outside itself: rewrite the sentence without the pointer, or drop "
            "the sentence when the pointer was the whole of it.",
            file=sys.stderr,
        )
        return 1
    print("comment-refs: no comment in the gated set points outward")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
