"""Comment and code extraction for the gated file set.

Two gates stand on this module: the comment-reference gate, which classifies
what a comment says, and the panic-index gate, which reads code with its
comments and string literals removed. Both need the same thing first — a
scanner that knows where a comment ends and a string begins — so there is one
scanner and not two.

WHAT THE SCANNERS SEE, stated rather than assumed, because the reach of a
lexical scanner is the whole of what its gate can decide:

Rust: line comments, doc comments and nested block comments are found by a
character scan that understands string literals, byte strings, raw strings with
any number of hashes, and character literals. A lifetime is not mistaken for a
character literal: an apostrophe is consumed as a literal only when a closing
apostrophe follows the one escape or one character after it.

Shell: a hash opens a comment only at a word start outside quoting, so a
parameter expansion of the argument count and a hash inside a quoted string are
not comments. Single and double quotes carry across lines, and a here-document
body is skipped to its terminator. What this scanner does NOT resolve is the
shell grammar itself — a hash inside an unquoted case pattern or inside an
arithmetic expansion reads as a comment here. Both are refusals a reader can see
and rewrite; neither has appeared in this tree.

YAML: a hash opens a comment at a line start or after whitespace outside quotes,
and a block scalar's body is skipped by indentation, so shell written under a
literal block is content rather than comment. A stream with more than one
document is refused rather than guessed at.

SQL: a double hyphen opens a line comment and a slash-star a block comment, both
outside single-quoted strings and dollar-quoted bodies.

Makefile: the shell scanner's rules, with the recipe's leading tab respected.

Gitignore: a hash opens a comment at column zero only, which is that format's
own rule; an escaped hash is a pattern.
"""

from __future__ import annotations

import os


class UnsupportedSource(Exception):
    """The scanner refuses a source shape it cannot place comments in."""


def language_of(path: str) -> str | None:
    """Return the scanner name for a path, or None when the path is not gated."""
    base = os.path.basename(path)
    if base == "Makefile" or base.endswith(".mk"):
        return "makefile"
    if base == ".gitignore":
        return "gitignore"
    ext = os.path.splitext(base)[1]
    return {
        ".rs": "rust",
        ".sh": "shell",
        ".bash": "shell",
        ".yml": "yaml",
        ".yaml": "yaml",
        ".sql": "sql",
    }.get(ext)


def extract_comments(path: str, text: str) -> list[tuple[int, str]]:
    """Return every comment in text as (1-based line number, comment text)."""
    lang = language_of(path)
    if lang is None:
        return []
    if lang == "rust":
        return _rust_comments(text)
    if lang in ("shell", "makefile"):
        return _shell_comments(text, makefile=(lang == "makefile"))
    if lang == "yaml":
        return _yaml_comments(text)
    if lang == "sql":
        return _sql_comments(text)
    if lang == "gitignore":
        return _gitignore_comments(text)
    return []


def rust_code(text: str) -> str:
    """Return text with every comment and string body replaced by spaces.

    Line structure is preserved: a line number in the result is the line number
    in the source, which is what lets a caller report a finding's coordinate.
    """
    out = []
    for ch, kind in _rust_scan(text):
        if kind == "code" or ch == "\n":
            out.append(ch)
        else:
            out.append(" ")
    return "".join(out)


# --- Rust --------------------------------------------------------------------


def _rust_scan(text: str):
    """Yield (character, kind) for every character, kind in code/comment/string."""
    i, n = 0, len(text)
    while i < n:
        ch = text[i]
        # Raw string: an optional b, an r, any number of hashes, then a quote.
        raw = _raw_string_open(text, i)
        if raw is not None:
            start, hashes = raw
            close = '"' + "#" * hashes
            end = text.find(close, start)
            end = n if end == -1 else end + len(close)
            for j in range(i, end):
                yield text[j], ("code" if text[j] == "\n" else "string")
            i = end
            continue
        if ch == '"' or (ch == "b" and text[i : i + 2] == 'b"'):
            j = i + (2 if ch == "b" else 1)
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            for k in range(i, min(j, n)):
                yield text[k], ("code" if text[k] == "\n" else "string")
            i = j
            continue
        if ch == "'":
            end = _char_literal_end(text, i)
            if end is not None:
                for k in range(i, end):
                    yield text[k], "string"
                i = end
                continue
            yield ch, "code"
            i += 1
            continue
        if text[i : i + 2] == "//":
            j = text.find("\n", i)
            j = n if j == -1 else j
            for k in range(i, j):
                yield text[k], "comment"
            i = j
            continue
        if text[i : i + 2] == "/*":
            depth, j = 1, i + 2
            while j < n and depth:
                if text[j : j + 2] == "/*":
                    depth += 1
                    j += 2
                    continue
                if text[j : j + 2] == "*/":
                    depth -= 1
                    j += 2
                    continue
                j += 1
            for k in range(i, j):
                yield text[k], ("code" if text[k] == "\n" else "comment")
            i = j
            continue
        yield ch, "code"
        i += 1


def _raw_string_open(text: str, i: int):
    """Return (index after the opening quote, hash count) for a raw string at i."""
    j = i
    if text[j : j + 1] == "b":
        j += 1
    if text[j : j + 1] != "r":
        return None
    j += 1
    hashes = 0
    while text[j : j + 1] == "#":
        hashes += 1
        j += 1
    if text[j : j + 1] != '"':
        return None
    return j + 1, hashes


def _char_literal_end(text: str, i: int):
    """Return the index past a character literal at i, or None for a lifetime."""
    if text[i : i + 2] == "'\\":
        j = i + 2
        while j < len(text) and text[j] != "'":
            if text[j] == "\n":
                return None
            j += 1
        return j + 1 if j < len(text) else None
    if len(text) > i + 2 and text[i + 2] == "'":
        return i + 3
    return None


def _rust_comments(text: str) -> list[tuple[int, str]]:
    out: list[tuple[int, str]] = []
    line, buf, buf_line = 1, [], None
    for ch, kind in _rust_scan(text):
        if kind == "comment":
            if buf_line is None:
                buf_line = line
            buf.append(ch)
        else:
            if buf_line is not None:
                out.append((buf_line, _strip_rust_marker("".join(buf))))
                buf, buf_line = [], None
        if ch == "\n":
            line += 1
    if buf_line is not None:
        out.append((buf_line, _strip_rust_marker("".join(buf))))
    return out


def _strip_rust_marker(raw: str) -> str:
    body = raw
    for marker in ("///", "//!", "//", "/*!", "/**", "/*"):
        if body.startswith(marker):
            body = body[len(marker) :]
            break
    if body.endswith("*/"):
        body = body[:-2]
    return body.strip()


# --- Shell and Makefile ------------------------------------------------------


def _shell_comments(text: str, makefile: bool = False) -> list[tuple[int, str]]:
    out: list[tuple[int, str]] = []
    lines = text.split("\n")
    quote = ""          # "'" or '"' while a quoted run carries across lines
    heredocs: list[tuple[str, bool]] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        lineno = i + 1
        if heredocs:
            term, dash = heredocs[0]
            probe = line.lstrip("\t") if dash else line
            if probe.strip() == term:
                heredocs.pop(0)
            i += 1
            continue
        comment, quote, opened = _shell_scan_line(line, quote, makefile)
        if comment is not None:
            out.append((lineno, comment))
        heredocs.extend(opened)
        i += 1
    return out


def _shell_scan_line(line: str, quote: str, makefile: bool):
    """Scan one line: return (comment text or None, trailing quote state, heredocs)."""
    opened: list[tuple[str, bool]] = []
    j, n = 0, len(line)
    prev = " "
    while j < n:
        ch = line[j]
        if quote:
            if ch == "\\" and quote == '"':
                j += 2
                prev = "x"
                continue
            if ch == quote:
                quote = ""
            j += 1
            prev = "x"
            continue
        if ch == "\\":
            j += 2
            prev = "x"
            continue
        if ch in ("'", '"'):
            quote = ch
            j += 1
            prev = "x"
            continue
        if line[j : j + 2] == "<<" and line[j : j + 3] != "<<<":
            j, opened_here = _heredoc_word(line, j)
            if opened_here:
                opened.append(opened_here)
            prev = "x"
            continue
        if ch == "#" and prev in (" ", "\t", ";", "&", "|", "(", ""):
            body = line[j + 1 :]
            if makefile and line[:j].strip() == "" and line.startswith("\t"):
                # A recipe line's comment is the shell's, and reads the same.
                pass
            return body.strip(), quote, opened
        prev = ch
        j += 1
    return None, quote, opened


def _heredoc_word(line: str, j: int):
    """Consume a here-document operator; return (next index, (terminator, dash))."""
    j += 2
    dash = False
    if line[j : j + 1] == "-":
        dash = True
        j += 1
    while line[j : j + 1] in (" ", "\t"):
        j += 1
    quote = ""
    if line[j : j + 1] in ("'", '"'):
        quote = line[j]
        j += 1
    start = j
    while j < len(line) and (line[j].isalnum() or line[j] in "_-." or (quote and line[j] != quote)):
        j += 1
    word = line[start:j]
    if quote and line[j : j + 1] == quote:
        j += 1
    return j, ((word, dash) if word else None)


# --- YAML --------------------------------------------------------------------


def _yaml_comments(text: str) -> list[tuple[int, str]]:
    lines = text.split("\n")
    for n, line in enumerate(lines, 1):
        if line.startswith("---") and line.strip() not in ("---",):
            continue
        if line.startswith("%") or (line.strip() == "..." and n > 1):
            raise UnsupportedSource(
                f"line {n} opens a directive or a second document; this scanner reads one plain document"
            )
    out: list[tuple[int, str]] = []
    block_indent: int | None = None
    for n, line in enumerate(lines, 1):
        stripped = line.strip()
        indent = len(line) - len(line.lstrip(" "))
        if block_indent is not None:
            if stripped == "" or indent > block_indent:
                continue
            block_indent = None
        comment = _yaml_comment_of(line)
        if comment is not None:
            out.append((n, comment))
        if _opens_block_scalar(line):
            block_indent = indent
    return out


def _opens_block_scalar(line: str) -> bool:
    body = _yaml_strip_comment(line).rstrip()
    if not body:
        return False
    tail = body.split(":")[-1].strip() if ":" in body else body.strip()
    if tail.startswith("- "):
        tail = tail[2:].strip()
    return tail.startswith("|") or tail.startswith(">")


def _yaml_comment_of(line: str):
    quote = ""
    for j, ch in enumerate(line):
        if quote:
            if ch == quote:
                quote = ""
            continue
        if ch in ("'", '"'):
            quote = ch
            continue
        if ch == "#" and (j == 0 or line[j - 1] in (" ", "\t")):
            return line[j + 1 :].strip()
    return None


def _yaml_strip_comment(line: str) -> str:
    comment = _yaml_comment_of(line)
    if comment is None:
        return line
    return line[: line.rindex("#" + comment) if ("#" + comment) in line else len(line)]


# --- SQL ---------------------------------------------------------------------


def _sql_comments(text: str) -> list[tuple[int, str]]:
    out: list[tuple[int, str]] = []
    i, n, line = 0, len(text), 1
    while i < n:
        ch = text[i]
        if ch == "'":
            i += 1
            while i < n and text[i] != "'":
                if text[i] == "\n":
                    line += 1
                i += 1
            i += 1
            continue
        if text[i : i + 2] == "--":
            j = text.find("\n", i)
            j = n if j == -1 else j
            out.append((line, text[i + 2 : j].strip()))
            i = j
            continue
        if text[i : i + 2] == "/*":
            j = text.find("*/", i + 2)
            j = n if j == -1 else j + 2
            body = text[i + 2 : max(i + 2, j - 2)]
            out.append((line, " ".join(body.split())))
            line += text.count("\n", i, j)
            i = j
            continue
        if ch == "\n":
            line += 1
        i += 1
    return out


# --- Gitignore ---------------------------------------------------------------


def _gitignore_comments(text: str) -> list[tuple[int, str]]:
    out = []
    for n, line in enumerate(text.split("\n"), 1):
        if line.startswith("#"):
            out.append((n, line[1:].strip()))
    return out
