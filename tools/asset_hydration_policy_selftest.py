#!/usr/bin/env python3
"""Regression test for Havenwild's GitHub-first core asset hydration policy."""
from __future__ import annotations

import importlib.util
import hashlib
from pathlib import Path
import sys
import json
import tempfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("havenwild_forgepy_asset_policy", ROOT / "ForgePY.py")
assert spec and spec.loader
forge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(forge)


def run_case(upstream_rc: int, *, local_only: bool = False):
    events: list[str] = []
    old_refresh = forge.refresh_assets_from_upstream
    old_resolve = forge.resolve_asset_source
    try:
        def fake_refresh():
            events.append("github")
            return upstream_rc

        def fake_resolve(_explicit):
            events.append("local")
            return None

        forge.refresh_assets_from_upstream = fake_refresh
        forge.resolve_asset_source = fake_resolve
        rc = forge.sync_assets(None, local_only=local_only)
        return rc, events
    finally:
        forge.refresh_assets_from_upstream = old_refresh
        forge.resolve_asset_source = old_resolve


def main() -> int:
    rc, events = run_case(0)
    assert rc == 0, (rc, events)
    assert events == ["github"], events

    rc, events = run_case(3)
    assert rc == 2, (rc, events)
    assert events == ["github", "local"], events

    rc, events = run_case(0, local_only=True)
    assert rc == 2, (rc, events)
    assert events == ["local"], events

    assert forge.ELIZAWY_UPSTREAM_COMMIT == "f07f7f5892e67c932c68f70bb04472f2c64e46bc"

    # Historical core manifests were captured from a Windows checkout where Git
    # materialized text files with CRLF. GitHub raw/blob endpoints serve the
    # repository's LF bytes. Prove that LF Credits.txt blobs are accepted only
    # when their CRLF-canonicalized identity matches the checked-in manifest.
    upstream_lf = b"alpha\nbeta\ngamma\n"
    historical_crlf = upstream_lf.replace(b"\n", b"\r\n")
    text_entry = {
        "path": "Terrain/Credits.txt",
        "bytes": len(historical_crlf),
        "sha256": hashlib.sha256(historical_crlf).hexdigest(),
    }
    ok, mode = forge._manifest_data_matches(upstream_lf, text_entry)
    assert ok and mode == "git-text-eol", (ok, mode)

    tampered = b"alpha\nBETA\ngamma\n"
    ok, mode = forge._manifest_data_matches(tampered, text_entry)
    assert not ok, (ok, mode)

    binary = b"\x89PNG\r\n\x1a\nfixture"
    binary_entry = {
        "path": "Terrain/fixture.png",
        "bytes": len(binary),
        "sha256": hashlib.sha256(binary).hexdigest(),
    }
    ok, mode = forge._manifest_data_matches(binary, binary_entry)
    assert ok and mode == "exact", (ok, mode)
    ok, _ = forge._manifest_data_matches(binary + b"x", binary_entry)
    assert not ok


    # Full Gate readiness must distinguish "active sheets exist" from
    # "the complete core manifest exists". A partial tree is repair-needed
    # even when its currently-used runtime sheets are present.
    old_asset_root = forge.ASSET_ROOT
    old_active = forge.ACTIVE_SHEETS
    old_manifest_entries = forge._manifest_entries
    try:
        with tempfile.TemporaryDirectory(prefix="havenwild-asset-complete-") as td:
            root = Path(td)
            forge.ASSET_ROOT = root / "assets"
            forge.ASSET_ROOT.mkdir(parents=True)
            forge.ACTIVE_SHEETS = root / "active.json"
            forge.ACTIVE_SHEETS.write_text(json.dumps({"sheets": []}), encoding="utf-8")
            fixture_manifest = [
                {"path": "Terrain/a.png", "bytes": 1, "sha256": hashlib.sha256(b"a").hexdigest()},
                {"path": "Terrain/Credits.txt", "bytes": 3, "sha256": hashlib.sha256(b"x\r\n").hexdigest()},
            ]
            forge._manifest_entries = lambda: fixture_manifest
            (forge.ASSET_ROOT / "Terrain").mkdir(parents=True)
            (forge.ASSET_ROOT / "Terrain/a.png").write_bytes(b"a")
            assert forge.asset_status(require_complete=True) == 3
            (forge.ASSET_ROOT / "Terrain/Credits.txt").write_bytes(b"x\n")
            assert forge.asset_status(require_complete=True) == 0
            assert forge.asset_status(full_verify=True) == 0
    finally:
        forge.ASSET_ROOT = old_asset_root
        forge.ACTIVE_SHEETS = old_active
        forge._manifest_entries = old_manifest_entries

    print("ASSET HYDRATION POLICY SELFTEST: PASS / pinned GitHub first; partial-manifest repair guard; immediate identity verification; exact binary identity; Git text LF accepted against historical Windows-CRLF manifest; local fallback preserved")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
