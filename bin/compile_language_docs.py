#!/usr/bin/env python3
"""
Compile every markdown document under docs/10_language into a single file.

Usage:
    python bin/compile_language_docs.py [output_file]

If output_file is omitted or "-", the combined content is written to stdout.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path


def collect_markdown_files(docs_dir: Path) -> list[Path]:
    """Return all markdown files under docs_dir sorted by relative path."""
    return sorted(
        (path for path in docs_dir.rglob("*.md") if path.is_file()),
        key=lambda p: p.relative_to(docs_dir),
    )


def build_compilation(md_files: list[Path], repo_root: Path) -> str:
    """Return a single markdown string containing every file's content."""
    sections: list[str] = []
    for path in md_files:
        rel_path = path.relative_to(repo_root)
        header = f"# File: {rel_path}\n\n"
        content = path.read_text(encoding="utf-8")
        sections.append(header + content.rstrip() + "\n")
    return "\n\n---\n\n".join(sections) + "\n"


def write_output(compiled: str, destination: Path | None) -> None:
    """Write compiled markdown to the destination or stdout."""
    if destination is None:
        sys.stdout.write(compiled)
        return

    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(compiled, encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compile docs/10_language markdown files into a single document."
    )
    parser.add_argument(
        "output",
        nargs="?",
        default="-",
        help="Path of the combined markdown file (default: stdout).",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    docs_dir = repo_root / "docs" / "10_language"

    if not docs_dir.exists():
        raise SystemExit(f"Missing docs directory: {docs_dir}")

    md_files = collect_markdown_files(docs_dir)
    if not md_files:
        raise SystemExit(f"No markdown files found under {docs_dir}")

    compiled = build_compilation(md_files, repo_root)
    destination = None if args.output == "-" else Path(args.output)
    write_output(compiled, destination)


if __name__ == "__main__":
    main()
