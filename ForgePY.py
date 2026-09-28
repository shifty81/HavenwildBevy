#!/usr/bin/env python3
"""Havenwild Bevy standalone project controller.

ForgePY owns local dependency/bootstrap workflow for this repository:
- asset discovery + immutable copy/hydration
- source catalog generation
- Rust/Cargo dependency fetch
- build/run/doctor
- live console capture + logs
- source-only packaging

The source repository intentionally does not vendor the ElizaWy/LPC Revised art.
Point ForgePY at an existing authoritative asset tree/archive staging folder once;
ForgePY copies validated assets into this project's local assets/ tree.
"""
from __future__ import annotations

import argparse
import datetime as _dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile
from typing import Iterable

ROOT = Path(__file__).resolve().parent
LOCAL_DIR = ROOT / ".forgepy"
LOG_DIR = LOCAL_DIR / "logs"
LOCAL_CONFIG = LOCAL_DIR / "local.json"
ASSET_ROOT = ROOT / "assets" / "elizawy"
OPTIONAL_ROOT = ROOT / "assets" / "optional"
REFERENCE_ROOT = ROOT / "reference"
CATALOG_ROOT = ROOT / "content" / "catalog"
CORE_MANIFEST = CATALOG_ROOT / "core_source_manifest.json"
ACTIVE_SHEETS = CATALOG_ROOT / "active_sheets.json"
GENERATED_INDEX = CATALOG_ROOT / "source_index.local.json"
CHARACTER_INDEX = CATALOG_ROOT / "characters_index.local.json"
ASSET_STATE = LOCAL_DIR / "asset_state.json"
PROJECT_META = ROOT / "project" / "forgepy.project.json"
PCC_ACTIVE_JOB = ROOT / ".pcc" / "active_job.json"
FORGEPY_VERSION = "0.4.6"

CORE_PACKS = ("Terrain.zip", "Structure.zip", "Objects.zip", "FX.zip")
OPTIONAL_ARCHIVES = ("Characters.zip", "FourSeasonAlternative.zip")
LEGACY_GENERATED_CATALOGS = {"content/catalog/source_index.json", "content/catalog/source_collisions.json"}


def _now() -> str:
    return _dt.datetime.now().astimezone().strftime("%Y%m%d-%H%M%S")


def _load_json(path: Path, default=None):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return default


def _save_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _png_size(path: Path):
    try:
        data = path.read_bytes()[:24]
    except OSError:
        return None
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return [int.from_bytes(data[16:20], "big"), int.from_bytes(data[20:24], "big")]


def _safe_rel(name: str) -> Path:
    p = Path(name.replace("\\", "/"))
    if p.is_absolute() or any(part in ("", ".", "..") for part in p.parts):
        raise ValueError(f"Unsafe archive path: {name!r}")
    return p


def _command_exists(name: str) -> bool:
    return shutil.which(name) is not None


def _stream(command: list[str], *, env=None, cwd=ROOT, label: str | None = None) -> int:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    label = label or Path(command[0]).stem
    log_path = LOG_DIR / f"{_now()}_{label}.log"
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    print("RUN:", subprocess.list2cmdline(command))
    print("LOG:", log_path)
    with log_path.open("w", encoding="utf-8", errors="replace") as log:
        log.write(f"ROOT: {ROOT}\n")
        log.write(f"RUN: {subprocess.list2cmdline(command)}\n\n")
        try:
            proc = subprocess.Popen(
                command,
                cwd=str(cwd),
                env=merged_env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
                errors="replace",
                bufsize=1,
            )
        except FileNotFoundError:
            print(f"ERROR: command not found: {command[0]}")
            return 127
        assert proc.stdout is not None
        for line in proc.stdout:
            print(line, end="")
            log.write(line)
            log.flush()
        return proc.wait()


def _config() -> dict:
    return _load_json(LOCAL_CONFIG, {}) or {}


def _remember_source(path: Path) -> None:
    cfg = _config()
    cfg["asset_source"] = str(path.resolve())
    _save_json(LOCAL_CONFIG, cfg)


def _candidate_sources(explicit: str | None) -> list[Path]:
    candidates: list[Path] = []
    if explicit:
        candidates.append(Path(explicit).expanduser())
    env = os.environ.get("HAVENWILD_ASSET_SOURCE")
    if env:
        candidates.append(Path(env).expanduser())
    cfg = _config().get("asset_source")
    if cfg:
        candidates.append(Path(cfg).expanduser())
    candidates += [
        ROOT / "_asset_source",
        ROOT.parent / "Havenwild_Assets",
        ROOT.parent / "ElizaWy",
        ROOT.parent / "Havenwild" / "assets" / "elizawy",
        ROOT.parent / "Test Havenwild" / "assets" / "elizawy",
    ]
    out, seen = [], set()
    for p in candidates:
        try:
            key = str(p.resolve())
        except OSError:
            key = str(p)
        if key not in seen:
            seen.add(key)
            out.append(p)
    return out


def _canonical_dir(source: Path) -> Path | None:
    checks = [
        source / "assets" / "elizawy",
        source,
    ]
    for candidate in checks:
        if (candidate / "Terrain").is_dir() and (candidate / "Structure").is_dir():
            return candidate
    return None


def _archive_dir(source: Path) -> Path | None:
    checks = [
        source,
        source / "assets" / "optional",
        source / "source",
        source / "archives",
    ]
    for candidate in checks:
        if any((candidate / name).is_file() for name in CORE_PACKS):
            return candidate
    return None


def _validate_probe(source: Path) -> bool:
    active = _load_json(ACTIVE_SHEETS, {}) or {}
    sheets = active.get("sheets", [])
    canonical = _canonical_dir(source)
    if canonical:
        for item in sheets:
            p = canonical / item["path"]
            if p.is_file() and _sha256(p) == item["sha256"]:
                return True
    archives = _archive_dir(source)
    if archives and (archives / "Terrain.zip").is_file():
        try:
            with zipfile.ZipFile(archives / "Terrain.zip") as z:
                names = set(z.namelist())
                return any(item["path"] in names for item in sheets)
        except zipfile.BadZipFile:
            return False
    return False


def resolve_asset_source(explicit: str | None) -> Path | None:
    for p in _candidate_sources(explicit):
        if p.exists() and _validate_probe(p):
            return p.resolve()
    return None


def _manifest_entries() -> list[dict]:
    data = _load_json(CORE_MANIFEST)
    if not data:
        raise RuntimeError(f"Missing asset manifest: {CORE_MANIFEST}")
    return data["entries"]


def _copy_verified(src: Path, dst: Path, expected_sha: str | None) -> tuple[bool, str]:
    if not src.is_file():
        return False, "missing"
    actual = _sha256(src)
    if expected_sha and actual != expected_sha:
        return False, f"hash mismatch {actual}"
    dst.parent.mkdir(parents=True, exist_ok=True)
    if dst.is_file() and _sha256(dst) == actual:
        return True, "already current"
    shutil.copy2(src, dst)
    return True, "copied"


def _extract_verified(z: zipfile.ZipFile, member: str, dst: Path, expected_sha: str | None) -> tuple[bool, str]:
    try:
        info = z.getinfo(member)
    except KeyError:
        return False, "missing in archive"
    _safe_rel(info.filename)
    h = hashlib.sha256()
    dst.parent.mkdir(parents=True, exist_ok=True)
    tmp = dst.with_suffix(dst.suffix + ".tmp")
    with z.open(info) as source, tmp.open("wb") as target:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            h.update(chunk)
            target.write(chunk)
    actual = h.hexdigest()
    if expected_sha and actual != expected_sha:
        tmp.unlink(missing_ok=True)
        return False, f"hash mismatch {actual}"
    if dst.is_file() and _sha256(dst) == actual:
        tmp.unlink(missing_ok=True)
        return True, "already current"
    tmp.replace(dst)
    return True, "extracted"


def sync_assets(source_arg: str | None, *, characters: bool = False, include_optional: bool = False, reference: bool = False) -> int:
    source = resolve_asset_source(source_arg)
    if not source:
        print("ASSETS: no authoritative source discovered.")
        print("Use: python ForgePY.py assets sync --source \"D:\\path\\to\\asset-source\"")
        print("Accepted source: complete project containing assets/elizawy, extracted Terrain/Structure/Objects/FX tree, or pack ZIP staging folder.")
        return 2
    _remember_source(source)
    print(f"ASSET SOURCE: {source}")
    canonical = _canonical_dir(source)
    archives = _archive_dir(source)
    entries = _manifest_entries()
    archive_handles: dict[str, zipfile.ZipFile] = {}
    copied = current = missing = bad = 0
    try:
        for entry in entries:
            rel = Path(entry["path"])
            dst = ASSET_ROOT / rel
            ok = False
            status = "missing"
            if canonical:
                ok, status = _copy_verified(canonical / rel, dst, entry.get("sha256"))
            if not ok and archives:
                pack = entry.get("pack")
                archive_path = archives / pack if pack else None
                if archive_path and archive_path.is_file():
                    z = archive_handles.get(pack)
                    if z is None:
                        z = zipfile.ZipFile(archive_path)
                        archive_handles[pack] = z
                    ok, status = _extract_verified(z, rel.as_posix(), dst, entry.get("sha256"))
            if ok:
                if status == "already current": current += 1
                else: copied += 1
            elif status.startswith("hash mismatch"):
                bad += 1
                print(f"HASH FAIL: {rel}: {status}")
            else:
                missing += 1
        # Collection-wide credits if present outside pack manifests.
        credit_candidates = []
        if canonical:
            credit_candidates.append(canonical / "Credits.txt")
        credit_candidates += [source / "Credits.txt", source / "assets" / "elizawy" / "Credits.txt"]
        for candidate in credit_candidates:
            if candidate.is_file():
                (ASSET_ROOT / "Credits.txt").parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(candidate, ASSET_ROOT / "Credits.txt")
                break

        optional_locations = [source, source / "assets" / "optional"]
        for name in OPTIONAL_ARCHIVES:
            found = next((p / name for p in optional_locations if (p / name).is_file()), None)
            if found and (include_optional or (characters and name == "Characters.zip")):
                OPTIONAL_ROOT.mkdir(parents=True, exist_ok=True)
                meta = (_load_json(CORE_MANIFEST, {}) or {}).get("optionalArchives", {}).get(name, {})
                ok, status = _copy_verified(found, OPTIONAL_ROOT / name, meta.get("sha256"))
                if not ok:
                    print(f"OPTIONAL HASH FAIL: {name}: {status}")
                    bad += 1
                else:
                    print(f"OPTIONAL: {name}: {status}")

        if reference:
            ref_src = source / "reference"
            if ref_src.is_dir():
                shutil.copytree(ref_src, REFERENCE_ROOT, dirs_exist_ok=True)
                print("REFERENCE: synchronized")

        if characters:
            rc = hydrate_characters(source_arg=str(source))
            if rc:
                return rc
    finally:
        for z in archive_handles.values():
            z.close()

    print(f"CORE ASSETS: copied/extracted={copied} current={current} missing={missing} hash_fail={bad}")
    if bad or missing:
        return 3
    build_core_catalog()
    return 0


def _write_character_state(*, count: int, archive: Path, verification: str = "archive_member_size") -> None:
    state = _load_json(ASSET_STATE, {}) or {}
    state["characters"] = {
        "hydrated": True,
        "count": count,
        "archive": str(archive.relative_to(ROOT)),
        "archiveBytes": archive.stat().st_size,
        "verification": verification,
        "updatedAt": _dt.datetime.now().astimezone().isoformat(),
    }
    _save_json(ASSET_STATE, state)


def build_character_catalog(archive: Path | None = None) -> int:
    archive = archive or (OPTIONAL_ROOT / "Characters.zip")
    entries = []
    if archive.is_file():
        with zipfile.ZipFile(archive) as z:
            infos = [item for item in z.infolist() if not item.is_dir()]
            total = len(infos)
            for idx, item in enumerate(infos, 1):
                rel = _safe_rel(item.filename)
                if not rel.parts or rel.parts[0] != "Characters":
                    raise RuntimeError(f"Unexpected character archive path: {item.filename}")
                entries.append({
                    "id": f"elizawy:{rel.as_posix()}",
                    "path": rel.as_posix(),
                    "pack": "Characters.zip",
                    "availability": "local_or_archive",
                    "bytes": item.file_size,
                    "zipCrc32": f"{item.CRC:08x}",
                    "sha256": None,
                    "imageSizePx": None,
                    "stage": "archive_metadata",
                })
                if idx % 10000 == 0 or idx == total:
                    print(f"CHARACTER CATALOG: {idx}/{total} metadata entries")
    else:
        char_root = ASSET_ROOT / "Characters"
        if not char_root.is_dir():
            print("CHARACTER CATALOG: no Characters.zip or hydrated Characters tree; skipped")
            return 0
        files = [p for p in char_root.rglob("*") if p.is_file()]
        total = len(files)
        for idx, p in enumerate(files, 1):
            rel = p.relative_to(ASSET_ROOT).as_posix()
            entries.append({
                "id": f"elizawy:{rel}",
                "path": rel,
                "pack": "Characters.zip",
                "availability": "local",
                "bytes": p.stat().st_size,
                "zipCrc32": None,
                "sha256": None,
                "imageSizePx": None,
                "stage": "local_metadata",
            })
            if idx % 10000 == 0 or idx == total:
                print(f"CHARACTER CATALOG: {idx}/{total} local metadata entries")
    payload = {
        "schema": "elizawy.characters_index.local.v1",
        "description": "Generated locally from Characters.zip metadata; character files are not individually SHA-256 hashed during normal PCC workflows.",
        "count": len(entries),
        "entries": entries,
    }
    _save_json(CHARACTER_INDEX, payload)
    print(f"CHARACTER CATALOG: {len(entries)} files -> {CHARACTER_INDEX.relative_to(ROOT)}")
    return 0


def hydrate_characters(source_arg: str | None = None, *, verify: bool = False) -> int:
    archive = OPTIONAL_ROOT / "Characters.zip"
    if not archive.is_file():
        source = resolve_asset_source(source_arg)
        if source:
            candidates = [source / "Characters.zip", source / "assets" / "optional" / "Characters.zip"]
            found = next((p for p in candidates if p.is_file()), None)
            if found:
                OPTIONAL_ROOT.mkdir(parents=True, exist_ok=True)
                expected = (_load_json(CORE_MANIFEST, {}) or {}).get("optionalArchives", {}).get("Characters.zip", {}).get("sha256")
                ok, status = _copy_verified(found, archive, expected)
                if not ok:
                    print(f"Characters.zip rejected: {status}")
                    return 3
    if not archive.is_file():
        print("Characters.zip is unavailable. Run assets sync with a source containing the archive.")
        return 2

    base = ASSET_ROOT
    with zipfile.ZipFile(archive) as z:
        infos = [item for item in z.infolist() if not item.is_dir()]
        total = len(infos)
        state = (_load_json(ASSET_STATE, {}) or {}).get("characters", {})
        state_current = (
            not verify
            and state.get("hydrated") is True
            and state.get("count") == total
            and state.get("archiveBytes") == archive.stat().st_size
            and (base / "Characters").is_dir()
        )
        if state_current:
            print(f"CHARACTERS: ready {total} files [cached hydration state]")
            if CHARACTER_INDEX.is_file():
                print(f"CHARACTER CATALOG: present -> {CHARACTER_INDEX.relative_to(ROOT)}")
                return 0
            return build_character_catalog(archive)

        char_root = base / "Characters"
        if not verify and char_root.is_dir() and not state.get("hydrated"):
            legacy_count = sum(len(files) for _, _, files in os.walk(char_root))
            if legacy_count == total:
                print(f"CHARACTERS: adopting existing complete tree ({legacy_count}/{total}) from pre-v0.2 hydration")
                print("CHARACTERS: migration used file-count certification; use `PCC.cmd assets hydrate-characters --verify` for an explicit size recheck")
                _write_character_state(count=total, archive=archive, verification="legacy_file_count")
                return build_character_catalog(archive)

        written = 0
        current = 0
        action = "verifying" if verify else "validating/hydrating"
        print(f"CHARACTERS: {action} {total} files")
        for idx, item in enumerate(infos, 1):
            rel = _safe_rel(item.filename)
            if not rel.parts or rel.parts[0] != "Characters":
                raise RuntimeError(f"Unexpected character archive path: {item.filename}")
            dst = base / rel
            try:
                if dst.stat().st_size == item.file_size:
                    current += 1
                else:
                    raise OSError
            except OSError:
                dst.parent.mkdir(parents=True, exist_ok=True)
                with z.open(item) as source, dst.open("wb") as target:
                    shutil.copyfileobj(source, target, length=1024 * 1024)
                written += 1
            if idx % 2500 == 0 or idx == total:
                print(f"CHARACTERS: {idx}/{total} checked | new={written} current={current}")

    _write_character_state(count=total, archive=archive, verification="archive_member_size")
    print(f"CHARACTERS: ready {total} files in {base / 'Characters'}")
    return build_character_catalog(archive)


def asset_status(*, full_verify: bool = False) -> int:
    active = _load_json(ACTIVE_SHEETS, {}) or {}
    problems = 0
    print(f"ASSET ROOT: {ASSET_ROOT}")
    for sheet in active.get("sheets", []):
        p = ASSET_ROOT / sheet["path"]
        if not p.is_file():
            print(f"[MISSING] {sheet['path']}")
            problems += 1
            continue
        actual = _sha256(p)
        if actual != sheet["sha256"]:
            print(f"[HASH FAIL] {sheet['path']}\n  expected {sheet['sha256']}\n  actual   {actual}")
            problems += 1
        else:
            print(f"[OK] {sheet['path']} {_png_size(p)}")

    manifest = _manifest_entries()
    if full_verify:
        verified = 0
        for idx, entry in enumerate(manifest, 1):
            rel = entry["path"]
            p = ASSET_ROOT / rel
            if not p.is_file():
                print(f"[CORE MISSING] {rel}")
                problems += 1
                continue
            expected_size = entry.get("bytes")
            if expected_size is not None and p.stat().st_size != int(expected_size):
                print(f"[CORE SIZE FAIL] {rel}: expected {expected_size}, actual {p.stat().st_size}")
                problems += 1
                continue
            expected_sha = str(entry.get("sha256") or "").lower()
            actual_sha = _sha256(p)
            if expected_sha and actual_sha != expected_sha:
                print(f"[CORE HASH FAIL] {rel}\n  expected {expected_sha}\n  actual   {actual_sha}")
                problems += 1
                continue
            verified += 1
            if idx % 50 == 0 or idx == len(manifest):
                print(f"CORE VERIFY: {idx}/{len(manifest)} checked | verified={verified} | problems={problems}")
        print(f"CORE MANIFEST FULL VERIFY: {verified}/{len(manifest)} files verified")
    else:
        present = sum(1 for e in manifest if (ASSET_ROOT / e["path"]).is_file())
        print(f"CORE MANIFEST: {present}/{len(manifest)} files present")

    char_dir = ASSET_ROOT / "Characters"
    state = (_load_json(ASSET_STATE, {}) or {}).get("characters", {})
    char_index = _load_json(CHARACTER_INDEX, {}) or {}
    if char_dir.is_dir() and state.get("hydrated"):
        print(f"CHARACTERS: {state.get('count', '?')} hydrated files [state: {state.get('verification', 'unknown')}]")
    elif char_dir.is_dir() and char_index.get("count"):
        print(f"CHARACTERS: hydrated tree present; {char_index.get('count')} indexed files")
    elif char_dir.is_dir():
        print("CHARACTERS: hydrated tree present; count not scanned during normal status")
    else:
        print("CHARACTERS: not hydrated (optional for current mapper)")
    print(f"Characters.zip: {'present' if (OPTIONAL_ROOT/'Characters.zip').is_file() else 'absent'}")
    return 0 if not problems else 3


def build_core_catalog() -> int:
    entries = []
    if not ASSET_ROOT.is_dir():
        print("CATALOG: asset root missing; nothing to index")
        return 2

    candidates: list[Path] = []
    for root_name in ("Terrain", "Structure", "Objects", "FX"):
        root = ASSET_ROOT / root_name
        if root.is_dir():
            candidates.extend(p for p in root.rglob("*") if p.is_file())
    candidates.extend(p for p in ASSET_ROOT.iterdir() if p.is_file())
    candidates = sorted(set(candidates))
    total = len(candidates)

    for idx, p in enumerate(candidates, 1):
        rel = p.relative_to(ASSET_ROOT).as_posix()
        top = Path(rel).parts[0] if Path(rel).parts else ""
        pack = f"{top}.zip" if top in {"Terrain", "Structure", "Objects", "FX"} else None
        entries.append({
            "id": f"elizawy:{rel}",
            "path": rel,
            "pack": pack,
            "availability": "local",
            "bytes": p.stat().st_size,
            "sha256": _sha256(p),
            "imageSizePx": _png_size(p) if p.suffix.lower() == ".png" else None,
            "stage": "source_only",
        })
        if idx % 100 == 0 or idx == total:
            print(f"CORE CATALOG: {idx}/{total}")

    payload = {
        "schema": "elizawy.source_index.local.v2",
        "description": "Generated locally by ForgePY from immutable core project asset copies. Character metadata is kept in characters_index.local.json.",
        "activeFamily": "LPC Revised / ElizaWy collection (multi-artist)",
        "count": len(entries),
        "entries": entries,
    }
    _save_json(GENERATED_INDEX, payload)
    print(f"CORE CATALOG: {len(entries)} files -> {GENERATED_INDEX.relative_to(ROOT)}")
    return 0


def build_catalog() -> int:
    rc = build_core_catalog()
    if rc:
        return rc
    return build_character_catalog()


def _git_text(args: list[str]) -> tuple[int, str]:
    """Run a small Git query against this project and return (rc, stdout).

    Source status/refresh intentionally use this non-streaming helper for branch,
    remote, dirty-state, and ahead/behind probes. User-visible Git operations
    such as fetch/fast-forward still use _stream so they are logged live.
    """
    try:
        proc = subprocess.run(
            ["git", "-C", str(ROOT), *args],
            cwd=str(ROOT),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        return proc.returncode, (proc.stdout or "").strip()
    except FileNotFoundError:
        return 127, "git not found"
    except OSError as exc:
        return 126, f"{type(exc).__name__}: {exc}"


def source_status() -> int:
    """Report the Git source-control state used by the Project Control Center."""
    print("SOURCE CONTROL")
    if not _command_exists("git"):
        print("[MISSING] git")
        return 127
    rc, inside = _git_text(["rev-parse", "--is-inside-work-tree"])
    if rc or inside.lower() != "true":
        print("[LOCAL] This folder is not a Git checkout yet.")
        print("        Builds continue locally; GitHub refresh activates automatically once the project is cloned/checked out with an origin remote.")
        return 0
    _, branch = _git_text(["branch", "--show-current"])
    _, head = _git_text(["rev-parse", "--short=12", "HEAD"])
    remote_rc, remote_url = _git_text(["remote", "get-url", "origin"])
    dirty_rc, dirty = _git_text(["status", "--porcelain"])
    print(f"BRANCH: {branch or '(detached)'}")
    print(f"HEAD:   {head or '-'}")
    print(f"ORIGIN: {remote_url if remote_rc == 0 else '(not configured)'}")
    print(f"WORKTREE: {'MODIFIED' if dirty_rc == 0 and dirty else 'CLEAN'}")
    if dirty:
        lines = dirty.splitlines()
        for line in lines[:20]:
            print(f"  {line}")
        if len(lines) > 20:
            print(f"  ... {len(lines) - 20} more")
    return 0


def source_refresh(*, explicit: bool = False) -> int:
    """Safely refresh a clean checkout from its GitHub origin.

    The PCC never stashes, resets, rebases, or overwrites local work. Dirty
    worktrees are reported and skipped. Clean branches are fetched and only
    fast-forwarded to origin/<current-branch>.
    """
    if not _command_exists("git"):
        print("SOURCE REFRESH: git not installed; skipped")
        return 0 if not explicit else 127
    rc, inside = _git_text(["rev-parse", "--is-inside-work-tree"])
    if rc or inside.lower() != "true":
        print("SOURCE REFRESH: local source folder is not a Git checkout; skipped")
        return 0
    remote_rc, remote_url = _git_text(["remote", "get-url", "origin"])
    if remote_rc:
        print("SOURCE REFRESH: no origin remote configured; skipped")
        return 0
    dirty_rc, dirty = _git_text(["status", "--porcelain"])
    if dirty_rc:
        print("SOURCE REFRESH: unable to read worktree status; skipped")
        return 0 if not explicit else dirty_rc
    if dirty:
        print("SOURCE REFRESH: local source is modified; GitHub fast-forward skipped to protect local work.")
        print("Commit/stash your source changes, then run `PCC.cmd source refresh`.")
        return 0
    branch_rc, branch = _git_text(["branch", "--show-current"])
    if branch_rc or not branch:
        print("SOURCE REFRESH: detached HEAD; fetching origin without moving HEAD")
        return _stream(["git", "-C", str(ROOT), "fetch", "--prune", "origin"], label="source-fetch")
    print(f"SOURCE REFRESH: {remote_url} [{branch}]")
    rc = _stream(["git", "-C", str(ROOT), "fetch", "--prune", "origin"], label="source-fetch")
    if rc:
        return rc if explicit else 0
    remote_ref = f"refs/remotes/origin/{branch}"
    ref_rc, _ = _git_text(["show-ref", "--verify", "--quiet", remote_ref])
    if ref_rc:
        print(f"SOURCE REFRESH: origin/{branch} does not exist; fetch completed, no merge performed")
        return 0
    counts_rc, counts = _git_text(["rev-list", "--left-right", "--count", f"HEAD...origin/{branch}"])
    if counts_rc:
        print("SOURCE REFRESH: unable to compare local and remote branch; no merge performed")
        return 0 if not explicit else counts_rc
    try:
        ahead, behind = [int(v) for v in counts.split()[:2]]
    except (ValueError, IndexError):
        ahead = behind = 0
    if ahead and behind:
        print(f"SOURCE REFRESH: branch diverged (ahead {ahead}, behind {behind}); automatic merge/rebase refused")
        return 0 if not explicit else 4
    if ahead:
        print(f"SOURCE REFRESH: local branch is {ahead} commit(s) ahead and not behind; keeping local HEAD")
        return 0
    if not behind:
        print("SOURCE REFRESH: already current")
        return 0
    print(f"SOURCE REFRESH: fast-forwarding {behind} commit(s)")
    return _stream(["git", "-C", str(ROOT), "merge", "--ff-only", f"origin/{branch}"], label="source-ff")

def _rustc_version_tuple() -> tuple[int, int, int] | None:
    if not _command_exists("rustc"):
        return None
    try:
        result = subprocess.run(
            ["rustc", "--version"],
            cwd=str(ROOT),
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        token = (result.stdout or "").strip().split()[1]
        numeric = token.split("-")[0].split("+")[0]
        parts = numeric.split(".")
        return tuple(int(part) for part in parts[:3])  # type: ignore[return-value]
    except (IndexError, ValueError, OSError):
        return None


def doctor() -> int:
    print(f"HAVENWILD BEVY / ForgePY v{FORGEPY_VERSION} DOCTOR")
    failures = 0

    # A cumulative update can replace ProjectControlCenter.py while an older PCC
    # process is still executing its pre-update FULL phase list. New controllers
    # stamp the active child-job record with their runtime version. If a PCC-launched
    # Doctor has no stamp (legacy controller) or the stamp disagrees with project
    # metadata, fail closed and require a fresh PCC invocation before certification.
    active = _load_json(PCC_ACTIVE_JOB, {}) or {}
    if isinstance(active, dict) and active.get("label") == "doctor":
        meta = _load_json(PROJECT_META, {}) or {}
        expected_pcc = str((meta.get("pcc") or {}).get("version", "")).strip() if isinstance(meta, dict) else ""
        runtime_pcc = str(active.get("pccVersion", "")).strip()
        if not runtime_pcc:
            print("[STALE PCC] Doctor was launched by a pre-handshake PCC process after an update. Restart PCC and rerun FULL.")
            failures += 1
        elif expected_pcc and runtime_pcc != expected_pcc:
            print(f"[STALE PCC] running controller {runtime_pcc} != project controller {expected_pcc}; restart PCC")
            failures += 1
        else:
            print(f"[OK] PCC runtime generation: {runtime_pcc}")
    print(f"ROOT: {ROOT}")
    for name in ("python", "git", "cargo", "rustc", "rustfmt"):
        display = sys.executable if name == "python" else shutil.which(name)
        ok = bool(display)
        print(f"[{'OK' if ok else 'MISSING'}] {name}: {display or '-'}")
        if name in ("cargo", "rustc", "rustfmt") and not ok:
            failures += 1
    rust_version = _rustc_version_tuple()
    if rust_version is not None:
        minimum = (1, 95, 0)
        version_text = ".".join(str(part) for part in rust_version)
        if rust_version < minimum:
            print(f"[OLD] rustc {version_text}; Havenwild Bevy requires >= 1.95.0")
            failures += 1
        else:
            print(f"[OK] rustc minimum: {version_text} >= 1.95.0")
    for p in (ROOT / "Cargo.toml", ROOT / "src" / "main.rs", CORE_MANIFEST, ACTIVE_SHEETS):
        ok = p.is_file()
        print(f"[{'OK' if ok else 'MISSING'}] {p.relative_to(ROOT)}")
        failures += 0 if ok else 1
    asset_rc = asset_status()
    if asset_rc:
        print("[INFO] Runtime assets are not ready. Use `ForgePY.py assets sync --source <path>`. Build can still be attempted.")
    if _command_exists("rustc"):
        _stream(["rustc", "--version"], label="rustc-version")
    if _command_exists("cargo"):
        _stream(["cargo", "--version"], label="cargo-version")
    return 0 if failures == 0 else 2


def cargo_fetch() -> int:
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    return _stream(["cargo", "fetch"], label="fetch")



def cargo_fmt_check() -> int:
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    return _stream(["cargo", "fmt", "--", "--check"], label="fmt-check")


def cargo_fmt_write() -> int:
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    return _stream(["cargo", "fmt"], label="fmt-write")


def cargo_check() -> int:
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    return _stream(["cargo", "check"], label="check")


def cargo_test() -> int:
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    return _stream(["cargo", "test", "--all-targets"], label="test")


def cargo_lock_verify() -> int:
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    lock = ROOT / "Cargo.lock"
    if not lock.is_file():
        print("Cargo.lock missing")
        return 2
    return _stream(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        label="lock-verify",
    )


def build(release: bool = False, *, refresh_source: bool = True) -> int:
    if refresh_source:
        rc = source_refresh(explicit=False)
        if rc:
            return rc
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    cmd = ["cargo", "build"] + (["--release"] if release else [])
    return _stream(cmd, label="build-release" if release else "build")


def run(backend: str = "dx12", native_frame: bool = False, verbose_gpu: bool = False) -> int:
    if asset_status() != 0:
        print("RUN BLOCKED: required source sheets are not hydrated.")
        return 3
    if not _command_exists("cargo"):
        print("cargo not found")
        return 127
    env = {
        "WGPU_BACKEND": backend,
        "STUDIO_NATIVE_FRAME": "1" if native_frame else "0",
        "RUST_BACKTRACE": "full" if verbose_gpu else "1",
    }
    if verbose_gpu:
        # Keep the normal run readable. This opt-in profile increases renderer/window
        # evidence only when diagnosing a backend-specific launch or draw failure.
        env["RUST_LOG"] = "info,wgpu_core=debug,wgpu_hal=info,bevy_render=debug,bevy_winit=debug"
    suffix = ("-native" if native_frame else "") + ("-gpu-debug" if verbose_gpu else "")
    return _stream(["cargo", "run"], env=env, label=f"run-{backend}{suffix}")


def full(source: str | None = None) -> int:
    rc = source_refresh(explicit=False)
    if rc:
        return rc
    rc = doctor()
    if rc:
        # FULL must honor every Doctor failure, including an old Rust compiler or
        # missing rustfmt. PCC FULL already fails closed here; keep the backend
        # entry point equivalent so users cannot obtain a weaker GREEN path.
        return rc
    if asset_status() != 0:
        rc = sync_assets(source)
        if rc:
            return rc
    rc = cargo_fetch()
    if rc:
        return rc
    if _command_exists("cargo"):
        rc = cargo_fmt_check()
        if rc:
            return rc
        rc = cargo_check()
        if rc:
            return rc
        rc = cargo_test()
        if rc:
            return rc
    rc = build(refresh_source=False)
    if rc:
        return rc
    lock = ROOT / "Cargo.lock"
    if not lock.is_file():
        print("FULL: build passed but Cargo.lock is missing; reproducible dependency snapshot required")
        return 2
    print(f"FULL: Cargo.lock present ({lock.stat().st_size} bytes)")
    rc = cargo_lock_verify()
    if rc:
        print("FULL: Cargo.lock exists but cannot reproduce the resolved dependency graph with --locked")
        return rc
    return 0


def clean() -> int:
    if _command_exists("cargo"):
        return _stream(["cargo", "clean"], label="clean")
    target = ROOT / "target"
    if target.exists():
        shutil.rmtree(target)
    print("CLEAN: target removed")
    return 0


def package_source() -> int:
    out_dir = ROOT / "artifacts"
    out_dir.mkdir(parents=True, exist_ok=True)
    out = out_dir / f"Havenwild_Bevy_Standalone_SOURCE_ONLY_{_dt.date.today():%Y%m%d}.zip"
    excluded_roots = {".git", "target", "artifacts", ".vs", ".idea", ".vscode", ".forgepy", ".pcc", "__pycache__"}
    with zipfile.ZipFile(out, "w", allowZip64=True, compression=zipfile.ZIP_DEFLATED) as z:
        for p in sorted(ROOT.rglob("*")):
            if not p.is_file() or p == out or p.suffix.lower() == ".pyc":
                continue
            rel = p.relative_to(ROOT)
            if rel.as_posix() in {"LOCAL_SEED_MANIFEST.json", "LOCAL_SEED_OVERLAY_README.txt", "pcc_patch.json"}:
                continue
            if rel.parts[:3] == ("content", "scenes", "derived"):
                # Only an explicit authored-scene promotion may publish local corrections.
                continue
            if p.name.lower().endswith(".pccpatch.zip"):
                continue
            if rel.parts and rel.parts[0] in excluded_roots:
                continue
            if rel.parts and rel.parts[0] == "updates":
                if rel.as_posix() != "updates/README.md":
                    continue
            if rel.parts and rel.parts[0] in {"assets", "reference"}:
                # Keep only explanatory source-controlled asset readme.
                if rel.as_posix() != "assets/README.md":
                    continue
            if rel.as_posix() in {GENERATED_INDEX.relative_to(ROOT).as_posix(), CHARACTER_INDEX.relative_to(ROOT).as_posix()}:
                continue
            if rel.as_posix() in LEGACY_GENERATED_CATALOGS:
                continue
            z.write(p, rel.as_posix())
    print(f"PACKAGE: {out}")
    return 0


def setup(source: str | None = None) -> int:
    for p in (ASSET_ROOT, OPTIONAL_ROOT, CATALOG_ROOT, LOG_DIR):
        p.mkdir(parents=True, exist_ok=True)
    if asset_status() != 0:
        rc = sync_assets(source)
        if rc:
            return rc
    return cargo_fetch() if _command_exists("cargo") else doctor()


def menu() -> int:
    actions = {
        "1": ("FULL gate (doctor -> assets -> fetch -> fmt/check/test/build)", lambda: full()),
        "2": ("Run Studio (DX12)", lambda: run("dx12", False)),
        "3": ("Build Studio", lambda: build()),
        "4": ("Doctor / status", doctor),
        "5": ("Sync core assets", lambda: sync_assets(None)),
        "6": ("Hydrate characters", lambda: hydrate_characters()),
        "7": ("Rebuild local asset catalog", build_catalog),
        "8": ("Run Studio (Vulkan)", lambda: run("vulkan", False)),
        "9": ("Run Studio (DX12 native frame)", lambda: run("dx12", True)),
        "10": ("Package source-only ZIP", package_source),
        "11": ("Cargo clean", clean),
        "12": ("Source status / GitHub origin", source_status),
        "13": ("Refresh clean source from GitHub", lambda: source_refresh(explicit=True)),
    }
    while True:
        print("\n" + "=" * 72)
        print(f" HAVENWILD BEVY — ForgePY v{FORGEPY_VERSION}")
        print("=" * 72)
        for key, (label, _) in actions.items():
            print(f" {key:>2}. {label}")
        print("  0. Exit")
        choice = input("> ").strip()
        if choice == "0":
            return 0
        action = actions.get(choice)
        if not action:
            continue
        rc = action[1]()
        print(f"RESULT: {'PASS' if rc == 0 else 'FAIL'} ({rc})")
        input("Press Enter...")


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(prog="ForgePY.py")
    sub = p.add_subparsers(dest="command")
    sub.add_parser("menu")
    sub.add_parser("doctor")
    sub.add_parser("fetch")
    sub.add_parser("fmt")
    sub.add_parser("format")
    sub.add_parser("check")
    sub.add_parser("test")
    sub.add_parser("lock-verify")
    source_p = sub.add_parser("source")
    source_sub = source_p.add_subparsers(dest="source_command")
    source_sub.add_parser("status")
    source_sub.add_parser("refresh")
    build_p = sub.add_parser("build")
    build_p.add_argument("--no-refresh", action="store_true")
    sub.add_parser("clean")
    sub.add_parser("catalog")
    sub.add_parser("package")
    setup_p = sub.add_parser("setup")
    setup_p.add_argument("--source")
    full_p = sub.add_parser("full")
    full_p.add_argument("--source")
    run_p = sub.add_parser("run")
    run_p.add_argument("--backend", choices=["dx12", "vulkan"], default="dx12")
    run_p.add_argument("--native-frame", action="store_true")
    run_p.add_argument("--verbose-gpu", action="store_true")
    assets = sub.add_parser("assets")
    aset = assets.add_subparsers(dest="asset_command")
    status_p = aset.add_parser("status")
    status_p.add_argument("--full", action="store_true", help="hash-verify every core manifest file")
    sync_p = aset.add_parser("sync")
    sync_p.add_argument("--source")
    sync_p.add_argument("--characters", action="store_true")
    sync_p.add_argument("--include-optional", action="store_true")
    sync_p.add_argument("--reference", action="store_true")
    char_p = aset.add_parser("hydrate-characters")
    char_p.add_argument("--source")
    char_p.add_argument("--verify", action="store_true", help="recheck all hydrated character file sizes even when local hydration state is current")
    return p.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    command = args.command or "menu"
    if command == "menu": return menu()
    if command == "doctor": return doctor()
    if command == "fetch": return cargo_fetch()
    if command == "fmt": return cargo_fmt_check()
    if command == "format": return cargo_fmt_write()
    if command == "check": return cargo_check()
    if command == "test": return cargo_test()
    if command == "lock-verify": return cargo_lock_verify()
    if command == "source":
        if args.source_command == "status": return source_status()
        if args.source_command == "refresh": return source_refresh(explicit=True)
        print("Use `ForgePY.py source --help`.")
        return 2
    if command == "build": return build(refresh_source=not args.no_refresh)
    if command == "clean": return clean()
    if command == "catalog": return build_catalog()
    if command == "package": return package_source()
    if command == "setup": return setup(args.source)
    if command == "full": return full(args.source)
    if command == "run": return run(args.backend, args.native_frame, args.verbose_gpu)
    if command == "assets":
        if args.asset_command == "status": return asset_status(full_verify=args.full)
        if args.asset_command == "sync": return sync_assets(args.source, characters=args.characters, include_optional=args.include_optional, reference=args.reference)
        if args.asset_command == "hydrate-characters": return hydrate_characters(args.source, verify=args.verify)
        print("Use `ForgePY.py assets --help`.")
        return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
