#!/usr/bin/env python3
"""Rename code2prompt -> gnaw across source + config. Dry-run by default; --apply to write.
Skips Cargo.lock, *.lock, *.bak, target/, .git/, and *.md (attribution/third-party links).
Run from repo root. KEEP THIS FILE OUTSIDE THE REPO or it rewrites itself."""
import sys
from pathlib import Path

REPO = Path.cwd()

# Ordered, plain substring replace, top-to-bottom: compound identifiers first so
# the bare `code2prompt` rule can't chew into them.
REPLACEMENTS = [
    ("code2prompt_core",         "gnaw_core"),
    ("code2prompt-python",       "gnaw-python"),
    ("code2prompt_rs",           "gnaw"),
    ("code2prompt-rs",           "gnaw"),
    # PascalCase API names — comment this block out to keep the old type names.
    ("Code2PromptConfigBuilder", "GnawConfigBuilder"),
    ("Code2PromptConfig",        "GnawConfig"),
    ("Code2PromptSession",       "GnawSession"),
    ("Code2promptSession",       "GnawSession"),  # original's casing typo
    ("Code2Prompt",              "Gnaw"),
    ("code2prompt",              "gnaw"),          # bare: bin, config dir, prose
]

CONTENT_SUFFIXES = {".rs", ".toml", ".py", ".hbs"}
CONTENT_NAMES    = {"Cargo.toml", "pyproject.toml"}
SKIP_DIRS        = {".git", "target", "node_modules", "__pycache__"}
SKIP_NAMES       = {"Cargo.lock", Path(__file__).name}
SKIP_SUFFIXES    = {".bak", ".lock", ".md"}

def rewrite(s):
    for old, new in REPLACEMENTS:
        s = s.replace(old, new)
    return s

def is_content(p):
    if p.name in SKIP_NAMES or p.suffix in SKIP_SUFFIXES:
        return False
    return p.suffix in CONTENT_SUFFIXES or p.name in CONTENT_NAMES

def walk():
    for p in REPO.rglob("*"):
        if any(part in SKIP_DIRS for part in p.relative_to(REPO).parts):
            continue
        yield p

def main():
    apply = "--apply" in sys.argv[1:]
    edited = 0
    for p in walk():
        if not (p.is_file() and is_content(p)):
            continue
        try:
            old = p.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        new = rewrite(old)
        if new != old:
            edited += 1
            print(f"[edit] {p.relative_to(REPO)}")
            if apply:
                p.write_text(new, encoding="utf-8")

    renames = [(p, p.with_name(rewrite(p.name))) for p in walk()
               if rewrite(p.name) != p.name and p.suffix not in SKIP_SUFFIXES
               and p.name not in SKIP_NAMES]
    for src, dst in sorted(renames, key=lambda t: len(t[0].parts), reverse=True):
        print(f"[move] {src.relative_to(REPO)} -> {dst.name}")
        if apply:
            src.rename(dst)

    print(f"\n{'APPLIED' if apply else 'DRY RUN'}: {edited} edited, {len(renames)} renamed"
          + ("" if apply else "  (re-run with --apply)"))

if __name__ == "__main__":
    main()
