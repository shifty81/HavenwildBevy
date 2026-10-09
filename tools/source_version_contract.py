from __future__ import annotations

import re
from typing import Iterable

_VERSION_RE = re.compile(r'^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', re.M)


def cargo_source_version(cargo_text: str) -> str:
    match = _VERSION_RE.search(cargo_text)
    if not match:
        raise ValueError('Cargo package version missing')
    return '.'.join(match.groups())


def version_tuple(version: str) -> tuple[int, int, int]:
    parts = version.split('.')
    if len(parts) != 3 or any(not part.isdigit() for part in parts):
        raise ValueError(f'invalid source version: {version!r}')
    return tuple(int(part) for part in parts)  # type: ignore[return-value]


def source_at_least(cargo_text: str, minimum: str) -> bool:
    return version_tuple(cargo_source_version(cargo_text)) >= version_tuple(minimum)


def project_matches_cargo(project: dict, cargo_text: str) -> bool:
    return project.get('sourceVersion') == cargo_source_version(cargo_text)


def studio_title_uses_cargo_version(main_text: str) -> bool:
    """Return True when the Bevy window title visibly derives from CARGO_PKG_VERSION.

    Keep this semantic instead of pinning the exact Rust format! source shape so
    optional diagnostic suffixes can evolve without breaking historical gates.
    """
    match = re.search(
        r'title\s*:\s*format!\(\s*"(?P<fmt>Havenwild — Bevy Studio v[^"]*)"\s*,(?P<args>[\s\S]{0,320})',
        main_text,
    )
    if not match:
        return False
    fmt = match.group('fmt')
    args = match.group('args')
    return 'v{}' in fmt and 'env!("CARGO_PKG_VERSION")' in args
