#!/usr/bin/env python3
"""Restore verified original byte identity of ElizaWy Credits.txt files.

The only permitted transformation is LF -> CRLF, and ONLY when the resulting
bytes exactly match BOTH byte count and SHA-256 from the pinned core manifest.
Dry-run is the default. No PNG/GIF/source manifest bytes are modified. ``--apply``
creates local backups and replaces files atomically, rolling back on errors.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
from pathlib import Path, PurePosixPath
import tempfile

MANIFEST = Path("content/catalog/core_source_manifest.json")
ART_ROOT = Path("assets/elizawy")
BACKUP_ROOT = Path(".forgepy/asset_authority_recovery/credits_crlf")


class RecoveryError(RuntimeError):
    """Fail closed on mismatched provenance or unsafe filesystem state."""


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_credit_path(value: str) -> Path:
    parsed = PurePosixPath(value)
    if (parsed.is_absolute() or not parsed.parts or value != parsed.as_posix() or
            any(part in ("", ".", "..") or ":" in part or "\\" in part for part in parsed.parts) or
            parsed.name != "Credits.txt"):
        raise RecoveryError(f"unsafe credit path in manifest: {value!r}")
    return Path(*parsed.parts)


def assert_real_file(root: Path, rel: Path) -> Path:
    destination = root / rel
    current = root
    for part in rel.parts:
        current = current / part
        if current.is_symlink():
            raise RecoveryError(f"symlinked path is prohibited: {current}")
    if not destination.is_file():
        raise RecoveryError(f"missing credit source: {destination}")
    return destination


def plan(root: Path) -> tuple[list[dict], list[dict]]:
    manifest_path = root / MANIFEST
    if not manifest_path.is_file() or manifest_path.is_symlink():
        raise RecoveryError("core manifest missing or symlinked")
    source = json.loads(manifest_path.read_text(encoding="utf-8"))
    if source.get("schema") != "havenwild.bevy.asset_manifest.v1":
        raise RecoveryError("unexpected manifest schema")
    rows: list[dict] = []
    conflicts: list[dict] = []
    seen: set[str] = set()
    for entry in source.get("entries", []):
        value = entry.get("path", "")
        if not isinstance(value, str) or not value.endswith("/Credits.txt"):
            continue
        rel = safe_credit_path(value)
        key = rel.as_posix().casefold()
        if key in seen:
            raise RecoveryError(f"duplicate credit in manifest: {value}")
        seen.add(key)
        path = assert_real_file(root, ART_ROOT / rel)
        old = path.read_bytes()
        wanted_sha = entry.get("sha256")
        wanted_size = entry.get("bytes")
        if (not isinstance(wanted_size, int) or isinstance(wanted_size, bool) or
                not isinstance(wanted_sha, str) or not re.fullmatch(r"[0-9a-f]{64}", wanted_sha)):
            raise RecoveryError(f"invalid byte identity for {value}")
        if digest(old) == wanted_sha and len(old) == wanted_size:
            status, new = "VERIFIED", old
        else:
            # Never guess at text content or normalize a partially changed source.
            new = old.replace(b"\n", b"\r\n") if b"\r" not in old else old
            status = ("RESTORABLE_CRLF" if len(new) == wanted_size and
                      digest(new) == wanted_sha and new != old else "CONFLICT")
        row = {"sourcePath": value, "status": status, "actualSha256": digest(old),
               "expectedSha256": wanted_sha, "actualBytes": len(old), "expectedBytes": wanted_size,
               "_old": old, "_new": new, "_path": path, "_rel": rel}
        if status == "CONFLICT":
            conflicts.append(row)
        rows.append(row)
    if not rows:
        raise RecoveryError("manifest has no Credits.txt entries")
    return rows, conflicts


def atomic_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, staged = tempfile.mkstemp(prefix=".hw-credit-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(staged, path)
    finally:
        if os.path.exists(staged):
            os.unlink(staged)


def apply(root: Path, rows: list[dict]) -> int:
    changes = [row for row in rows if row["status"] == "RESTORABLE_CRLF"]
    if not changes:
        return 0
    # Stage every backup before touching the source files.
    for row in changes:
        target = root / BACKUP_ROOT / row["_rel"]
        if not target.is_relative_to(root) or target.is_symlink() or any(
            parent.is_symlink() for parent in (root, *[p for p in target.parents if p.is_relative_to(root)])
        ):
            raise RecoveryError(f"symlinked backup path prohibited: {target}")
        if target.exists():
            if target.read_bytes() != row["_old"]:
                raise RecoveryError(f"existing backup has different bytes: {target}")
        else:
            atomic_write(target, row["_old"])
    completed: list[dict] = []
    try:
        for row in changes:
            if row["_path"].read_bytes() != row["_old"]:
                raise RecoveryError(f"source changed since plan: {row['sourcePath']}")
            atomic_write(row["_path"], row["_new"])
            completed.append(row)
            if (digest(row["_path"].read_bytes()) != row["expectedSha256"] or
                    row["_path"].stat().st_size != row["expectedBytes"]):
                raise RecoveryError(f"postwrite identity mismatch: {row['sourcePath']}")
    except Exception:
        for row in reversed(completed):
            atomic_write(row["_path"], row["_old"])
        raise
    return len(changes)


def run(root: Path, *, execute: bool = False, report_path: Path | None = None) -> dict:
    root = root.resolve(strict=True)
    rows, conflicts = plan(root)
    count = {status: sum(row["status"] == status for row in rows)
             for status in ("VERIFIED", "RESTORABLE_CRLF", "CONFLICT")}
    report = {"schema": "havenwild.elizawy.credit_identity_recovery.v1",
              "mode": "apply" if execute else "dry_run", "counts": count,
              "files": [{k: v for k, v in row.items() if not k.startswith("_")} for row in rows],
              "status": "BLOCKED_CONFLICT" if conflicts else "READY"}
    if execute and not conflicts:
        repaired = apply(root, rows)
        verified_rows, still_conflicted = plan(root)
        if still_conflicted or any(row["status"] != "VERIFIED" for row in verified_rows):
            raise RecoveryError("post-repair credit manifest verification failed")
        report["status"] = "PASS_VERIFIED"
        report["repairedCount"] = repaired
    if report_path:
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=Path("."))
    ap.add_argument("--apply", action="store_true", help="Explicitly restore pinned original CRLF bytes")
    ap.add_argument("--report", type=Path, help="Write a JSON audit report")
    args = ap.parse_args()
    try:
        report = run(args.root, execute=args.apply, report_path=args.report)
    except (OSError, ValueError, RecoveryError) as exc:
        print("CREDIT IDENTITY RECOVERY: ERROR -", exc)
        return 3
    print(f"CREDIT IDENTITY RECOVERY: {report['status']} / {report['mode']}")
    print("  entries:", report["counts"])
    if report.get("repairedCount") is not None:
        print("  verified repaired credits:", report["repairedCount"])
    if report["status"] == "BLOCKED_CONFLICT":
        print("  Unrecognized text changes detected; no credits were rewritten.")
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
