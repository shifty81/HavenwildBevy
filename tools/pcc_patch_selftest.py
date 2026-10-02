#!/usr/bin/env python3
"""Adversarial, non-mutating regression checks for governed PCC patch validation."""
from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PCC_PATH = ROOT / "ProjectControlCenter.py"

spec = importlib.util.spec_from_file_location("havenwild_pcc_selftest", PCC_PATH)
if spec is None or spec.loader is None:
    raise RuntimeError("Unable to load ProjectControlCenter.py")
pcc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pcc)

CURRENT = pcc.installed_patch_state()


def entry(path: str, data: bytes, *, size: int | None = None, sha: str | None = None) -> dict:
    return {
        "path": path,
        "bytes": len(data) if size is None else size,
        "sha256": hashlib.sha256(data).hexdigest() if sha is None else sha,
    }


def manifest(
    patch_id: str,
    files: list[dict],
    *,
    source_from: dict[str, str] | None = None,
    delete: list[str] | None = None,
) -> dict:
    return {
        "schema": "havenwild.bevy.pcc.patch.v1",
        "id": patch_id,
        "description": "PCC validator self-test fixture; never applied",
        "from": dict(CURRENT if source_from is None else source_from),
        "to": {"source": "selftest", "forgepy": "selftest", "pcc": "selftest"},
        "files": files,
        "delete": list(delete or []),
    }


def archive(root: Path, name: str, doc: dict, files: dict[str, bytes]) -> Path:
    path = root / name
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("pcc_patch.json", json.dumps(doc))
        for rel, data in files.items():
            z.writestr(rel, data)
    return path


def expect_accept(path: Path) -> None:
    pcc.validate_patch_archive(path)


def expect_reject(path: Path, contains: str) -> None:
    try:
        pcc.validate_patch_archive(path)
    except Exception as exc:
        text = str(exc)
        if contains.lower() not in text.lower():
            raise AssertionError(f"wrong rejection for {path.name}: {text!r}") from exc
        return
    raise AssertionError(f"validator accepted unsafe fixture {path.name}")



def expect_manifest_first_discovery(root: Path, payload: bytes) -> None:
    original_root = pcc.ROOT
    original_inbox = pcc.UPDATE_INBOX
    try:
        pcc.ROOT = root
        pcc.UPDATE_INBOX = root / "updates" / "inbox"
        pcc.UPDATE_INBOX.mkdir(parents=True, exist_ok=True)
        valid_entry = entry("docs/discovery.txt", payload)
        renamed = archive(
            root,
            "Havenwild patch download.pccpatch (1).zip",
            manifest("SelfTest-RenamedDiscovery", [valid_entry]),
            {"docs/discovery.txt": payload},
        )
        ordinary = root / "ordinary-source-rollup.zip"
        with zipfile.ZipFile(ordinary, "w", zipfile.ZIP_DEFLATED) as z:
            z.writestr("README.md", "not a patch\n")
        found = pcc.discover_updates()
        if renamed not in found:
            raise AssertionError("manifest-first root discovery missed a browser-renamed governed patch ZIP")
        if ordinary in found:
            raise AssertionError("manifest-first root discovery misclassified an ordinary ZIP as a patch")
    finally:
        pcc.ROOT = original_root
        pcc.UPDATE_INBOX = original_inbox


def expect_manual_overwrite_relation(root: Path, payload: bytes) -> None:
    current = dict(CURRENT)
    doc = manifest("SelfTest-ManualOverwrite", [entry("docs/manual.txt", payload)])
    doc["to"] = dict(current)
    relation, _ = pcc.patch_target_state(doc)
    if relation != "target-current":
        raise AssertionError(f"manual-overwrite relation not recognized: {relation}")

    original_root = pcc.ROOT
    try:
        pcc.ROOT = root
        target = root / "docs" / "manual.txt"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(payload)
        matches, reason = pcc.patch_payload_matches_installed(doc)
        if not matches:
            raise AssertionError(f"manual-overwrite payload should reconcile: {reason}")
        target.write_bytes(b"drifted\n")
        matches, _ = pcc.patch_payload_matches_installed(doc)
        if matches:
            raise AssertionError("manual-overwrite reconciliation accepted drifted bytes")
    finally:
        pcc.ROOT = original_root

def main() -> int:
    payload = b"validator-selftest\n"
    with tempfile.TemporaryDirectory(prefix="havenwild-pcc-selftest-") as tmp:
        root = Path(tmp)
        valid_entry = entry("docs/selftest-safe.txt", payload)
        valid = archive(root, "valid.pccpatch.zip", manifest("SelfTest-Valid", [valid_entry]), {"docs/selftest-safe.txt": payload})
        expect_accept(valid)

        wrong = dict(CURRENT)
        wrong["source"] = "not-installed"
        wrong_from = archive(root, "wrong-from.pccpatch.zip", manifest("SelfTest-WrongFrom", [valid_entry], source_from=wrong), {"docs/selftest-safe.txt": payload})
        expect_reject(wrong_from, "not compatible")

        bad_size_entry = entry("docs/selftest-size.txt", payload, size=len(payload) + 1)
        bad_size = archive(root, "bad-size.pccpatch.zip", manifest("SelfTest-BadSize", [bad_size_entry]), {"docs/selftest-size.txt": payload})
        expect_reject(bad_size, "byte-count mismatch")

        no_hash_entry = entry("docs/selftest-nohash.txt", payload, sha="")
        no_hash = archive(root, "no-hash.pccpatch.zip", manifest("SelfTest-NoHash", [no_hash_entry]), {"docs/selftest-nohash.txt": payload})
        expect_reject(no_hash, "full sha256")

        case_entries = [entry("docs/Foo.txt", b"a"), entry("docs/foo.txt", b"b")]
        case_collision = archive(root, "case-collision.pccpatch.zip", manifest("SelfTest-Case", case_entries), {"docs/Foo.txt": b"a", "docs/foo.txt": b"b"})
        expect_reject(case_collision, "colliding")

        reserved_entry = entry("docs/CON/file.txt", payload)
        reserved = archive(root, "reserved.pccpatch.zip", manifest("SelfTest-Reserved", [reserved_entry]), {"docs/CON/file.txt": payload})
        expect_reject(reserved, "reserved device")

        ads_entry = entry("docs/file.txt:stream", payload)
        ads = archive(root, "ads.pccpatch.zip", manifest("SelfTest-Ads", [ads_entry]), {"docs/file.txt:stream": payload})
        expect_reject(ads, "invalid path segment")

        trailing_entry = entry("docs/ambiguous./file.txt", payload)
        trailing = archive(root, "trailing-dot.pccpatch.zip", manifest("SelfTest-Trailing", [trailing_entry]), {"docs/ambiguous./file.txt": payload})
        expect_reject(trailing, "ambiguous")

        asset_entry = entry("assets/elizawy/forbidden.png", payload)
        forbidden_asset = archive(
            root,
            "forbidden-asset.pccpatch.zip",
            manifest("SelfTest-AssetBoundary", [asset_entry]),
            {"assets/elizawy/forbidden.png": payload},
        )
        expect_reject(forbidden_asset, "forbidden source path")

        collision_entry = entry("docs/collision.txt", payload)
        write_delete_collision = archive(
            root,
            "write-delete-collision.pccpatch.zip",
            manifest("SelfTest-WriteDelete", [collision_entry], delete=["DOCS/COLLISION.TXT"]),
            {"docs/collision.txt": payload},
        )
        expect_reject(write_delete_collision, "both write and delete")

        extra = archive(
            root,
            "undeclared-extra.pccpatch.zip",
            manifest("SelfTest-Undeclared", [valid_entry]),
            {"docs/selftest-safe.txt": payload, "docs/extra.txt": b"extra\n"},
        )
        expect_reject(extra, "undeclared files")

        discovery_root = root / "discovery"
        discovery_root.mkdir()
        expect_manifest_first_discovery(discovery_root, payload)
        manual_root = root / "manual"
        manual_root.mkdir()
        expect_manual_overwrite_relation(manual_root, payload)

    print("PCC PATCH VALIDATOR SELF-TEST: PASS (valid fixture accepted; 10 unsafe classes rejected; manifest-first renamed-ZIP discovery; manual-overwrite reconciliation)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
