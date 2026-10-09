#!/usr/bin/env python3
"""Guarded manual installation/reconciliation of M2D097C same-version PCC patch.

This exists because PCC 1.3.5 correctly refuses *different* target bytes when
from.source == to.source. It does not bypass or modify PCC validation. Instead,
it installs the exact archived payload only when src/world_doc.rs still matches
the version captured by the user's evidence bundle. PCC can then reconcile
matching target bytes on the next patch scan.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tempfile
from zipfile import ZipFile

EXPECTED_PATCH = "Havenwild_Bevy_M2D097C_Barrel_Test_Production_Lookup_From_0.10.19.pccpatch.zip"
EXPECTED_ID = "HW-BEVY-M2D097C-BARREL-LOOKUP-GATE"
EXPECTED_VERSION = {"source": "0.10.19", "forgepy": "0.4.10", "pcc": "1.3.5"}
EXPECTED_PREIMAGE_SHA256 = "6feb8ee5eb78b7a38cddfc2142f9a587c4eea527461f895a12aab1cc239aa409"
EXPECTED_TARGET_SHA256 = "450b477b1988409ba9fdc6a49c4cfeeb7bacc80c7e4bd2ba39447bb344830db0"
EXPECTED_PATHS = {
    "src/world_doc.rs",
    "docs/patch/M2D097C_BARREL_TEMPLATE_LOOKUP_GATE_REPAIR.md",
}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def die(message: str) -> None:
    raise ValueError(message)


def atomic_write(path: Path, data: bytes) -> None:
    """Atomically replace the file on the same volume; preserve mode on replace."""
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".m2d097c-", suffix=".tmp", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as f:
            f.write(data)
            f.flush()
            os.fsync(f.fileno())
        if path.exists():
            shutil.copymode(path, tmp)
        os.replace(tmp, path)
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)


def load_checked_patch(patch_path: Path) -> dict[str, bytes]:
    if not patch_path.is_file():
        die(f"Required original governed patch is missing: {patch_path}\n"
            "Leave the original .pccpatch.zip in HWlatest; do not extract it.")
    with ZipFile(patch_path) as z:
        if z.testzip() is not None:
            die("CRC/integrity check failed for the governed patch")
        if set(z.namelist()) != EXPECTED_PATHS | {"pcc_patch.json"}:
            die("Unexpected payload paths in the governed patch; refusing to apply")
        manifest = json.loads(z.read("pcc_patch.json"))
        if (manifest.get("schema") != "havenwild.bevy.pcc.patch.v1"
            or manifest.get("id") != EXPECTED_ID
            or manifest.get("from") != EXPECTED_VERSION
            or manifest.get("to") != EXPECTED_VERSION
            or manifest.get("delete") != []):
            die("Governed patch identity/version/schema does not match M2D097C")
        entries = manifest.get("files")
        if not isinstance(entries, list) or {e.get("path") for e in entries} != EXPECTED_PATHS or len(entries) != 2:
            die("Governed patch file declarations differ from expected")
        output = {}
        for e in entries:
            name = e["path"]
            payload = z.read(name)
            if (len(payload) != e.get("bytes") or digest(payload) != e.get("sha256")):
                die(f"Governed patch bytes/hash invalid for {name}")
            output[name] = payload
        if digest(output["src/world_doc.rs"]) != EXPECTED_TARGET_SHA256:
            die("Unexpected Rust source target hash; refusing a different patch")
        return output


def run(root: Path, patch: Path, *, check_only: bool = False) -> int:
    root = root.resolve()
    print("HAVENWILD M2D097C GUARDED MANUAL RECONCILIATION")
    print("ROOT:", root)
    cargo = root / "Cargo.toml"
    if not cargo.is_file():
        die(f"Not a Havenwild project root (no Cargo.toml): {root}")
    cargo_text = cargo.read_text(encoding="utf-8", errors="replace")
    match = re.search(r'(?m)^version\s*=\s*"([^"]+)"', cargo_text)
    if not match or match.group(1) != EXPECTED_VERSION["source"]:
        die(f"Project version mismatch: expected 0.10.19, found {match.group(1) if match else 'unknown'}")
    contents = load_checked_patch(patch)
    old_path = root / "src/world_doc.rs"
    doc_path = root / "docs/patch/M2D097C_BARREL_TEMPLATE_LOOKUP_GATE_REPAIR.md"
    if not old_path.is_file():
        die("Missing src/world_doc.rs")
    current = old_path.read_bytes()
    old_sha = digest(current)
    doc = contents["docs/patch/M2D097C_BARREL_TEMPLATE_LOOKUP_GATE_REPAIR.md"]
    print("CURRENT src/world_doc.rs SHA256:", old_sha)
    if old_sha == EXPECTED_TARGET_SHA256:
        already_corrected = True
        print("Rust file already contains the exact M2D097C correction.")
    elif old_sha == EXPECTED_PREIMAGE_SHA256:
        already_corrected = False
        print("Baseline SHA256 matches captured v0.10.19 source. Safe to install.")
    else:
        die("STOP: src/world_doc.rs differs from both the captured baseline and patch target.\n"
            "It may contain newer work or formatting changes. No files were modified.\n"
            "Provide an updated source-only ZIP for a new guarded patch; do not overwrite this file.")
    if doc_path.exists() and doc_path.read_bytes() != doc:
        die("STOP: docs/patch/M2D097C_BARREL_TEMPLATE_LOOKUP_GATE_REPAIR.md already exists with different bytes.\n"
            "No files were modified. Preserve the edited documentation.")
    if check_only:
        print("PRECHECK PASS (no files modified).")
        return 0

    stamp = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S-%f")
    rollback = root / ".pcc" / "rollback" / f"M2D097C_manual_reconcile_{stamp}"
    if not already_corrected:
        backup = rollback / "src/world_doc.rs"
        backup.parent.mkdir(parents=True, exist_ok=False)
        shutil.copy2(old_path, backup)
        if digest(backup.read_bytes()) != EXPECTED_PREIMAGE_SHA256:
            die("Source rollback backup failed checksum validation; no changes applied")
        atomic_write(old_path, contents["src/world_doc.rs"])
        print("REPLACED: src/world_doc.rs (validated exact captured source)")
        print("BACKUP:", backup)
    if not doc_path.exists():
        atomic_write(doc_path, doc)
        print("ADDED: docs/patch/M2D097C_BARREL_TEMPLATE_LOOKUP_GATE_REPAIR.md")
    for rel, payload in contents.items():
        actual = (root / rel).read_bytes()
        if actual != payload:
            die(f"STOP: post-apply byte mismatch: {rel}")
    receipt_path = root / ".pcc" / "receipts" / f"{stamp}_manual_reconcile_m2d097c.json"
    receipt_path.parent.mkdir(parents=True, exist_ok=True)
    receipt = {
        "schema": "havenwild.bevy.manual_reconcile_assistance.v1",
        "note": "Local guarded manual overwrite, NOT a PCC-certified installation receipt",
        "patchFile": patch.name,
        "patchId": EXPECTED_ID,
        "version": EXPECTED_VERSION,
        "sourceBaselineSha256": old_sha,
        "alreadyCorrected": already_corrected,
        "rollback": str(rollback.relative_to(root)) if not already_corrected else None,
        "targets": {rel: digest(data) for rel, data in contents.items()},
        "result": "exact_target_bytes_installed_for_pcc_reconciliation",
    }
    atomic_write(receipt_path, (json.dumps(receipt, indent=2) + "\n").encode())
    print("RECEIPT:", receipt_path)
    print("RESULT: PASS / all governed patch target bytes now match exactly")
    print("NEXT: Start PCC.cmd -> 3 Patch Scan / Review / Apply, then cargo test reviewed_barrel_template_uses_full_height_source_sprite, then FULL GATE.")
    print("NOTE: This local receipt is not PCC gate certification; the PCC must confirm reconciliation.")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=Path.cwd(), help="Havenwild HWlatest source root")
    ap.add_argument("--patch", type=Path, default=None, help="Path to existing pending M2D097C .pccpatch.zip")
    ap.add_argument("--check", action="store_true", help="Check compatibility only; do not write files")
    args = ap.parse_args()
    root = args.root.resolve()
    patch = args.patch.resolve() if args.patch else (root / EXPECTED_PATCH)
    try:
        return run(root, patch, check_only=args.check)
    except (OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print("RESULT: BLOCKED / " + str(exc), file=sys.stderr)
        return 5


if __name__ == "__main__":
    sys.exit(main())
