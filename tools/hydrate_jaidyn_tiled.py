#!/usr/bin/env python3
"""Hydrate the pinned JaidynReiman configured LPC/Tiled derivative locally.

Third-party art/TSX are never embedded in Havenwild source packages. This tool
fetches or accepts the already-downloaded OGA ZIP, verifies the exact audited
archive and member hashes, and installs the 10 configured files under the Bevy
asset root at assets/elizawy/_jaidyn_tiled.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import tempfile
import urllib.request
import zipfile

URL = "https://opengameart.org/sites/default/files/lpc-revised-exterior-tilesets.zip"
ARCHIVE_SHA256 = "f0236c16b272a7328dad66772e4f971513bc508cb69a56c60d736264a3a5b43a"
TARGET_REL = Path("assets/elizawy/_jaidyn_tiled")
MAX_ARCHIVE_BYTES = 32 * 1024 * 1024
MAX_MEMBER_BYTES = 8 * 1024 * 1024
MEMBERS = {
    "lpc-tileset-terrain-summer.tsx": "6261982fad28ee031196cc97aa0f6bdd13f49499acc0edba9aa61c59a4b538bb",
    "lpc-tileset-terrain-summer.png": "15dc6b9a1c904a4b9944555baca93cdf622e117a88ceb52a5a5b32eab06e2f2e",
    "lpc-tileset-terrain-spring.tsx": "ad08b18b3f11181f5ce62b5699c4499ebd9ed1144305822c7a7bd86e5796587e",
    "lpc-tileset-terrain-spring.png": "7f79da5bc88f04dc106dee933cd06fbf84518ecc8e4be67ef6765aa1c08ac572",
    "lpc-tileset-terrain-autumn-old.tsx": "77493afe99f6acdd8ef45eb76511db485c8feac7880bf746960f206a36629d49",
    "lpc-tileset-terrain-autumn.png": "e29277a97a0e4b177ea274f624c19a6076c25e6adf9289cd5a097f32346ad9ec",
    "lpc-tileset-terrain-winter.tsx": "cd90512ebefea37a69901e8c8914a551479e72fc3cb99732b7c8c899c94ecf16",
    "lpc-tileset-terrain-winter.png": "0b20228f655393cb782af0c5d21612ac412d605063d44d1311432a7c4788f915",
    "lpc-tileset-buildings.tsx": "1aabcbeb74ab9985ce4ea576465a9989350d6ae6d17fa3415ad94ba6c3e854f5",
    "lpc-tileset-buildings.png": "e16b9537e0cf6cf4870d99d2050695fb0cbb1804c481c77cbf97ae4402bb5e8c",
}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def safe_member(name: str) -> bool:
    p = PurePosixPath(name.replace("\\", "/"))
    return not p.is_absolute() and ".." not in p.parts and not (p.parts and ":" in p.parts[0])


def acquire(archive: Path | None) -> tuple[Path, bool]:
    if archive is not None:
        path = archive.resolve()
        if not path.is_file():
            raise RuntimeError(f"Archive does not exist: {path}")
        return path, False
    tmp = tempfile.NamedTemporaryFile(prefix="havenwild-jaidyn-", suffix=".zip", delete=False)
    tmp.close()
    path = Path(tmp.name)
    print(f"DOWNLOAD: {URL}")
    req = urllib.request.Request(
        URL,
        headers={"User-Agent": "Havenwild-Bevy-Jaidyn-Hydrator/1.0 (+OpenGameArt source hydration)"},
    )
    total = 0
    try:
        with urllib.request.urlopen(req, timeout=120) as response, path.open("wb") as out:
            advertised = response.headers.get("Content-Length")
            if advertised and int(advertised) > MAX_ARCHIVE_BYTES:
                raise RuntimeError(f"Server advertised archive larger than {MAX_ARCHIVE_BYTES} bytes")
            while True:
                chunk = response.read(1024 * 1024)
                if not chunk:
                    break
                total += len(chunk)
                if total > MAX_ARCHIVE_BYTES:
                    raise RuntimeError(f"Downloaded archive exceeds {MAX_ARCHIVE_BYTES} bytes")
                out.write(chunk)
        return path, True
    except Exception:
        path.unlink(missing_ok=True)
        raise


def write_attribution(path: Path) -> None:
    path.write_text(
        "Configured LPC Revised four-season Tiled tilesets by JaidynReiman.\n"
        "Source: https://opengameart.org/content/lpc-revised-fully-configured-4-seasons-tilesets-for-tiled-map-editor\n"
        "Licenses listed by the release: CC-BY 3.0 and OGA-BY 3.0.\n\n"
        "JaidynReiman compiled the LPC Revised assets, configured the Tiled Map Editor tilesets, "
        "and created a winter version of the bushes with a simple palette recolor.\n"
        "Original LPC Revised contributors listed by the release include Eliza Wyatt (DeathsDarling), "
        "Lanea Zimmerman (Sharm), Stephen Challener (Redshrike), Johannes Sjolund (Wulax), "
        "BlueCarrot16, BenCreating, Durrani, YuriNikolai and Craftpix.net 2D Game Assets.\n"
        "Upstream credits: https://github.com/ElizaWy/LPC/blob/main/Credits.txt\n"
        "Havenwild must preserve its existing per-source attribution/provenance records as well.\n",
        encoding="utf-8",
    )


def hydrate(root: Path, archive: Path | None, target: Path | None = None) -> Path:
    root = root.resolve()
    target = (target or root / TARGET_REL).resolve()
    source, temporary = acquire(archive)
    staged: Path | None = None
    backup: Path | None = None
    try:
        size = source.stat().st_size
        if size > MAX_ARCHIVE_BYTES:
            raise RuntimeError(f"Jaidyn archive is unexpectedly large: {size} bytes")
        actual_archive = sha256_file(source)
        if actual_archive != ARCHIVE_SHA256:
            raise RuntimeError(
                f"Jaidyn archive SHA mismatch: expected {ARCHIVE_SHA256}, found {actual_archive}"
            )
        with zipfile.ZipFile(source) as zf:
            index: dict[str, str] = {}
            for info in zf.infolist():
                if info.is_dir():
                    continue
                if not safe_member(info.filename):
                    raise RuntimeError(f"Unsafe ZIP member: {info.filename}")
                if info.file_size > MAX_MEMBER_BYTES:
                    raise RuntimeError(f"ZIP member too large: {info.filename} ({info.file_size} bytes)")
                base = PurePosixPath(info.filename.replace("\\", "/")).name
                if base in MEMBERS:
                    if base in index:
                        raise RuntimeError(f"Duplicate required member basename: {base}")
                    index[base] = info.filename
            missing = sorted(set(MEMBERS) - set(index))
            if missing:
                raise RuntimeError(f"Pinned archive is missing required members: {missing}")

            target.parent.mkdir(parents=True, exist_ok=True)
            staged = Path(tempfile.mkdtemp(prefix=f".{target.name}.staging-", dir=target.parent))
            for base, expected in MEMBERS.items():
                data = zf.read(index[base])
                actual = sha256_bytes(data)
                if actual != expected:
                    raise RuntimeError(
                        f"Member SHA mismatch for {base}: expected {expected}, found {actual}"
                    )
                (staged / base).write_bytes(data)

            manifest = {
                "schema": "havenwild.external.jaidyn_tiled_runtime.v2",
                "authority": "configured_tiled_topology",
                "archiveSha256": ARCHIVE_SHA256,
                "sourceUrl": URL,
                "license": ["CC-BY-3.0", "OGA-BY-3.0"],
                "configuredBy": "JaidynReiman",
                "files": {name: MEMBERS[name] for name in sorted(MEMBERS)},
                "policy": {
                    "worldStateAuthority": "Havenwild",
                    "wangTopologyAuthority": "JaidynReiman TSX via rs-tiled",
                    "collision": "provisional_until_havenwild_validated",
                    "autumn": "partial_metadata_fail_closed",
                    "transforms": "must_pass_m2d094b_acceptance_before_runtime_promotion",
                    "probability": "must_pass_m2d094b_acceptance_before_runtime_promotion",
                },
            }
            (staged / "havenwild.external.jaidyn_tiled_runtime.v2.json").write_text(
                json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
            )
            write_attribution(staged / "ATTRIBUTION.txt")

            if target.exists():
                backup = target.with_name(target.name + ".previous")
                if backup.exists():
                    shutil.rmtree(backup)
                target.replace(backup)
            try:
                staged.replace(target)
                staged = None
            except Exception:
                if backup is not None and backup.exists() and not target.exists():
                    backup.replace(target)
                raise
            if backup is not None and backup.exists():
                # The new verified target is already authoritative. Failure to
                # delete a stale backup should not turn a successful hydration
                # into an ambiguous rollback state.
                shutil.rmtree(backup, ignore_errors=True)
                backup = None
    finally:
        if staged is not None and staged.exists():
            shutil.rmtree(staged, ignore_errors=True)
        if temporary:
            source.unlink(missing_ok=True)
    print(f"JAIDYN TILED HYDRATION: PASS / {len(MEMBERS)} pinned files / {target}")
    return target



def check_existing(root: Path, target: Path | None = None) -> Path:
    root = root.resolve()
    target = (target or root / TARGET_REL).resolve()
    manifest_path = target / "havenwild.external.jaidyn_tiled_runtime.v2.json"
    if not manifest_path.is_file():
        raise RuntimeError(f"Hydration manifest missing: {manifest_path}")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema") != "havenwild.external.jaidyn_tiled_runtime.v2":
        raise RuntimeError("Hydration manifest schema mismatch")
    if manifest.get("archiveSha256") != ARCHIVE_SHA256:
        raise RuntimeError("Hydration manifest archive SHA mismatch")
    if manifest.get("files") != {name: MEMBERS[name] for name in sorted(MEMBERS)}:
        raise RuntimeError("Hydration manifest file hash table mismatch")
    for name, expected in MEMBERS.items():
        path = target / name
        if not path.is_file():
            raise RuntimeError(f"Hydrated file missing: {name}")
        actual = sha256_file(path)
        if actual != expected:
            raise RuntimeError(f"Hydrated file SHA mismatch for {name}: {actual}")
    if not (target / "ATTRIBUTION.txt").is_file():
        raise RuntimeError("Hydrated attribution file is missing")
    print(f"JAIDYN TILED HYDRATION CHECK: PASS / {len(MEMBERS)} pinned files / {target}")
    return target

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path.cwd())
    ap.add_argument("--archive", type=Path, help="Use an already-downloaded pinned OGA ZIP")
    ap.add_argument("--target", type=Path, help="Override local hydration target")
    ap.add_argument("--check", action="store_true", help="Verify an existing hydrated target without downloading or changing files")
    args = ap.parse_args()
    if args.check:
        check_existing(args.root, args.target)
    else:
        hydrate(args.root, args.archive, args.target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
