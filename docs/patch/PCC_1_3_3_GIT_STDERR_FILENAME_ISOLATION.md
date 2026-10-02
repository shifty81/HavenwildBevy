# PCC 1.3.3 — Git stderr / filename stream isolation

## Problem

On Windows with `core.autocrlf` enabled, Git can write line-ending warnings to stderr while `git diff --name-only -z` writes NUL-delimited filenames to stdout. PCC 1.3.2 merged the two byte streams, allowing a warning plus `Cargo.lock` to be interpreted as one fake path and causing certified staging to fail.

## Correction

- Machine-readable Git byte queries now capture stdout and stderr separately and parse stdout only.
- The Git workflow self-test enables `core.autocrlf=true` and proves a warning-producing `Cargo.lock` change resolves to exactly `Cargo.lock`.
- `.gitattributes`, `.gitignore`, and `Cargo.lock` are explicitly normalized to LF in repository history.
- Human-facing Git commands continue to stream stderr/stdout into PCC logs normally.

This does not weaken source certification, path filtering, remote divergence checks, or the canonical-history adoption policy.
