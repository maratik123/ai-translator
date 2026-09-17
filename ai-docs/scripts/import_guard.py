"""The dependency-direction gate: a shipped binary never reaches a test-only crate.

WHAT IT DECIDES. For every binary target the workspace builds, the gate walks the
resolved dependency graph along normal and build edges — never dev edges — and
refuses a path that reaches one of the forbidden crates below. A dev-dependency
on the same crate is exactly what a test-only database is supposed to be, so the
distinction between the edge kinds is the whole rule and not an optimisation.

WHY IT EXISTS. The test database arrives through a container-runtime crate. That
crate pulls a client for the runtime's socket into whatever links it, and a
shipped binary that carries one grows a runtime dependency nobody declared, in a
deployment that has no container runtime at all. The failure is quiet: it builds,
it ships, and it surfaces as a start-up error on a machine no test ran on.

THE FORBIDDEN SET IS A TABLE, not a principle, and it is meant to be added to as
the workspace grows. Each row states the crate and the reason in one line, so a
reader can tell a rule that still holds from one that outlived its cause.

WHILE THE WORKSPACE IS EMPTY the gate reports a named skip and exits 0. Once a
workspace exists, every rule is enforced.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys

# crate name -> why a shipped binary must not reach it
FORBIDDEN = {
    "testcontainers": "starts and stops containers; the test database's business, not a shipped binary's",
    "testcontainers-modules": "the container images the suite provisions; test-only for the same reason",
}


def usage() -> None:
    print(
        "Usage:\n"
        "  import_guard.py    walk every binary target's non-dev dependency graph"
    )


def load_metadata() -> dict | None:
    if not os.path.isfile("Cargo.toml"):
        print(
            "import-guard: no Cargo.toml at the repository root; gate skipped",
            file=sys.stderr,
        )
        return None
    if shutil.which("cargo") is None:
        print("import-guard: cargo not on PATH; gate skipped", file=sys.stderr)
        return None
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked"],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        print(
            "import-guard: cargo metadata failed, so the graph could not be read. That is an "
            "instrument failure, not a clean graph:",
            file=sys.stderr,
        )
        print(proc.stderr.strip(), file=sys.stderr)
        sys.exit(1)
    return json.loads(proc.stdout)


def main(argv: list[str]) -> int:
    if argv and argv[0] in ("-h", "--help"):
        usage()
        return 0
    root = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True
    ).stdout.strip()
    if root:
        os.chdir(root)

    meta = load_metadata()
    if meta is None:
        return 0

    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta.get("resolve", {}).get("nodes", [])}
    members = set(meta.get("workspace_members", []))

    binaries = [
        pid
        for pid in members
        if any("bin" in t.get("kind", []) for t in packages[pid].get("targets", []))
    ]
    if not binaries:
        print("import-guard: the workspace builds no binary target yet; nothing to walk")
        return 0

    findings = 0
    for pid in sorted(binaries):
        name = packages[pid]["name"]
        for forbidden, why in sorted(FORBIDDEN.items()):
            chain = reaches(pid, forbidden, nodes, packages)
            if chain:
                print(f"{name}: reaches {forbidden} through {' -> '.join(chain)}")
                print(f"    {why}")
                findings += 1

    if findings:
        print(
            f"\nimport-guard: {findings} forbidden path(s). Move the dependency to "
            "dev-dependencies, or move the code that needs it into a crate the binary does not "
            "link.",
            file=sys.stderr,
        )
        return 1
    print(f"import-guard: {len(binaries)} binary target(s), no forbidden path")
    return 0


def reaches(start: str, forbidden: str, nodes: dict, packages: dict) -> list[str] | None:
    """Return the chain of crate names from start to forbidden, or None."""
    seen = {start}
    stack = [(start, [packages[start]["name"]])]
    while stack:
        pid, chain = stack.pop()
        node = nodes.get(pid)
        if node is None:
            continue
        for dep in node.get("deps", []):
            kinds = {k.get("kind") for k in dep.get("dep_kinds", [{"kind": None}])}
            # A None kind is a normal dependency; "build" links into the build
            # script, which still runs on the machine that builds the binary.
            if kinds and kinds <= {"dev"}:
                continue
            dep_id = dep["pkg"]
            dep_name = packages[dep_id]["name"] if dep_id in packages else dep.get("name", "?")
            if dep_name == forbidden:
                return chain + [dep_name]
            if dep_id in seen:
                continue
            seen.add(dep_id)
            stack.append((dep_id, chain + [dep_name]))
    return None


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
