#!/usr/bin/env python3
"""Havenwild Bevy Project Control Center.

PCC is the resilient operator/supervisor for this standalone repository.
ForgePY remains the project backend for hydration, Cargo, source refresh, and
source-only packaging. PCC adds:

- persistent interactive host that survives child-process failures
- per-session and per-job live streaming logs
- JSON job receipts and last-job state
- automatic source-only debug bundles on failures
- project/source/asset dashboard
- governed transactional .pccpatch.zip update inbox
- terrain-contract validation/certification hooks
- source-only build/package/diagnostics orchestration

No PCC diagnostic, receipt, update, or package path may vendor hydrated assets.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import queue
import re
from pathlib import Path
import shutil
import subprocess
import sys
import threading
import tempfile
import time
import traceback
import zipfile

ROOT = Path(__file__).resolve().parent
PCC_VERSION = "1.3.5"
STATE_ROOT = ROOT / ".pcc"
LOG_ROOT = STATE_ROOT / "logs"
RECEIPT_ROOT = STATE_ROOT / "receipts"
DEBUG_ROOT = STATE_ROOT / "debug"
GATE_ROOT = STATE_ROOT / "gates"
ROLLBACK_ROOT = STATE_ROOT / "rollback"
STAGING_ROOT = STATE_ROOT / "staging"
LAST_JOB = STATE_ROOT / "last_job.json"
LAST_GATE = STATE_ROOT / "last_gate.json"
ACTIVE_JOB = STATE_ROOT / "active_job.json"
PUBLISH_STATE = STATE_ROOT / "git_publish_state.json"
UPDATE_ROOT = ROOT / "updates"
UPDATE_INBOX = UPDATE_ROOT / "inbox"
UPDATE_ARCHIVE = UPDATE_ROOT / "archive"
UPDATE_REJECTED = UPDATE_ROOT / "rejected"
BACKEND = ROOT / "ForgePY.py"
TERRAIN_VALIDATOR = ROOT / "tools" / "terrain_validate.py"
LEGACY_MAPPING_RECOVERY = ROOT / "tools" / "recover_legacy_mapping.py"
SUMMER_FLATWORLD_MAPPER = ROOT / "tools" / "build_summer_flatworld_mapping.py"
NATIVE_SUMMER_CONVERGENCE = ROOT / "tools" / "converge_native_summer_authority.py"
CLIFF_SUMMER_AUTHORITY = ROOT / "tools" / "build_cliff_summer_authority.py"
PATCH_SELFTEST = ROOT / "tools" / "pcc_patch_selftest.py"
RUST_SOURCE_AUDIT = ROOT / "tools" / "rust_source_audit.py"
ELIZAWY_AUDIT = ROOT / "tools" / "audit_elizawy_concordance.py"
ELIZAWY_DERIVATIVE_GATE = ROOT / "tools" / "verify_elizawy_derivative_evidence.py"
PROJECT_META = ROOT / "project" / "forgepy.project.json"
FORBIDDEN_PATCH_ROOTS = {
    ".git", ".pcc", ".forgepy", "target", "artifacts", "assets", "reference"
}
PCC_RESTART_PATHS = {"ProjectControlCenter.py"}
PATCH_ID_RE = re.compile(r"^[A-Za-z0-9](?:[A-Za-z0-9._-]{0,94}[A-Za-z0-9])?$")
WINDOWS_DEVICE_NAMES = {"CON", "PRN", "AUX", "NUL", *(f"COM{i}" for i in range(1, 10)), *(f"LPT{i}" for i in range(1, 10))}
_SELF_UPDATE_APPLIED = False
MAX_PATCH_FILES = 512
MAX_PATCH_FILE_BYTES = 16 * 1024 * 1024
MAX_PATCH_TOTAL_BYTES = 64 * 1024 * 1024

FORBIDDEN_PATCH_EXACT = {
    "content/catalog/source_index.local.json",
    "content/catalog/characters_index.local.json",
    "LOCAL_SEED_MANIFEST.json",
    "LOCAL_SEED_OVERLAY_README.txt",
    # Manual extract/overwrite fallback may leave the patch manifest at root. It is
    # deployment metadata, never publishable project source.
    "pcc_patch.json",
}

GIT_PUBLISH_FORBIDDEN_ROOTS = {
    ".git", ".pcc", ".forgepy", "target", "artifacts", "reference",
    ".vs", ".idea", ".vscode", "__pycache__", ".pytest_cache", ".mypy_cache",
}
GIT_PUBLISH_FORBIDDEN_PREFIXES = (
    "updates/inbox/", "updates/archive/", "updates/rejected/",
)
GIT_PUBLISH_ALLOWED_ASSET_FILES = {"assets/README.md"}


_SESSION_PATH: Path | None = None
_SESSION_HANDLE = None


def now_id() -> str:
    return dt.datetime.now().astimezone().strftime("%Y%m%d-%H%M%S-%f")


def iso_now() -> str:
    return dt.datetime.now().astimezone().isoformat()


def ensure_dirs() -> None:
    for p in (LOG_ROOT, RECEIPT_ROOT, DEBUG_ROOT, GATE_ROOT, ROLLBACK_ROOT, STAGING_ROOT,
              UPDATE_INBOX, UPDATE_ARCHIVE, UPDATE_REJECTED):
        p.mkdir(parents=True, exist_ok=True)


def _prune_files(folder: Path, pattern: str, keep: int) -> None:
    try:
        files = sorted(folder.glob(pattern), key=lambda p: p.stat().st_mtime, reverse=True)
    except OSError:
        return
    for path in files[keep:]:
        try:
            path.unlink()
        except OSError:
            pass


def prune_state() -> None:
    ensure_dirs()
    _prune_files(LOG_ROOT, "*.log", 160)
    _prune_files(RECEIPT_ROOT, "*.json", 200)
    _prune_files(DEBUG_ROOT, "*.zip", 32)
    _prune_files(GATE_ROOT, "*.json", 80)


def recover_stale_active_job() -> None:
    if not ACTIVE_JOB.is_file():
        return
    active = read_json(ACTIVE_JOB, {}) or {}
    recovered = {
        "schema": "havenwild.bevy.pcc.recovered_job.v1",
        "recoveredAt": iso_now(),
        "result": "INTERRUPTED_OR_HOST_TERMINATED",
        "activeJob": active,
    }
    target = RECEIPT_ROOT / f"{now_id()}_recovered-interrupted-job.json"
    write_json(target, recovered)
    ACTIVE_JOB.unlink(missing_ok=True)
    pcc_print("RECOVERY: previous PCC session ended with an active child job.")
    pcc_print("RECOVERY RECEIPT:", target)


def pcc_print(*parts, sep=" ", end="\n") -> None:
    text = sep.join(str(p) for p in parts) + end
    try:
        sys.stdout.write(text)
        sys.stdout.flush()
    except Exception:
        pass
    if _SESSION_HANDLE is not None:
        try:
            _SESSION_HANDLE.write(text)
            _SESSION_HANDLE.flush()
        except Exception:
            pass


def open_session() -> None:
    global _SESSION_PATH, _SESSION_HANDLE
    ensure_dirs()
    prune_state()
    _SESSION_PATH = LOG_ROOT / f"{now_id()}_pcc-session.log"
    _SESSION_HANDLE = _SESSION_PATH.open("a", encoding="utf-8", errors="replace")
    pcc_print(f"HAVENWILD BEVY PCC v{PCC_VERSION}")
    pcc_print(f"SESSION: {_SESSION_PATH}")
    pcc_print(f"ROOT: {ROOT}")
    recover_stale_active_job()


def close_session() -> None:
    global _SESSION_HANDLE
    if _SESSION_HANDLE is not None:
        try:
            _SESSION_HANDLE.flush()
            _SESSION_HANDLE.close()
        finally:
            _SESSION_HANDLE = None


def write_json(path: Path, payload) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def read_json(path: Path, default=None):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError, OSError):
        return default


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def python_cmd() -> str:
    return sys.executable


def safe_rel(raw: str) -> Path:
    # Governed update paths use one canonical slash form. Backslashes are not
    # normalized because ZIP extraction on Windows can otherwise make two archive
    # names address the same destination through different spellings.
    if "\\" in raw:
        raise ValueError(f"patch paths must use forward slashes: {raw!r}")
    rel = Path(raw)
    if rel.is_absolute() or not rel.parts or any(part in ("", ".", "..") for part in rel.parts):
        raise ValueError(f"unsafe relative path: {raw!r}")
    for part in rel.parts:
        if part != part.strip() or part.endswith((".", " ")):
            raise ValueError(f"Windows-ambiguous path segment is not allowed: {part!r}")
        if any(ord(ch) < 32 for ch in part) or any(ch in '<>:"|?*' for ch in part):
            raise ValueError(f"Windows-invalid path segment is not allowed: {part!r}")
        stem = part.split(".", 1)[0].upper()
        if stem in WINDOWS_DEVICE_NAMES:
            raise ValueError(f"Windows reserved device path is not allowed: {part!r}")
    return rel


def patch_path_key(rel: Path) -> str:
    # Windows destination identity is case-insensitive for the supported project
    # workflow. Reject collisions even when validation runs on Linux/macOS.
    return "/".join(part.casefold() for part in rel.parts)


def ensure_patch_destination(rel: Path) -> Path:
    root = ROOT.resolve()
    current = ROOT
    for part in rel.parts[:-1]:
        current = current / part
        if current.is_symlink():
            raise ValueError(f"patch destination traverses symlinked directory: {rel.as_posix()}")
    destination = ROOT / rel
    parent = destination.parent.resolve(strict=False)
    try:
        parent.relative_to(root)
    except ValueError as exc:
        raise ValueError(f"patch destination escapes repository root: {rel.as_posix()}") from exc
    if destination.is_symlink():
        raise ValueError(f"patch destination is a symlink: {rel.as_posix()}")
    return destination


def source_safe_patch_path(rel: Path) -> bool:
    posix = rel.as_posix()
    if posix in FORBIDDEN_PATCH_EXACT:
        return False
    if rel.parts[0].lower() in {x.lower() for x in FORBIDDEN_PATCH_ROOTS}:
        return False
    return True


def run_stream(command: list[str], *, label: str, env: dict[str, str] | None = None,
               cwd: Path | None = None, debug_on_fail: bool = True) -> int:
    """Run one child process with live output, heartbeat, receipts, and crash state."""
    ensure_dirs()
    run_cwd = cwd or ROOT
    job_id = f"{now_id()}_{label}"
    log_path = LOG_ROOT / f"{job_id}.log"
    receipt_path = RECEIPT_ROOT / f"{job_id}.json"
    started = time.monotonic()
    started_at = iso_now()
    merged_env = os.environ.copy()
    merged_env.setdefault("PYTHONUTF8", "1")
    merged_env.setdefault("PYTHONIOENCODING", "utf-8")
    merged_env.setdefault("PYTHONUNBUFFERED", "1")
    if env:
        merged_env.update(env)

    active = {
        "schema": "havenwild.bevy.pcc.active_job.v1",
        "pccVersion": PCC_VERSION,
        "jobId": job_id,
        "label": label,
        "command": command,
        "startedAt": started_at,
        "jobLog": str(log_path.relative_to(ROOT)),
        "pid": None,
    }
    write_json(ACTIVE_JOB, active)

    pcc_print("-" * 72)
    pcc_print(f"START {label}")
    pcc_print("RUN:", subprocess.list2cmdline(command))
    pcc_print("JOB LOG:", log_path)
    rc = 1
    error_text = None
    interrupted = False
    proc: subprocess.Popen[str] | None = None

    with log_path.open("w", encoding="utf-8", errors="replace") as job_log:
        job_log.write(f"PCC VERSION: {PCC_VERSION}\n")
        job_log.write(f"START: {started_at}\n")
        job_log.write(f"ROOT: {ROOT}\n")
        job_log.write(f"RUN: {subprocess.list2cmdline(command)}\n\n")
        try:
            proc = subprocess.Popen(
                command,
                cwd=str(run_cwd),
                env=merged_env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
                errors="replace",
                bufsize=1,
            )
            active["pid"] = proc.pid
            write_json(ACTIVE_JOB, active)
            assert proc.stdout is not None

            lines: queue.Queue[str | None] = queue.Queue()

            def reader() -> None:
                try:
                    for line in proc.stdout:
                        lines.put(line)
                finally:
                    lines.put(None)

            thread = threading.Thread(target=reader, name=f"pcc-{label}-stdout", daemon=True)
            thread.start()
            last_visible = time.monotonic()
            heartbeat_seconds = 15.0

            while True:
                try:
                    line = lines.get(timeout=1.0)
                except queue.Empty:
                    if proc.poll() is None and time.monotonic() - last_visible >= heartbeat_seconds:
                        elapsed = time.monotonic() - started
                        heartbeat = f"[PCC] {label} still running... {elapsed:.0f}s elapsed\n"
                        try:
                            sys.stdout.write(heartbeat)
                            sys.stdout.flush()
                        except Exception:
                            pass
                        if _SESSION_HANDLE is not None:
                            _SESSION_HANDLE.write(heartbeat)
                            _SESSION_HANDLE.flush()
                        job_log.write(heartbeat)
                        job_log.flush()
                        last_visible = time.monotonic()
                    continue
                if line is None:
                    break
                last_visible = time.monotonic()
                try:
                    sys.stdout.write(line)
                    sys.stdout.flush()
                except Exception:
                    pass
                if _SESSION_HANDLE is not None:
                    _SESSION_HANDLE.write(line)
                    _SESSION_HANDLE.flush()
                job_log.write(line)
                job_log.flush()
            rc = proc.wait()
        except KeyboardInterrupt:
            interrupted = True
            rc = 130
            pcc_print(f"INTERRUPT: stopping {label} child process")
            if proc is not None and proc.poll() is None:
                try:
                    proc.terminate()
                    proc.wait(timeout=5)
                except Exception:
                    try:
                        proc.kill()
                    except Exception:
                        pass
        except Exception as exc:
            rc = 250
            error_text = f"{type(exc).__name__}: {exc}"
            tb = traceback.format_exc()
            pcc_print("PCC CHILD LAUNCH FAILURE:", error_text)
            pcc_print(tb)
            job_log.write("\nPCC CHILD LAUNCH FAILURE\n")
            job_log.write(tb)

    elapsed = round(time.monotonic() - started, 3)
    receipt = {
        "schema": "havenwild.bevy.pcc.job_receipt.v1",
        "pccVersion": PCC_VERSION,
        "jobId": job_id,
        "label": label,
        "command": command,
        "startedAt": started_at,
        "finishedAt": iso_now(),
        "durationSeconds": elapsed,
        "exitCode": rc,
        "result": "INTERRUPTED" if interrupted else ("PASS" if rc == 0 else "FAIL"),
        "jobLog": str(log_path.relative_to(ROOT)),
        "sessionLog": str(_SESSION_PATH.relative_to(ROOT)) if _SESSION_PATH else None,
        "error": error_text,
    }
    write_json(receipt_path, receipt)
    receipt["receipt"] = str(receipt_path.relative_to(ROOT))
    write_json(LAST_JOB, receipt)
    ACTIVE_JOB.unlink(missing_ok=True)
    pcc_print(
        f"END {label}: {receipt['result']} ({rc}) in {elapsed:.1f}s"
    )
    pcc_print(f"RECEIPT: {receipt_path}")
    if rc != 0 and debug_on_fail:
        try:
            bundle = make_debug_bundle(
                reason=f"job-failure:{label}", job_log=log_path, receipt=receipt_path
            )
            pcc_print(f"DEBUG BUNDLE: {bundle}")
        except Exception:
            pcc_print("DEBUG BUNDLE CREATION FAILED")
            pcc_print(traceback.format_exc())
    return rc

def run_backend(args: list[str], *, label: str, debug_on_fail: bool = True) -> int:
    return run_stream([python_cmd(), str(BACKEND), *args], label=label, debug_on_fail=debug_on_fail)


def run_terrain(args: list[str], *, label: str, debug_on_fail: bool = True) -> int:
    if not TERRAIN_VALIDATOR.is_file():
        pcc_print(f"[MISSING] {TERRAIN_VALIDATOR.relative_to(ROOT)}")
        return 2
    return run_stream([python_cmd(), str(TERRAIN_VALIDATOR), *args], label=label, debug_on_fail=debug_on_fail)


def run_legacy_mapping(args: list[str], *, label: str, debug_on_fail: bool = True) -> int:
    if not LEGACY_MAPPING_RECOVERY.is_file():
        pcc_print(f"[MISSING] {LEGACY_MAPPING_RECOVERY.relative_to(ROOT)}")
        return 2
    return run_stream(
        [python_cmd(), str(LEGACY_MAPPING_RECOVERY), *args],
        label=label,
        debug_on_fail=debug_on_fail,
    )



def run_summer_flatworld_mapper(args: list[str], *, label: str, debug_on_fail: bool = True) -> int:
    if not SUMMER_FLATWORLD_MAPPER.is_file():
        pcc_print(f"[MISSING] {SUMMER_FLATWORLD_MAPPER.relative_to(ROOT)}")
        return 2
    return run_stream(
        [python_cmd(), str(SUMMER_FLATWORLD_MAPPER), *args],
        label=label,
        debug_on_fail=debug_on_fail,
    )


def run_native_summer_convergence(args: list[str], *, label: str, debug_on_fail: bool = True) -> int:
    if not NATIVE_SUMMER_CONVERGENCE.is_file():
        pcc_print(f"[MISSING] {NATIVE_SUMMER_CONVERGENCE.relative_to(ROOT)}")
        return 2
    return run_stream(
        [python_cmd(), str(NATIVE_SUMMER_CONVERGENCE), *args],
        label=label,
        debug_on_fail=debug_on_fail,
    )


def run_cliff_summer_authority(args: list[str], *, label: str, debug_on_fail: bool = True) -> int:
    if not CLIFF_SUMMER_AUTHORITY.is_file():
        pcc_print(f"[MISSING] {CLIFF_SUMMER_AUTHORITY.relative_to(ROOT)}")
        return 2
    return run_stream(
        [python_cmd(), str(CLIFF_SUMMER_AUTHORITY), *args],
        label=label,
        debug_on_fail=debug_on_fail,
    )

def capture(command: list[str], *, timeout: float = 20.0) -> tuple[int, str]:
    try:
        p = subprocess.run(
            command,
            cwd=str(ROOT),
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
            timeout=timeout,
        )
        return p.returncode, (p.stdout or "").strip()
    except subprocess.TimeoutExpired:
        return 124, f"timeout after {timeout:.0f}s: {subprocess.list2cmdline(command)}"
    except Exception as exc:
        return 250, f"{type(exc).__name__}: {exc}"


def git_snapshot() -> str:
    if not shutil.which("git"):
        return "git: unavailable\n"
    rc, inside = capture(["git", "rev-parse", "--is-inside-work-tree"])
    if rc or inside.lower() != "true":
        return "git: folder is not a checkout\n"
    lines = []
    for name, cmd in [
        ("branch", ["git", "branch", "--show-current"]),
        ("head", ["git", "rev-parse", "HEAD"]),
        ("origin", ["git", "remote", "get-url", "origin"]),
        ("status", ["git", "status", "--porcelain=v1", "--branch"]),
    ]:
        _, out = capture(cmd)
        lines.append(f"{name}:\n{out}\n")
    return "\n".join(lines)


def git_bytes(args: list[str], *, timeout: float = 30.0) -> tuple[int, bytes]:
    """Run a machine-readable Git query and return stdout bytes only.

    Git writes advisory messages such as core.autocrlf / line-ending warnings to
    stderr.  NUL-delimited commands (diff --name-only -z, ls-files -z, etc.)
    must never merge those warnings into stdout or they become fake path names.
    Human-facing Git commands continue to use run_stream/capture for visible logs.
    """
    try:
        proc = subprocess.run(
            ["git", *args],
            cwd=str(ROOT),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=timeout,
        )
        return proc.returncode, proc.stdout or b""
    except subprocess.TimeoutExpired as exc:
        return 124, exc.stdout or b""
    except (FileNotFoundError, OSError):
        return 127, b""


def project_repository_url() -> str:
    """Return this project's declared canonical GitHub origin."""
    meta = read_json(PROJECT_META, {}) or {}
    value = str(((meta.get("sourceControl") or {}).get("repositoryUrl")) or "").strip()
    if value and value != "auto-detect-from-git-origin":
        return value
    return "https://github.com/shifty81/HavenwildBevy.git"


def git_worktree_state() -> dict:
    """Return Git state plus the source-byte fingerprint Full Gate certifies."""
    source_fingerprint, source_paths = source_snapshot_fingerprint()
    state = {
        "available": False,
        "insideWorktree": False,
        "branch": None,
        "head": None,
        "origin": None,
        "fingerprint": None,
        "sourceFingerprint": source_fingerprint,
        "sourcePathCount": len(source_paths),
        "changedPaths": [],
    }
    if not shutil.which("git"):
        return state
    rc, inside = capture(["git", "rev-parse", "--is-inside-work-tree"])
    if rc or inside.lower() != "true":
        state["available"] = True
        return state
    state["available"] = True
    state["insideWorktree"] = True
    _, branch = capture(["git", "branch", "--show-current"])
    _, head = capture(["git", "rev-parse", "HEAD"])
    remote_rc, origin = capture(["git", "remote", "get-url", "origin"])
    state["branch"] = branch or None
    state["head"] = head or None
    state["origin"] = origin if remote_rc == 0 and origin else None

    diff_rc, diff_bytes = git_bytes(["diff", "--binary", "--no-ext-diff", "HEAD", "--"])
    tracked_rc, tracked_bytes = git_bytes(["diff", "--name-only", "-z", "HEAD", "--"])
    untracked_rc, untracked_bytes = git_bytes(["ls-files", "--others", "--exclude-standard", "-z"])
    if diff_rc or tracked_rc or untracked_rc:
        return state

    def decode_paths(raw: bytes) -> list[str]:
        return [
            part.decode("utf-8", errors="surrogateescape").replace("\\", "/")
            for part in raw.split(b"\0") if part
        ]

    tracked = decode_paths(tracked_bytes)
    untracked = decode_paths(untracked_bytes)
    changed = sorted(set(tracked + untracked), key=str.lower)
    state["changedPaths"] = changed

    digest = hashlib.sha256()
    digest.update(b"havenwild.git.worktree.v1\0")
    digest.update((head or "").encode("utf-8", errors="surrogateescape"))
    digest.update(b"\0DIFF\0")
    digest.update(diff_bytes)
    digest.update(b"\0UNTRACKED\0")
    for rel in sorted(untracked, key=str.lower):
        digest.update(rel.encode("utf-8", errors="surrogateescape"))
        digest.update(b"\0")
        path = ROOT / Path(rel)
        if path.is_file():
            try:
                digest.update(hashlib.sha256(path.read_bytes()).digest())
            except OSError:
                digest.update(b"<unreadable>")
        else:
            digest.update(b"<missing>")
        digest.update(b"\0")
    state["fingerprint"] = digest.hexdigest()
    return state


def git_publish_path_error(rel: str) -> str | None:
    normalized = rel.replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    if not normalized:
        return "empty source path"
    root = normalized.split("/", 1)[0].lower()
    if root in {value.lower() for value in GIT_PUBLISH_FORBIDDEN_ROOTS}:
        return f"local/runtime state is not publishable: {normalized}"
    lower = normalized.lower()
    if any(lower.startswith(prefix.lower()) for prefix in GIT_PUBLISH_FORBIDDEN_PREFIXES):
        return f"patch inbox/history is not source: {normalized}"
    if root == "assets" and normalized not in GIT_PUBLISH_ALLOWED_ASSET_FILES:
        return f"hydrated/runtime asset payload is not publishable: {normalized}"
    name = Path(normalized).name.lower()
    if name.endswith(".pyc"):
        return f"generated Python bytecode is not source: {normalized}"
    if lower.count("/") == 0 and (name.endswith(".pccpatch.zip") or name.endswith(".patch") or name.endswith(".zip")):
        return f"root handoff/archive is not source: {normalized}"
    if normalized in FORBIDDEN_PATCH_EXACT:
        return f"generated/local catalog is not publishable: {normalized}"
    return None


def source_snapshot_fingerprint() -> tuple[str, list[str]]:
    """Hash the publishable source tree independently of Git history/configuration.

    This lets a successful Full Gate remain valid while PCC initializes Git, adds
    the canonical origin, or adopts existing origin history, provided not one
    publishable source byte changed. Local runtime/build/asset state is excluded by
    the same publish policy used by commit staging.
    """
    digest = hashlib.sha256()
    digest.update(b"havenwild.source.snapshot.v1\0")
    paths: list[str] = []
    for path in sorted(ROOT.rglob("*"), key=lambda p: p.as_posix().lower()):
        if not path.is_file():
            continue
        try:
            rel = path.relative_to(ROOT).as_posix()
        except ValueError:
            continue
        if git_publish_path_error(rel):
            continue
        paths.append(rel)
        digest.update(rel.encode("utf-8", errors="surrogateescape"))
        digest.update(b"\0")
        try:
            digest.update(hashlib.sha256(path.read_bytes()).digest())
        except OSError:
            digest.update(b"<unreadable>")
        digest.update(b"\0")
    return digest.hexdigest(), paths


def git_publish_readiness() -> tuple[bool, str, dict]:
    state = git_worktree_state()
    gate = read_json(LAST_GATE, {}) or {}
    gate_passed = gate.get("result") == "PASS"
    certified_source = (gate.get("sourceControl") or {}).get("sourceFingerprint")
    current_source = state.get("sourceFingerprint")

    if not state.get("available"):
        return False, "Git is unavailable", state
    if not state.get("insideWorktree"):
        suffix = "; primary option 2 can bootstrap the canonical GitHub origin" if gate_passed else "; run Full Gate, then primary option 2 can bootstrap Git"
        return False, "repository is not a Git checkout" + suffix, state
    if not state.get("branch"):
        return False, "detached HEAD is not publishable", state
    if not state.get("origin"):
        suffix = "; primary option 2 can connect the canonical GitHub origin" if gate_passed else ""
        return False, "origin remote is not configured" + suffix, state
    for rel in state.get("changedPaths") or []:
        error = git_publish_path_error(rel)
        if error:
            return False, error, state
    if not gate_passed:
        return False, "run Full Gate successfully before publishing", state

    # PCC 1.3.2+ certifies source bytes independently of Git history/configuration.
    # That allows first-time Git bootstrap/origin adoption after a successful gate
    # without invalidating certification when the actual source tree is unchanged.
    if certified_source:
        if certified_source != current_source:
            return False, "source changed after the last successful Full Gate", state
    else:
        # Backward-compatible fallback for older gate receipts.
        certified_git = (gate.get("sourceControl") or {}).get("fingerprint")
        if not certified_git:
            return False, "last Full Gate predates publish certification; run Full Gate again", state
        if certified_git != state.get("fingerprint"):
            return False, "source/Git state changed after the last successful Full Gate", state

    publish = read_json(PUBLISH_STATE, {}) or {}
    same_committed_state = (
        publish.get("gateId") == gate.get("gateId")
        and publish.get("head") == state.get("head")
        and publish.get("branch") == state.get("branch")
        and publish.get("origin") == state.get("origin")
        and not (state.get("changedPaths") or [])
    )
    if same_committed_state and publish.get("status") == "COMMITTED_PENDING_PUSH":
        return True, "certified PCC commit is pending push; safe retry available", state
    if same_committed_state and publish.get("status") == "PASS":
        return False, "current certified commit is already published; run Full Gate after new source edits", state
    return True, f"certified by {gate.get('gateId', 'last Full Gate')}", state


def _git_remote_counts(branch: str) -> tuple[int, int, bool, int]:
    remote_ref = f"refs/remotes/origin/{branch}"
    ref_rc, _ = capture(["git", "show-ref", "--verify", "--quiet", remote_ref])
    if ref_rc:
        return 0, 0, False, 0
    counts_rc, counts = capture(["git", "rev-list", "--left-right", "--count", f"HEAD...origin/{branch}"])
    if counts_rc:
        return 0, 0, True, counts_rc
    try:
        ahead, behind = [int(v) for v in counts.split()[:2]]
    except (ValueError, IndexError):
        return 0, 0, True, 4
    return ahead, behind, True, 0


def source_publish(message: str | None = None) -> int:
    """Commit and push only the exact source state certified by Full Gate."""
    ready, reason, state = git_publish_readiness()
    pcc_print("GITHUB PUBLISH PREFLIGHT:", "READY" if ready else "BLOCKED", f"— {reason}")
    if not ready:
        return 6

    branch = str(state["branch"])
    origin = str(state["origin"])
    pcc_print(f" Branch     : {branch}")
    pcc_print(f" Origin     : {origin}")
    pcc_print(f" Gate       : {(read_json(LAST_GATE, {}) or {}).get('gateId', '-')}")
    changed = list(state.get("changedPaths") or [])
    pcc_print(f" Changes    : {len(changed)} source path(s)")
    for rel in changed[:30]:
        pcc_print(f"   {rel}")
    if len(changed) > 30:
        pcc_print(f"   ... {len(changed) - 30} more")

    rc = run_stream(["git", "fetch", "--prune", "origin"], label="git-publish-fetch", debug_on_fail=False)
    if rc:
        pcc_print("GITHUB PUBLISH: fetch failed; no commit or push performed.")
        return rc
    ahead, behind, remote_exists, counts_rc = _git_remote_counts(branch)
    if counts_rc:
        pcc_print("GITHUB PUBLISH: unable to compare local HEAD with origin; refusing publish.")
        return counts_rc
    if behind:
        if ahead:
            pcc_print(f"GITHUB PUBLISH BLOCKED: branch diverged (ahead {ahead}, behind {behind}).")
        else:
            pcc_print(f"GITHUB PUBLISH BLOCKED: origin/{branch} is {behind} commit(s) ahead.")
        pcc_print("No automatic merge, rebase, reset, or stash is permitted by PCC.")
        return 7

    if changed:
        # Fetch must not create a race window where a different source state gets staged.
        ready_after_fetch, reason_after_fetch, refreshed_state = git_publish_readiness()
        if not ready_after_fetch or refreshed_state.get("fingerprint") != state.get("fingerprint"):
            pcc_print("GITHUB PUBLISH BLOCKED AFTER FETCH:", reason_after_fetch)
            return 9
        # Stage only source paths whose index does not yet match the current working
        # tree. This is retry-safe after a partially completed publish: an already-
        # staged deletion no longer exists in either the worktree or index, so passing
        # it to `git add -A -- <path>` again can produce a fatal pathspec error.
        # Existing files with stale/partial index content still appear in the unstaged
        # diff and are re-staged from the certified working tree.
        unstaged_rc, unstaged_bytes = git_bytes(["diff", "--name-only", "-z", "--"])
        untracked_rc, untracked_bytes = git_bytes(["ls-files", "--others", "--exclude-standard", "-z"])
        if unstaged_rc or untracked_rc:
            pcc_print("GITHUB PUBLISH: unable to determine retry-safe staging candidates.")
            return unstaged_rc or untracked_rc

        def _decode_git_paths(raw: bytes) -> list[str]:
            return [
                part.decode("utf-8", errors="surrogateescape").replace("\\", "/")
                for part in raw.split(b"\0") if part
            ]

        stage_candidates = sorted(
            set(_decode_git_paths(unstaged_bytes) + _decode_git_paths(untracked_bytes)),
            key=str.lower,
        )
        changed_set = set(changed)
        unexpected_candidates = [rel for rel in stage_candidates if rel not in changed_set]
        if unexpected_candidates:
            pcc_print("GITHUB PUBLISH BLOCKED: staging candidates drifted after certification/fetch.")
            for rel in unexpected_candidates[:10]:
                pcc_print(f"   {rel}")
            return 9

        for start in range(0, len(stage_candidates), 48):
            batch = stage_candidates[start:start + 48]
            rc = run_stream(["git", "add", "-A", "--", *batch], label="git-stage-certified-source", debug_on_fail=False)
            if rc:
                return rc

        staged_rc, staged_bytes = git_bytes(["diff", "--cached", "--name-only", "-z", "HEAD", "--"])
        if staged_rc:
            return staged_rc
        staged = _decode_git_paths(staged_bytes)
        for rel in staged:
            error = git_publish_path_error(rel)
            if error:
                pcc_print("GITHUB PUBLISH BLOCKED:", error)
                return 8

        staged_set = set(staged)
        if staged_set != changed_set:
            missing = sorted(changed_set - staged_set, key=str.lower)
            extra = sorted(staged_set - changed_set, key=str.lower)
            pcc_print("GITHUB PUBLISH BLOCKED: staged source set does not exactly match the certified change set.")
            for rel in missing[:10]:
                pcc_print(f"   missing staged path: {rel}")
            for rel in extra[:10]:
                pcc_print(f"   unexpected staged path: {rel}")
            return 9
        if staged:
            if not message:
                message = f"Havenwild certified update {dt.datetime.now().astimezone():%Y-%m-%d %H:%M}"
            rc = run_stream(["git", "commit", "-m", message], label="git-commit-certified-source", debug_on_fail=False)
            if rc:
                return rc
            _, committed_head = capture(["git", "rev-parse", "HEAD"])
            gate = read_json(LAST_GATE, {}) or {}
            write_json(PUBLISH_STATE, {
                "schema": "havenwild.bevy.pcc.git_publish_state.v1",
                "status": "COMMITTED_PENDING_PUSH",
                "updatedAt": iso_now(),
                "gateId": gate.get("gateId"),
                "certifiedFingerprint": (gate.get("sourceControl") or {}).get("fingerprint"),
                "certifiedSourceFingerprint": (gate.get("sourceControl") or {}).get("sourceFingerprint"),
                "head": committed_head,
                "branch": branch,
                "origin": origin,
                "message": message,
            })
        else:
            pcc_print("GITHUB PUBLISH: no source differences remained after staging.")

    # Re-read ahead count after an optional commit.
    ahead, behind, remote_exists, counts_rc = _git_remote_counts(branch)
    if counts_rc or behind:
        pcc_print("GITHUB PUBLISH: remote relationship changed during publish; refusing push.")
        return counts_rc or 7
    if ahead == 0 and remote_exists:
        gate = read_json(LAST_GATE, {}) or {}
        _, current_head = capture(["git", "rev-parse", "HEAD"])
        write_json(PUBLISH_STATE, {
            "schema": "havenwild.bevy.pcc.git_publish_state.v1",
            "status": "PASS",
            "updatedAt": iso_now(),
            "gateId": gate.get("gateId"),
            "certifiedFingerprint": (gate.get("sourceControl") or {}).get("fingerprint"),
            "head": current_head,
            "branch": branch,
            "origin": origin,
            "message": None,
            "receipt": None,
            "note": "origin already matched the certified HEAD; no push was required",
        })
        pcc_print("GITHUB PUBLISH: origin is already current; certified HEAD verified.")
        return 0

    push_cmd = ["git", "push"]
    if not remote_exists:
        push_cmd += ["-u", "origin", branch]
    else:
        push_cmd += ["origin", branch]
    rc = run_stream(push_cmd, label="git-push-certified-source", debug_on_fail=False)
    if rc:
        return rc

    _, new_head = capture(["git", "rev-parse", "HEAD"])
    gate = read_json(LAST_GATE, {}) or {}
    receipt = {
        "schema": "havenwild.bevy.pcc.git_publish_receipt.v1",
        "publishedAt": iso_now(),
        "branch": branch,
        "origin": origin,
        "head": new_head,
        "gateId": gate.get("gateId"),
        "certifiedFingerprint": (gate.get("sourceControl") or {}).get("fingerprint"),
        "certifiedSourceFingerprint": (gate.get("sourceControl") or {}).get("sourceFingerprint"),
        "message": message,
    }
    receipt_path = RECEIPT_ROOT / f"{now_id()}_git_publish.json"
    write_json(receipt_path, receipt)
    write_json(PUBLISH_STATE, {
        "schema": "havenwild.bevy.pcc.git_publish_state.v1",
        "status": "PASS",
        "updatedAt": iso_now(),
        "gateId": gate.get("gateId"),
        "certifiedFingerprint": (gate.get("sourceControl") or {}).get("fingerprint"),
        "head": new_head,
        "branch": branch,
        "origin": origin,
        "message": message,
        "receipt": str(receipt_path.relative_to(ROOT)),
    })
    pcc_print("GITHUB PUBLISH: PASS")
    pcc_print("PUBLISH RECEIPT:", receipt_path)
    return 0


def source_publish_interactive() -> int:
    ready, reason, state = git_publish_readiness()
    pcc_print("\nCOMMIT + PUSH CERTIFIED SOURCE")
    pcc_print(" Status :", "READY" if ready else "BLOCKED")
    pcc_print(" Reason :", reason)

    gate = read_json(LAST_GATE, {}) or {}
    if not ready and gate.get("result") == "PASS":
        canonical = project_repository_url()
        if not state.get("insideWorktree"):
            pcc_print(f" Canonical GitHub : {canonical}")
            answer = input("Initialize/adopt the canonical GitHub repository now? [Y/n]: ").strip().lower()
            if answer in {"", "y", "yes"}:
                rc = git_bootstrap_interactive(preferred_url=canonical)
                if rc:
                    return rc
                ready, reason, state = git_publish_readiness()
        elif not state.get("origin"):
            pcc_print(f" Canonical GitHub : {canonical}")
            answer = input("Connect/adopt the canonical GitHub origin now? [Y/n]: ").strip().lower()
            if answer in {"", "y", "yes"}:
                rc = git_configure_origin_interactive(preferred_url=canonical)
                if rc:
                    return rc
                ready, reason, state = git_publish_readiness()

    pcc_print(" Status :", "READY" if ready else "BLOCKED")
    pcc_print(" Reason :", reason)
    if not ready:
        return 6
    changed = list(state.get("changedPaths") or [])
    if changed:
        default = f"Havenwild certified update {dt.datetime.now().astimezone():%Y-%m-%d %H:%M}"
        entered = input(f"Commit message [{default}]: ").strip()
        message = entered or default
    else:
        message = None
        pcc_print("No uncommitted source changes; PCC will push already-certified ahead commits if present.")
    answer = input(f"Publish certified source to origin/{state.get('branch')}? [y/N]: ").strip().lower()
    if answer not in {"y", "yes"}:
        pcc_print("GITHUB PUBLISH: cancelled; source was not changed.")
        return 0
    return source_publish(message)


def environment_snapshot() -> str:
    rows = [
        f"timestamp: {iso_now()}",
        f"root: {ROOT}",
        f"python: {sys.version}",
        f"python_executable: {sys.executable}",
        f"platform: {sys.platform}",
        f"WGPU_BACKEND: {os.environ.get('WGPU_BACKEND', '-')}",
        f"RUST_BACKTRACE: {os.environ.get('RUST_BACKTRACE', '-')}",
        f"STUDIO_NATIVE_FRAME: {os.environ.get('STUDIO_NATIVE_FRAME', '-')}",
    ]
    for tool in ("git", "cargo", "rustc", "rustup"):
        exe = shutil.which(tool)
        rows.append(f"{tool}_path: {exe or '-'}")
        if exe:
            _, out = capture([tool, "--version"])
            rows.append(f"{tool}_version: {out}")
    rustup = shutil.which("rustup")
    if rustup:
        _, active = capture([rustup, "show", "active-toolchain"])
        rows.append(f"rustup_active_toolchain: {active or '-'}")
    if os.name == "nt":
        powershell = shutil.which("powershell") or shutil.which("pwsh")
        if powershell:
            gpu_script = (
                "Get-CimInstance Win32_VideoController | "
                "Select-Object Name,DriverVersion,AdapterRAM | ConvertTo-Json -Compress"
            )
            os_script = (
                "Get-CimInstance Win32_OperatingSystem | "
                "Select-Object Caption,Version,BuildNumber,OSArchitecture | ConvertTo-Json -Compress"
            )
            _, gpu = capture([powershell, "-NoProfile", "-Command", gpu_script])
            _, os_info = capture([powershell, "-NoProfile", "-Command", os_script])
            rows.append(f"windows_os: {os_info or '-'}")
            rows.append(f"video_controllers: {gpu or '-'}")
    return "\n".join(rows) + "\n"


def make_debug_bundle(*, reason: str = "manual", job_log: Path | None = None,
                      receipt: Path | None = None) -> Path:
    """Create a diagnostics ZIP containing source/config/log evidence only."""
    ensure_dirs()
    bundle = DEBUG_ROOT / f"Havenwild_Bevy_DebugBundle_{now_id()}.zip"
    env_text = environment_snapshot()
    git_text = git_snapshot()
    with zipfile.ZipFile(bundle, "w", compression=zipfile.ZIP_DEFLATED, allowZip64=True) as z:
        z.writestr("bundle/REASON.txt", reason + "\n")
        z.writestr("bundle/ENVIRONMENT.txt", env_text)
        z.writestr("bundle/GIT_STATUS.txt", git_text)
        if LAST_JOB.is_file():
            z.write(LAST_JOB, "bundle/last_job.json")
        if LAST_GATE.is_file():
            z.write(LAST_GATE, "bundle/last_gate.json")
        if job_log and job_log.is_file():
            z.write(job_log, f"logs/{job_log.name}")
        if receipt and receipt.is_file():
            z.write(receipt, f"receipts/{receipt.name}")
        if _SESSION_PATH and _SESSION_PATH.is_file():
            z.write(_SESSION_PATH, f"logs/{_SESSION_PATH.name}")
        for path in sorted(RECEIPT_ROOT.glob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)[:12]:
            z.write(path, f"receipts/recent/{path.name}")
        gate = read_json(LAST_GATE, {}) or {}
        for phase in gate.get("phases", []):
            rel = phase.get("phaseReceipt")
            if not rel:
                continue
            phase_path = ROOT / rel
            if phase_path.is_file():
                z.write(phase_path, f"gates/phases/{phase_path.name}")
        for path in sorted(LOG_ROOT.glob("*.log"), key=lambda p: p.stat().st_mtime, reverse=True)[:8]:
            arc = f"logs/recent/{path.name}"
            if arc not in z.namelist():
                z.write(path, arc)
        for rel in [
            "Cargo.toml", "Cargo.lock", ".gitignore", "ForgePY.py", "ProjectControlCenter.py",
            "project/forgepy.project.json", "content/catalog/active_sheets.json",
            "content/catalog/core_source_manifest.json", "content/terrain/dual_grid_contract.v1.json",
            "content/terrain/recipes/summer_grass_void.v1.json",
            "content/terrain/recipes/summer_flatworld_runtime.v1.json",
            "content/terrain/recovery/historical_authority.v1.json",
            "content/terrain/recovery/summer_source_cells.v1.json",
            "content/terrain/previews/dg01_grass_void.semantic_lab.v1.json",
            "content/scenes/summer_river.scene.json",
            "src/main.rs", "src/atomic_file.rs", "src/document.rs", "src/editor_layout.rs",
            "src/source_catalog.rs", "src/semantic_lab.rs", "src/terrain.rs", "src/terrain_mapper.rs", "src/terrain_resolver.rs",
            "src/scene_v2.rs", "src/playtest.rs", "src/project_root.rs", "src/world_chunks.rs", "src/historical_evidence.rs",
            "content/mapping/recovered/source_registry.v1.json", "content/mapping/recovered/consolidation_audit.v1.json",
            "content/mapping/recovered/review_triage.v3.json", "content/mapping/recovered/review_ledger.v1.json",
            "tools/terrain_validate.py", "tools/recover_legacy_mapping.py", "tools/build_summer_flatworld_mapping.py", "tools/audit_consolidated_mapping.py",
            "tools/pcc_patch_selftest.py", "tools/pcc_git_workflow_selftest.py", "tools/package_source_policy_selftest.py", "tools/rust_source_audit.py", "tools/package_source.py",
        ]:
            p = ROOT / rel
            if p.is_file():
                z.write(p, f"source/{rel}")
        semantic_lab_local = ROOT / ".forgepy" / "semantic_lab.local.json"
        if semantic_lab_local.is_file():
            z.write(semantic_lab_local, "bundle/semantic_lab.local.json")
        layout_path = ROOT / ".forgepy" / "editor_layout.local.json"
        if layout_path.is_file():
            layout = read_json(layout_path, {}) or {}
            safe_layout = {
                "schema": layout.get("schema"),
                "terrainMapper": layout.get("terrainMapper"),
            }
            z.writestr("bundle/editor_layout.local.json", json.dumps(safe_layout, indent=2) + "\n")
        for p in sorted((ROOT / "docs").glob("*.md")) if (ROOT / "docs").is_dir() else []:
            z.write(p, f"source/docs/{p.name}")
        fixture_root = ROOT / "content" / "terrain" / "fixtures"
        for p in sorted(fixture_root.glob("*.json")) if fixture_root.is_dir() else []:
            z.write(p, f"source/content/terrain/fixtures/{p.name}")
    return bundle


def dashboard() -> int:
    pcc_print("\n" + "=" * 72)
    pcc_print(" HAVENWILD BEVY PROJECT CONTROL CENTER")
    pcc_print("=" * 72)
    pcc_print(f" Repository : {ROOT}")
    meta = read_json(PROJECT_META, {}) or {}
    pcc_print(f" Engine     : {meta.get('engine', {}).get('name', 'Bevy')} {meta.get('engine', {}).get('version', '?')}")
    pcc_print(f" PCC        : {PCC_VERSION}")
    pcc_print(f" ForgePY    : {meta.get('forgepyVersion', '?')}")
    pcc_print(f" Source     : {meta.get('sourceVersion', '?')}")
    last = read_json(LAST_JOB, {}) or {}
    if last:
        pcc_print(f" Last job   : {last.get('label', '?')} [{last.get('result', '?')}] {last.get('durationSeconds', '?')}s")
    else:
        pcc_print(" Last job   : none")
    gate = read_json(LAST_GATE, {}) or {}
    if gate:
        pcc_print(f" Last gate  : {gate.get('result', '?')} / {gate.get('durationSeconds', '?')}s")
    else:
        pcc_print(" Last gate  : none")
    recipe = read_json(ROOT / "content" / "terrain" / "recipes" / "summer_grass_void.v1.json", {}) or {}
    states = recipe.get("states", {})
    classified = sum(1 for entry in states.values() if entry.get("resolution") in ("sprite", "composite", "unsupported"))
    certified = sum(
        1
        for entry in states.values()
        if entry.get("resolution") in ("sprite", "composite", "unsupported")
        and entry.get("confidence") == "certified"
    )
    recovery = read_json(ROOT / "content" / "terrain" / "recovery" / "summer_source_cells.v1.json", {}) or {}
    recovered = recovery.get("recovery", {}).get("currentPixelReplayNonTransparent", "?")
    historical = recovery.get("recovery", {}).get("historicalNonTransparentMapped", "?")
    pcc_print(f" Summer map : {recovered}/{historical} non-transparent source cells recovered")
    pcc_print(f" DG recipe  : {classified}/16 classified / {certified}/16 certified")
    authority = read_json(ROOT / "content" / "assets" / "authority" / "elizawy_runtime_index.v1.json", {}) or {}
    counts = authority.get("counts", {})
    if authority.get("schema") == "havenwild.bevy.elizawy.runtime_index.v1":
        pcc_print(f" Asset lane : {counts.get('runtimeSourceImages', '?')} images / {counts.get('canonicalRuntimeRegions', '?')} exact regions / generated art OFF")
    world = read_json(ROOT / ".forgepy" / "world" / "havenwild_world.local.json", {}) or {}
    if world.get("schema") == "havenwild.bevy.world_composition.v1":
        pcc_print(f" World      : seed {world.get('seed', '?')} / {len(world.get('chunks', []))} chunks / rev {world.get('revision', '?')}")
    else:
        pcc_print(" World      : not materialized yet (Studio → HW → World Generator)")
    pending = discover_updates()
    pcc_print(f" Updates    : {len(pending)} pending governed patch(es)")
    if shutil.which("git"):
        rc, inside = capture(["git", "rev-parse", "--is-inside-work-tree"])
        if rc == 0 and inside.lower() == "true":
            _, branch = capture(["git", "branch", "--show-current"])
            _, status = capture(["git", "status", "--porcelain"])
            pcc_print(f" Git        : {branch or '(detached)'} / {'Modified' if status else 'Clean'}")
        else:
            pcc_print(" Git        : local folder (not a checkout yet)")
    else:
        pcc_print(" Git        : unavailable")
    asset_state = read_json(ROOT / ".forgepy" / "asset_state.json", {}) or {}
    chars = asset_state.get("characters", {})
    if chars.get("hydrated"):
        pcc_print(f" Characters : {chars.get('count', '?')} hydrated [local-only]")
    else:
        char_dir = ROOT / "assets" / "elizawy" / "Characters"
        pcc_print(f" Characters : {'tree present / state pending' if char_dir.is_dir() else 'not hydrated'}")
    pcc_print("-" * 72)
    return 0


def _zip_has_patch_manifest(path: Path) -> bool:
    """Return True when a ZIP advertises itself as a governed PCC patch.

    Root-drop intake is manifest-first rather than filename-first. This handles
    browser-renamed downloads such as ``foo.pccpatch (1).zip`` without treating
    ordinary source/debug ZIPs as patches. Explicit ``*.pccpatch.zip`` names are
    still surfaced even when corrupt so status can report them as INVALID.
    """
    try:
        with zipfile.ZipFile(path) as z:
            info = z.getinfo("pcc_patch.json")
            return not info.is_dir()
    except (OSError, zipfile.BadZipFile, KeyError):
        return False


def discover_updates() -> list[Path]:
    ensure_dirs()
    # Anything placed in updates/inbox is an explicit operator request and is
    # therefore surfaced for validation. Root ZIPs are discovered by internal
    # manifest, with the legacy canonical suffix retained as an INVALID-visible
    # hint if the archive itself is damaged.
    found = list(UPDATE_INBOX.glob("*.zip"))
    for candidate in ROOT.glob("*.zip"):
        lower = candidate.name.lower()
        if lower.endswith(".pccpatch.zip") or _zip_has_patch_manifest(candidate):
            found.append(candidate)
    unique = {}
    for item in found:
        try:
            unique[str(item.resolve()).lower()] = item
        except OSError:
            unique[str(item).lower()] = item
    return sorted(unique.values(), key=lambda item: item.name.lower())


def safe_patch_id(value: object) -> str:
    patch_id = str(value or "").strip()
    if not patch_id:
        raise ValueError("patch id is required")
    if not PATCH_ID_RE.fullmatch(patch_id):
        raise ValueError("patch id must be 1-96 characters using letters, numbers, dot, underscore or hyphen")
    if patch_id.upper().split(".", 1)[0] in WINDOWS_DEVICE_NAMES:
        raise ValueError(f"patch id is a reserved Windows device name: {patch_id}")
    return patch_id


def restart_current_pcc(command: list[str]) -> int:
    pcc_print("PCC SELF-UPDATE: control source changed; restarting into the updated controller.")
    try:
        if _SESSION_HANDLE:
            _SESSION_HANDLE.flush()
    except Exception:
        pass
    argv = [sys.executable, str(Path(__file__).resolve()), *command]
    try:
        os.execv(sys.executable, argv)
    except OSError as exc:
        pcc_print(f"PCC SELF-UPDATE RESTART FAILED: {exc}")
        return 253
    return 253


def installed_patch_state() -> dict[str, str]:
    meta = read_json(PROJECT_META, {}) or {}
    backend_text = BACKEND.read_text(encoding="utf-8", errors="replace") if BACKEND.is_file() else ""
    match = re.search(r'^FORGEPY_VERSION\s*=\s*["\']([^"\']+)', backend_text, re.MULTILINE)
    return {
        "source": str(meta.get("sourceVersion", "")).strip(),
        "forgepy": match.group(1) if match else str(meta.get("forgepyVersion", "")).strip(),
        "pcc": PCC_VERSION,
    }


def patch_compatibility_error(manifest: dict) -> str | None:
    required = manifest.get("from")
    if not isinstance(required, dict):
        return "patch manifest requires a from version object"
    current = installed_patch_state()
    for key in ("source", "forgepy", "pcc"):
        expected = str(required.get(key, "")).strip()
        if not expected:
            return f"patch manifest from.{key} is required"
        if current.get(key) != expected:
            return f"requires {key}={expected}, installed {key}={current.get(key) or '-'}"
    target = manifest.get("to")
    if not isinstance(target, dict) or any(not str(target.get(key, "")).strip() for key in ("source", "forgepy", "pcc")):
        return "patch manifest requires complete to source/forgepy/pcc versions"
    return None


def _numeric_version(value: object) -> tuple[int, ...] | None:
    text = str(value or "").strip()
    if not re.fullmatch(r"\d+(?:\.\d+)*", text):
        return None
    return tuple(int(part) for part in text.split("."))


def patch_target_state(manifest: dict) -> tuple[str, str]:
    """Classify a validated patch against the currently installed versions."""
    current = installed_patch_state()
    target = manifest.get("to") or {}
    required = manifest.get("from") or {}
    if all(str(target.get(key, "")).strip() == current.get(key) for key in ("source", "forgepy", "pcc")):
        return "target-current", "target versions are already installed"
    if all(str(required.get(key, "")).strip() == current.get(key) for key in ("source", "forgepy", "pcc")):
        return "from-current", "exact installed state matches patch source versions"

    comparisons = []
    for key in ("source", "forgepy", "pcc"):
        target_version = _numeric_version(target.get(key))
        current_version = _numeric_version(current.get(key))
        if target_version is None or current_version is None:
            return "other", patch_compatibility_error(manifest) or "version relation is ambiguous"
        comparisons.append((target_version, current_version))
    if all(target <= current_version for target, current_version in comparisons) and any(
        target < current_version for target, current_version in comparisons
    ):
        return "superseded", "patch target is older than the installed project state"
    return "other", patch_compatibility_error(manifest) or "patch does not match the installed project state"


def patch_payload_matches_installed(manifest: dict) -> tuple[bool, str]:
    """Verify manual extract/overwrite produced the exact declared target bytes."""
    for entry in manifest.get("files") or []:
        rel = safe_rel(str(entry.get("path", "")))
        dst = ensure_patch_destination(rel)
        if not dst.is_file():
            return False, f"target version is installed but patch file is missing: {rel.as_posix()}"
        actual = hashlib.sha256(dst.read_bytes()).hexdigest()
        if actual != str(entry.get("sha256", "")).lower():
            return False, f"target version is installed but patch bytes differ: {rel.as_posix()}"
    for raw in manifest.get("delete") or []:
        rel = safe_rel(str(raw))
        if ensure_patch_destination(rel).exists():
            return False, f"target version is installed but declared deletion remains: {rel.as_posix()}"
    return True, "manual extract/overwrite bytes match the governed manifest"


def _archive_nonpending_patch(path: Path, manifest: dict, *, status: str, reason: str) -> None:
    ensure_dirs()
    patch_id = safe_patch_id(manifest.get("id"))
    archived = UPDATE_ARCHIVE / f"{now_id()}_{path.name}"
    shutil.move(str(path), str(archived))
    receipt = {
        "schema": "havenwild.bevy.pcc.patch_reconcile_receipt.v1",
        "patchId": patch_id,
        "reconciledAt": iso_now(),
        "status": status,
        "reason": reason,
        "archive": str(archived.relative_to(ROOT)),
        "from": manifest.get("from"),
        "to": manifest.get("to"),
    }
    write_json(RECEIPT_ROOT / f"{now_id()}_patch_reconcile_{patch_id}.json", receipt)


def validate_patch_archive(path: Path, *, require_compatible: bool = True) -> tuple[dict, dict[str, zipfile.ZipInfo]]:
    with zipfile.ZipFile(path) as z:
        try:
            manifest_info = z.getinfo("pcc_patch.json")
            if manifest_info.file_size > 1024 * 1024:
                raise ValueError("pcc_patch.json exceeds 1 MiB")
            manifest = json.loads(z.read(manifest_info).decode("utf-8"))
        except KeyError as exc:
            raise ValueError("missing pcc_patch.json") from exc
        if manifest.get("schema") != "havenwild.bevy.pcc.patch.v1":
            raise ValueError("unsupported patch schema")
        patch_id = safe_patch_id(manifest.get("id"))
        manifest["id"] = patch_id
        if require_compatible:
            incompatibility = patch_compatibility_error(manifest)
            if incompatibility:
                raise ValueError(f"patch is not compatible with this project state: {incompatibility}")

        file_infos = [info for info in z.infolist() if not info.is_dir()]
        infos: dict[str, zipfile.ZipInfo] = {}
        archive_keys: set[str] = set()
        for info in file_infos:
            rel = safe_rel(info.filename)
            name = rel.as_posix()
            key = patch_path_key(rel)
            if key in archive_keys:
                raise ValueError(f"duplicate/colliding archive file name: {name}")
            archive_keys.add(key)
            infos[name] = info

        declared = manifest.get("files") or []
        if not isinstance(declared, list):
            raise ValueError("patch files must be a list")
        if len(declared) > MAX_PATCH_FILES:
            raise ValueError(f"patch declares too many files: {len(declared)} > {MAX_PATCH_FILES}")
        expected_names = {"pcc_patch.json"}
        declared_keys: set[str] = set()
        total_bytes = 0
        for entry in declared:
            if not isinstance(entry, dict):
                raise ValueError("each patch file declaration must be an object")
            rel = safe_rel(str(entry.get("path", "")))
            if not source_safe_patch_path(rel):
                raise ValueError(f"forbidden source path in patch: {rel.as_posix()}")
            ensure_patch_destination(rel)
            name = rel.as_posix()
            key = patch_path_key(rel)
            if key in declared_keys:
                raise ValueError(f"duplicate/colliding declared patch path: {name}")
            declared_keys.add(key)
            expected_names.add(name)
            if name not in infos:
                raise ValueError(f"declared file missing from archive: {name}")
            info = infos[name]
            if info.file_size > MAX_PATCH_FILE_BYTES:
                raise ValueError(f"patch file too large: {name} ({info.file_size} bytes)")
            expected_bytes = entry.get("bytes")
            if not isinstance(expected_bytes, int) or isinstance(expected_bytes, bool) or expected_bytes < 0:
                raise ValueError(f"patch file requires integer bytes metadata: {name}")
            if expected_bytes != info.file_size:
                raise ValueError(f"byte-count mismatch: {name} manifest={expected_bytes} archive={info.file_size}")
            total_bytes += info.file_size
            if total_bytes > MAX_PATCH_TOTAL_BYTES:
                raise ValueError(f"patch payload exceeds {MAX_PATCH_TOTAL_BYTES} bytes")
            expected_sha = str(entry.get("sha256") or "").lower()
            if not re.fullmatch(r"[0-9a-f]{64}", expected_sha):
                raise ValueError(f"patch file requires a full sha256: {name}")
            actual = hashlib.sha256(z.read(info)).hexdigest()
            if actual != expected_sha:
                raise ValueError(f"hash mismatch: {name}")

        extra = set(infos) - expected_names
        if extra:
            raise ValueError("undeclared files in patch: " + ", ".join(sorted(extra)[:8]))

        delete_values = manifest.get("delete") or []
        if not isinstance(delete_values, list):
            raise ValueError("patch delete must be a list")
        delete_keys: set[str] = set()
        for raw in delete_values:
            rel = safe_rel(str(raw))
            if not source_safe_patch_path(rel):
                raise ValueError(f"forbidden delete path in patch: {rel.as_posix()}")
            ensure_patch_destination(rel)
            name = rel.as_posix()
            key = patch_path_key(rel)
            if key in delete_keys:
                raise ValueError(f"duplicate/colliding delete path in patch: {name}")
            if key in declared_keys:
                raise ValueError(f"patch cannot both write and delete the same Windows path: {name}")
            delete_keys.add(key)
        return manifest, infos


def apply_patch(path: Path) -> int:
    global _SELF_UPDATE_APPLIED
    pcc_print(f"PATCH: validating {path}")
    try:
        manifest, _ = validate_patch_archive(path)
    except Exception as exc:
        pcc_print(f"PATCH REJECTED: {exc}")
        ensure_dirs()
        target = UPDATE_REJECTED / f"{now_id()}_{path.name}"
        try:
            shutil.move(str(path), str(target))
            pcc_print(f"PATCH MOVED: {target}")
        except OSError:
            pass
        return 4

    patch_id = safe_patch_id(manifest["id"])
    stage = STAGING_ROOT / patch_id
    rollback = ROLLBACK_ROOT / f"{now_id()}_{patch_id}"
    shutil.rmtree(stage, ignore_errors=True)
    stage.mkdir(parents=True, exist_ok=True)
    rollback.mkdir(parents=True, exist_ok=True)
    created: list[Path] = []
    replaced: list[Path] = []
    deleted: list[Path] = []

    try:
        with zipfile.ZipFile(path) as z:
            for entry in manifest.get("files") or []:
                rel = safe_rel(str(entry["path"]))
                dst = stage / rel
                dst.parent.mkdir(parents=True, exist_ok=True)
                dst.write_bytes(z.read(z.getinfo(rel.as_posix())))
        # Backup every destination before mutating anything.
        for entry in manifest.get("files") or []:
            rel = safe_rel(str(entry["path"]))
            dst = ensure_patch_destination(rel)
            if dst.exists():
                backup = rollback / rel
                backup.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(dst, backup)
                replaced.append(rel)
            else:
                created.append(rel)
        for raw in manifest.get("delete") or []:
            rel = safe_rel(str(raw))
            dst = ensure_patch_destination(rel)
            if dst.is_file():
                backup = rollback / rel
                backup.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(dst, backup)
                deleted.append(rel)

        for entry in manifest.get("files") or []:
            rel = safe_rel(str(entry["path"]))
            src = stage / rel
            dst = ensure_patch_destination(rel)
            dst.parent.mkdir(parents=True, exist_ok=True)
            tmp = dst.with_suffix(dst.suffix + ".pcc-new")
            shutil.copy2(src, tmp)
            tmp.replace(dst)
        for rel in deleted:
            ensure_patch_destination(rel).unlink(missing_ok=True)

        # Verify the installed bytes before the patch leaves the inbox. A receipt
        # must never claim success for a partially promoted update.
        for entry in manifest.get("files") or []:
            rel = safe_rel(str(entry["path"]))
            dst = ensure_patch_destination(rel)
            if not dst.is_file():
                raise RuntimeError(f"post-apply file missing: {rel.as_posix()}")
            actual = hashlib.sha256(dst.read_bytes()).hexdigest()
            if actual != str(entry["sha256"]).lower():
                raise RuntimeError(f"post-apply hash mismatch: {rel.as_posix()}")
        for raw in manifest.get("delete") or []:
            rel = safe_rel(str(raw))
            if ensure_patch_destination(rel).exists():
                raise RuntimeError(f"post-apply delete failed: {rel.as_posix()}")

        archived = UPDATE_ARCHIVE / f"{now_id()}_{path.name}"
        shutil.move(str(path), str(archived))
        receipt = {
            "schema": "havenwild.bevy.pcc.patch_receipt.v1",
            "patchId": patch_id,
            "appliedAt": iso_now(),
            "archive": str(archived.relative_to(ROOT)),
            "rollback": str(rollback.relative_to(ROOT)),
            "from": manifest.get("from"),
            "to": manifest.get("to"),
            "files": [
                {"path": e["path"], "bytes": e["bytes"], "sha256": e["sha256"]}
                for e in manifest.get("files") or []
            ],
            "deleted": [str(x) for x in manifest.get("delete") or []],
        }
        write_json(RECEIPT_ROOT / f"{now_id()}_patch_{patch_id}.json", receipt)
        changed_paths = {str(entry.get("path", "")).replace("\\", "/") for entry in manifest.get("files") or []}
        changed_paths.update(str(value).replace("\\", "/") for value in manifest.get("delete") or [])
        if changed_paths & PCC_RESTART_PATHS:
            _SELF_UPDATE_APPLIED = True
        pcc_print(f"PATCH APPLIED: {patch_id} ({len(receipt['files'])} file(s))")
        return 0
    except Exception:
        pcc_print(f"PATCH FAILED: {patch_id}; rolling back")
        pcc_print(traceback.format_exc())
        # Remove files created by the failed patch.
        for rel in created:
            (ROOT / rel).unlink(missing_ok=True)
        # Restore replaced/deleted files from rollback.
        for rel in set(replaced + deleted):
            backup = rollback / rel
            if backup.is_file():
                dst = ROOT / rel
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(backup, dst)
        return 5
    finally:
        shutil.rmtree(stage, ignore_errors=True)


def updates_status() -> int:
    pending = discover_updates()
    if not pending:
        pcc_print("UPDATE INBOX: no governed patches pending")
        pcc_print("ROOT ZIP INTAKE: manifest-first; browser-renamed governed ZIPs are supported.")
        pcc_print("MANUAL FALLBACK: extract/overwrite is supported; run Full Gate afterward to certify the resulting source.")
        return 0
    pcc_print(f"UPDATE INBOX: {len(pending)} governed patch candidate(s)")
    current = installed_patch_state()
    pcc_print(f"INSTALLED: source={current['source']} ForgePY={current['forgepy']} PCC={current['pcc']}")
    for patch in pending:
        try:
            manifest, _ = validate_patch_archive(patch, require_compatible=False)
            relation, reason = patch_target_state(manifest)
            if relation == "from-current":
                state = "READY"
            elif relation == "target-current":
                matches, detail = patch_payload_matches_installed(manifest)
                state = "ALREADY INSTALLED / RECONCILABLE" if matches else f"BLOCKED ({detail})"
            elif relation == "superseded":
                state = "SUPERSEDED / ARCHIVABLE"
            else:
                state = f"BLOCKED ({reason})"
            target = manifest.get("to") or {}
            pcc_print(
                f"  {patch.name}: {state} -> "
                f"source={target.get('source','-')} ForgePY={target.get('forgepy','-')} PCC={target.get('pcc','-')}"
            )
        except Exception as exc:
            pcc_print(f"  {patch.name}: INVALID ({exc})")
    pcc_print("Root ZIP discovery uses the internal pcc_patch.json manifest, not only the filename.")
    pcc_print("Manual extract/overwrite remains supported; exact target bytes are reconciled instead of re-applied.")
    pcc_print("Assets/reference/generated local catalogs remain forbidden patch destinations.")
    return 0


def updates_apply_all() -> int:
    # Apply exactly one compatible patch per controller generation. Already-manually
    # installed patches are verified byte-for-byte and archived without reapplying;
    # older superseded root handoffs are archived so they cannot poison Full Gate.
    pending = discover_updates()
    if not pending:
        pcc_print("UPDATE INBOX: no governed patches pending")
        return 0

    compatible: list[tuple[Path, dict]] = []
    invalid: list[tuple[Path, str]] = []
    blocked: list[tuple[Path, str]] = []
    reconciled: list[tuple[Path, dict, str]] = []
    superseded: list[tuple[Path, dict, str]] = []
    for patch in pending:
        try:
            manifest, _ = validate_patch_archive(patch, require_compatible=False)
        except Exception as exc:
            invalid.append((patch, str(exc)))
            continue
        relation, reason = patch_target_state(manifest)
        if relation == "from-current":
            compatible.append((patch, manifest))
        elif relation == "target-current":
            matches, detail = patch_payload_matches_installed(manifest)
            if matches:
                reconciled.append((patch, manifest, detail))
            else:
                blocked.append((patch, detail))
        elif relation == "superseded":
            superseded.append((patch, manifest, reason))
        else:
            blocked.append((patch, reason))

    if invalid:
        for patch, reason in invalid:
            pcc_print(f"PATCH REJECTED: {patch.name}: {reason}")
            target = UPDATE_REJECTED / f"{now_id()}_{patch.name}"
            try:
                shutil.move(str(patch), str(target))
            except OSError:
                pass
        return 4

    for patch, manifest, reason in reconciled:
        pcc_print(f"PATCH RECONCILED: {patch.name}: {reason}")
        _archive_nonpending_patch(patch, manifest, status="ALREADY_INSTALLED", reason=reason)
    for patch, manifest, reason in superseded:
        pcc_print(f"PATCH SUPERSEDED: {patch.name}: {reason}")
        _archive_nonpending_patch(patch, manifest, status="SUPERSEDED", reason=reason)

    if len(compatible) > 1:
        names = ", ".join(path.name for path, _ in compatible)
        pcc_print(f"UPDATE BLOCKED: multiple patches claim the same installed state: {names}")
        return 5
    if not compatible:
        if blocked:
            pcc_print("UPDATE BLOCKED: no pending patch matches the installed source/ForgePY/PCC state")
            for patch, reason in blocked:
                pcc_print(f"  {patch.name}: {reason}")
            return 5
        if reconciled or superseded:
            pcc_print("UPDATE INBOX: reconciled/archived; no patch application required")
        else:
            pcc_print("UPDATE INBOX: no compatible patch requires application")
        return 0

    patch, _ = compatible[0]
    return apply_patch(patch)


def python_source_check() -> int:
    # Compile every source-controlled Python helper, not only the primary PCC/backend.
    # This keeps compatibility wrappers and future tooling from escaping FULL.
    files = [BACKEND, Path(__file__).resolve()]
    tools_root = ROOT / "tools"
    if tools_root.is_dir():
        files.extend(sorted(path for path in tools_root.glob("*.py") if path.is_file()))
    # Preserve stable order while avoiding duplicate primary helper paths.
    unique = list(dict.fromkeys(path.resolve() for path in files))
    return run_stream(
        [python_cmd(), "-m", "py_compile", *[str(path) for path in unique]],
        label="python-source-check",
        debug_on_fail=False,
    )


def write_gate_receipt(gate_id: str, started_at: str, started: float,
                       phases: list[dict], result: str, exit_code: int) -> Path:
    payload = {
        "schema": "havenwild.bevy.pcc.gate_receipt.v1",
        "pccVersion": PCC_VERSION,
        "gateId": gate_id,
        "startedAt": started_at,
        "finishedAt": iso_now(),
        "durationSeconds": round(time.monotonic() - started, 3),
        "result": result,
        "exitCode": exit_code,
        "phases": phases,
        "sourceControl": git_worktree_state(),
    }
    path = GATE_ROOT / f"{gate_id}.json"
    write_json(path, payload)
    payload["receipt"] = str(path.relative_to(ROOT))
    write_json(LAST_GATE, payload)
    return path


def project_contract_check() -> int:
    meta = read_json(PROJECT_META, {}) or {}
    errors: list[str] = []
    source_version = str(meta.get("sourceVersion", "")).strip()
    forge_version = str(meta.get("forgepyVersion", "")).strip()
    pcc_version = str((meta.get("pcc") or {}).get("version", "")).strip()
    cargo_text = (ROOT / "Cargo.toml").read_text(encoding="utf-8", errors="replace") if (ROOT / "Cargo.toml").is_file() else ""
    backend_text = BACKEND.read_text(encoding="utf-8", errors="replace") if BACKEND.is_file() else ""

    match = re.search(r'^FORGEPY_VERSION\s*=\s*["\']([^"\']+)', backend_text, re.MULTILINE)
    backend_version = match.group(1) if match else ""
    if not source_version:
        errors.append("project metadata sourceVersion is missing")
    package_match = re.search(r'^version\s*=\s*"([^"]+)"', cargo_text, re.MULTILINE)
    package_version = package_match.group(1) if package_match else ""
    if source_version and package_version != source_version:
        errors.append(f"Cargo.toml package version {package_version or '-'} != source version {source_version}")
    if pcc_version != PCC_VERSION:
        errors.append(f"project metadata PCC {pcc_version or '-'} != controller {PCC_VERSION}")
    if not backend_version or forge_version != backend_version:
        errors.append(f"project metadata ForgePY {forge_version or '-'} != backend {backend_version or '-'}")

    engine = meta.get("engine") or {}
    bevy_version = str(engine.get("version", "")).strip()
    rust_min = str(engine.get("rustMinimum", "")).strip()
    gui = meta.get("gui") or {}
    forge_rev = str(gui.get("forgeGuiRevision", "")).strip()
    bevy_egui_version = str(gui.get("bevyEguiVersion", "")).strip()
    if bevy_version and f'bevy = "={bevy_version}"' not in cargo_text:
        errors.append(f"Cargo.toml Bevy pin does not match metadata {bevy_version}")
    if bevy_egui_version and f'bevy_egui = "={bevy_egui_version}"' not in cargo_text:
        errors.append(f"Cargo.toml bevy_egui pin does not match metadata {bevy_egui_version}")
    if not bevy_egui_version:
        errors.append("project metadata gui.bevyEguiVersion is missing")
    if rust_min and f'rust-version = "{rust_min}"' not in cargo_text:
        errors.append(f"Cargo.toml rust-version does not match metadata {rust_min}")
    if forge_rev and forge_rev not in cargo_text:
        errors.append("Cargo.toml ForgeGUI revision does not match project metadata")

    source_control = meta.get("sourceControl") or {}
    repository_url = str(source_control.get("repositoryUrl", "")).strip()
    if not repository_url or repository_url == "auto-detect-from-git-origin":
        errors.append("project metadata sourceControl.repositoryUrl must declare the canonical GitHub repository")
    elif not repository_url.startswith("https://github.com/"):
        errors.append(f"project metadata sourceControl.repositoryUrl is not a GitHub HTTPS URL: {repository_url}")

    terrain = meta.get("terrain") or {}
    for key in ("contract", "initialRecipeFamily", "summerRecovery", "historicalAuthority", "semanticLabSeed", "runtimeResolver"):
        rel = terrain.get(key)
        if rel and not (ROOT / str(rel)).is_file():
            errors.append(f"metadata terrain.{key} target missing: {rel}")

    if errors:
        for error in errors:
            pcc_print(f"[FAIL] {error}")
        return 2
    pcc_print(
        f"[OK] project contract source={source_version} ForgePY={forge_version} PCC={pcc_version} "
        f"Bevy={bevy_version} bevy_egui={bevy_egui_version} rust>={rust_min} ForgeGUI={forge_rev[:12]}"
    )
    return 0


def authored_scene_check() -> int:
    meta = read_json(PROJECT_META, {}) or {}
    scene_rel = str(((meta.get("authoredContent") or {}).get("initialScene")) or "").strip()
    manifest_rel = str(((meta.get("assets") or {}).get("coreManifest")) or "").strip()
    if not scene_rel or not manifest_rel:
        pcc_print("[FAIL] project metadata must define authoredContent.initialScene and assets.coreManifest")
        return 2
    scene_path = ROOT / scene_rel
    manifest_path = ROOT / manifest_rel
    try:
        scene = read_json(scene_path, None)
        manifest = read_json(manifest_path, None)
        if not isinstance(scene, dict) or not isinstance(manifest, dict):
            raise ValueError("scene/manifest JSON root must be an object")
        size = scene.get("size")
        if not isinstance(size, list) or len(size) != 2:
            raise ValueError("scene size must be [width,height]")
        width, height = int(size[0]), int(size[1])
        tile_size = int(scene.get("tile_size", 0))
        tiles = scene.get("tiles")
        if scene.get("schema") != "havenwild.elizawy.scene.v1":
            raise ValueError(f"unsupported scene schema: {scene.get('schema')}")
        if width <= 0 or height <= 0 or tile_size != 32:
            raise ValueError(f"invalid scene geometry {width}x{height} tile_size={tile_size}")
        if not isinstance(tiles, list) or len(tiles) != width * height:
            raise ValueError(f"scene tile count {len(tiles) if isinstance(tiles,list) else '-'} != {width * height}")

        entries = manifest.get("entries") or []
        assets = {str(entry.get("path")): entry for entry in entries if isinstance(entry, dict)}
        sourced = 0
        unsourced = 0
        used_assets: set[str] = set()
        errors: list[str] = []
        for index, tile in enumerate(tiles):
            if not isinstance(tile, dict):
                errors.append(f"tile {index}: record must be an object")
                continue
            source = str(tile.get("source_asset") or "")
            rect = tile.get("source_rect")
            if not source:
                unsourced += 1
                if rect not in ([0, 0, 0, 0], None):
                    errors.append(f"tile {index}: unsourced tile has non-empty source_rect {rect}")
                continue
            sourced += 1
            used_assets.add(source)
            entry = assets.get(source)
            if entry is None:
                errors.append(f"tile {index}: source asset is not in core manifest: {source}")
                continue
            if not isinstance(rect, list) or len(rect) != 4:
                errors.append(f"tile {index}: source_rect must contain four integers")
                continue
            try:
                x, y, w, h = [int(value) for value in rect]
            except (TypeError, ValueError):
                errors.append(f"tile {index}: source_rect is not integer-valued: {rect}")
                continue
            if min(x, y) < 0 or w <= 0 or h <= 0:
                errors.append(f"tile {index}: invalid source_rect {rect}")
                continue
            dims = entry.get("imageSizePx")
            if not isinstance(dims, list) or len(dims) != 2:
                errors.append(f"tile {index}: manifest lacks image dimensions for {source}")
                continue
            image_w, image_h = int(dims[0]), int(dims[1])
            if x + w > image_w or y + h > image_h:
                errors.append(f"tile {index}: source_rect {rect} exceeds {source} {image_w}x{image_h}")

        if errors:
            for error in errors[:24]:
                pcc_print(f"[FAIL] {error}")
            if len(errors) > 24:
                pcc_print(f"[FAIL] ... {len(errors) - 24} additional scene error(s)")
            return 2
        pcc_print(
            f"[OK] authored scene {scene_rel}: {width}x{height}={len(tiles)} tiles, "
            f"{sourced} sourced, {unsourced} source-unbound, {len(used_assets)} source sheet(s)"
        )
        return 0
    except Exception as exc:
        pcc_print(f"[FAIL] authored scene validation: {exc}")
        return 2


def cargo_lock_check() -> int:
    lock = ROOT / "Cargo.lock"
    if not lock.is_file():
        pcc_print("[MISSING] Cargo.lock after successful Cargo build; reproducible dependency snapshot was not produced")
        return 2
    digest = hashlib.sha256(lock.read_bytes()).hexdigest()
    pcc_print(f"[OK] Cargo.lock {lock.stat().st_size} bytes sha256={digest}")
    return 0


def patch_validator_selftest() -> int:
    return run_stream(
        [python_cmd(), str(PATCH_SELFTEST)],
        label="pcc-patch-validator-selftest",
        debug_on_fail=False,
    )


def rust_source_audit() -> int:
    return run_stream(
        [python_cmd(), str(RUST_SOURCE_AUDIT)],
        label="rust-source-audit",
        debug_on_fail=False,
    )


def elizawy_history_gate() -> int:
    # Source-only checkouts may not contain recovered workstation-local data.
    # Explicit SKIP is not a claim of historical-source certification.
    index = ROOT / ".forgepy/recovered_mapping/index.json"
    if not index.is_file():
        pcc_print("ELIZAWY HISTORY: SKIP / recovery index absent; use PCC terrain recover; no historic mapping authority certified")
        return 0
    return run_stream(
        [python_cmd(), str(ELIZAWY_AUDIT), "--root", str(ROOT)],
        label="elizawy-historical-source-audit", debug_on_fail=False,
    )


def elizawy_derivative_gate() -> int:
    # A generated local report/registry is only derivative review evidence.
    # No third-party ZIP becomes a required Cargo/source dependency.
    return run_stream(
        [python_cmd(), str(ELIZAWY_DERIVATIVE_GATE), "--root", str(ROOT)],
        label="elizawy-derivative-evidence-verify", debug_on_fail=False,
    )


def audit_gate() -> int:
    pcc_print("=" * 72)
    pcc_print(" STATIC PROJECT AUDIT — NO CARGO / NO RUNTIME LAUNCH")
    pcc_print("=" * 72)
    checks = [
        ("Python source syntax", python_source_check),
        ("Project metadata / dependency contract", project_contract_check),
        ("Governed patch adversarial validator", patch_validator_selftest),
        ("PCC certified Git publish workflow", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/pcc_git_workflow_selftest.py")],
            label="pcc-git-workflow-selftest", debug_on_fail=False)),
        ("Source-only package boundary", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/package_source_policy_selftest.py")],
            label="package-source-policy-selftest", debug_on_fail=False)),
        ("GitHub-first asset hydration policy", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/asset_hydration_policy_selftest.py")],
            label="asset-hydration-policy-selftest", debug_on_fail=False)),
        ("Rust source structural audit", rust_source_audit),
        ("ElizaWy M2C2 unit tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_m2c2_selftest.py")], label="elizawy-m2c2-selftest", debug_on_fail=False)),
        ("ElizaWy M2C3 unit tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_m2c3_selftest.py")], label="elizawy-m2c3-selftest", debug_on_fail=False)),
        ("ElizaWy M2C4 unit tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_m2c4_selftest.py")], label="elizawy-m2c4-selftest", debug_on_fail=False)),
        ("ElizaWy M2C4 derivative-gate tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_derivative_gate_selftest.py")], label="elizawy-derivative-gate-selftest", debug_on_fail=False)),
        ("M2D04 original Summer scene/packaging regression", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d04_summer_scene_selftest.py")],
            label="m2d04-summer-scene-selftest", debug_on_fail=False)),
        ("Consolidated ElizaWy historical evidence (zero promotions)", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/audit_consolidated_mapping.py")],
            label="elizawy-consolidated-source-audit", debug_on_fail=False)),
        ("Unified ElizaWy asset authority deterministic check", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_elizawy_asset_lane.py"), "--root", str(ROOT), "--check"],
            label="elizawy-asset-lane-check", debug_on_fail=False)),
        ("M2D07 unified asset lane contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d07_asset_lane_selftest.py")], label="m2d07-asset-lane-selftest", debug_on_fail=False)),
        ("M2D07 world composition contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d07_world_contract_selftest.py")], label="m2d07-world-contract-selftest", debug_on_fail=False)),
        ("M2D080 Native Summer source authority", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d080_native_summer_authority_selftest.py")], label="m2d080-native-summer-selftest", debug_on_fail=False)),
        ("M2D081C Summer profile / static-water authority check", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_summer_complete_runtime.py"), "--check"], label="m2d081-summer-profile-check", debug_on_fail=False)),
        ("M2D081C Summer completion + water authority repair", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d081_summer_complete_selftest.py")], label="m2d081-summer-complete-selftest", debug_on_fail=False)),
        ("M2D082 Summer cliff source authority deterministic check", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_cliff_summer_authority.py"), "--check"], label="m2d082-cliff-source-check", debug_on_fail=False)),
        ("M2D082 Summer cliff source authority contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082_cliff_source_authority_selftest.py")], label="m2d082-cliff-source-selftest", debug_on_fail=False)),
        ("M2D082C complete ElizaWy worldgen source catalog", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_worldgen_asset_catalog.py"), "--check"], label="m2d082c-worldgen-asset-catalog", debug_on_fail=False)),
        ("M2D082C Generated World consolidation / shoreline / tree contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082c_generated_world_selftest.py")], label="m2d082c-generated-world-selftest", debug_on_fail=False)),
        ("M2D082D dual-grid semantic/render coordinate authority", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082d_dual_grid_alignment_selftest.py")], label="m2d082d-dual-grid-alignment-selftest", debug_on_fail=False)),
        ("M2D082F land-connectivity + extended Summer materials", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082f_land_connectivity_materials_selftest.py")], label="m2d082f-land-connectivity-materials-selftest", debug_on_fail=False)),
        ("M2D090 ElizaWy kitchen-sink Generated World", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d090_kitchen_sink_world_selftest.py")], label="m2d090-kitchen-sink-world-selftest", debug_on_fail=False)),
        ("Authored scene source-address integrity", authored_scene_check),
        ("DG contract + fixtures", lambda: run_terrain(["fixtures"], label="audit-terrain-fixtures", debug_on_fail=False)),
        ("DG semantic resolver lab", lambda: run_terrain(["resolver"], label="audit-terrain-resolver", debug_on_fail=False)),
        ("DG resolved semantic-to-recipe pipeline", lambda: run_terrain(["pipeline"], label="audit-terrain-pipeline", debug_on_fail=False)),
        ("Summer mapping recovery", lambda: run_terrain(["recovery"], label="audit-terrain-recovery", debug_on_fail=False)),
        ("Full core asset hash verification", lambda: run_backend(["assets", "status", "--full"], label="audit-assets-full", debug_on_fail=False)),
        ("ElizaWy historical provenance/source artwork", elizawy_history_gate),
        ("ElizaWy external/complete-review evidence", elizawy_derivative_gate),
    ]
    for name, action in checks:
        pcc_print(f"\n>>> AUDIT: {name}")
        rc = action()
        if rc:
            pcc_print(f"STATIC AUDIT: FAIL at {name}")
            try:
                pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason=f"static-audit:{name}"))
            except Exception:
                pass
            return rc
    pcc_print("STATIC AUDIT: PASS")
    pcc_print("NOTE: Cargo compile/test/runtime remain separate Windows certification steps.")
    return 0


def full_gate() -> int:
    gate_id = f"{now_id()}_full"
    started_at = iso_now()
    started = time.monotonic()
    phases: list[dict] = []
    pcc_print("=" * 72)
    pcc_print(" FULL QUALITY GATE — PHASED / LIVE / RECEIPTED")
    pcc_print(f" GATE ID: {gate_id}")
    pcc_print("=" * 72)

    def phase(name: str, action) -> int:
        index = len(phases) + 1
        phase_id = f"{gate_id}:P{index:02d}"
        pcc_print(f"\n>>> PHASE {index}: {name} [{phase_id}]")
        before = time.monotonic()
        phase_started = iso_now()
        previous = read_json(LAST_JOB, {}) or {}
        previous_receipt = previous.get("receipt")
        rc = action()
        last = read_json(LAST_JOB, {}) or {}
        current_receipt = last.get("receipt")
        owns_job = bool(current_receipt and current_receipt != previous_receipt)
        entry = {
            "phaseId": phase_id,
            "name": name,
            "startedAt": phase_started,
            "finishedAt": iso_now(),
            "exitCode": rc,
            "result": "PASS" if rc == 0 else "FAIL",
            "durationSeconds": round(time.monotonic() - before, 3),
            "jobReceipt": current_receipt if owns_job else None,
            "jobLog": last.get("jobLog") if owns_job else None,
        }
        phase_receipt = GATE_ROOT / f"{gate_id}_phase_{index:02d}.json"
        write_json(phase_receipt, entry)
        entry["phaseReceipt"] = str(phase_receipt.relative_to(ROOT))
        phases.append(entry)
        return rc

    # Root-drop updates remain first authority. If PCC updates itself, restart
    # immediately before evaluating the rest of the deployment.
    rc = phase("Apply governed updates", updates_apply_all)
    if rc:
        receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
        pcc_print("FULL GATE: FAIL")
        pcc_print("GATE RECEIPT:", receipt)
        try:
            pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason="full-gate:Apply governed updates"))
        except Exception:
            pass
        return rc
    if _SELF_UPDATE_APPLIED:
        receipt = write_gate_receipt(gate_id, started_at, started, phases, "RESTART", 0)
        pcc_print("FULL GATE: restarting immediately after PCC self-update")
        pcc_print("GATE RECEIPT:", receipt)
        return restart_current_pcc(["full"])

    # Source-only deployments intentionally omit hydrated art. Resolve this
    # before spending time on the complete source/Cargo certification chain.
    rc = phase(
        "Asset manifest completeness preflight",
        lambda: run_backend(["assets", "status", "--complete"], label="asset-status-preflight", debug_on_fail=False),
    )
    if rc:
        rc = phase(
            "Asset hydration/repair — pinned GitHub first",
            lambda: run_backend(["assets", "sync"], label="asset-sync-preflight", debug_on_fail=False),
        )
        if rc:
            receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
            pcc_print("FULL GATE: BLOCKED — pinned ElizaWy GitHub hydration failed and no valid local fallback was available.")
            pcc_print("Default authority is the exact pinned GitHub revision; normally no source path is required on a fresh deployment.")
            pcc_print("Fallback only: PCC -> 6 Assets / hydration -> 3 Local fallback sync, or PCC.cmd assets sync --source \"D:\\path\\to\\asset-source\" --local-only")
            pcc_print("GATE RECEIPT:", receipt)
            return rc

    # Do not allow a partial or merely-present asset tree past the bootstrap
    # boundary. A successful repair must immediately prove all manifest bytes
    # before expensive source/build certification begins.
    rc = phase(
        "Core asset identity verification",
        lambda: run_backend(["assets", "status", "--full"], label="asset-verify-preflight", debug_on_fail=False),
    )
    if rc:
        receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
        pcc_print("FULL GATE: BLOCKED — core asset manifest is not 349/349 verified after hydration/repair.")
        pcc_print("Run PCC -> 6 Assets / hydration -> 2 Hydrate / repair core assets, then rerun Full Gate.")
        pcc_print("GATE RECEIPT:", receipt)
        return rc

    phase_specs = [
        ("Python source syntax", python_source_check),
        ("Project metadata / dependency contract", project_contract_check),
        ("Governed patch adversarial validator", patch_validator_selftest),
        ("PCC certified Git publish workflow", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/pcc_git_workflow_selftest.py")],
            label="pcc-git-workflow-selftest", debug_on_fail=False)),
        ("Source-only package boundary", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/package_source_policy_selftest.py")],
            label="package-source-policy-selftest", debug_on_fail=False)),
        ("GitHub-first asset hydration policy", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/asset_hydration_policy_selftest.py")],
            label="asset-hydration-policy-selftest", debug_on_fail=False)),
        ("Rust source structural audit", rust_source_audit),
        ("ElizaWy M2C2 unit tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_m2c2_selftest.py")], label="elizawy-m2c2-selftest", debug_on_fail=False)),
        ("ElizaWy M2C3 unit tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_m2c3_selftest.py")], label="elizawy-m2c3-selftest", debug_on_fail=False)),
        ("ElizaWy M2C4 unit tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_m2c4_selftest.py")], label="elizawy-m2c4-selftest", debug_on_fail=False)),
        ("ElizaWy M2C4 derivative-gate tests", lambda: run_stream([python_cmd(), str(ROOT / "tools/elizawy_derivative_gate_selftest.py")], label="elizawy-derivative-gate-selftest", debug_on_fail=False)),
        ("M2D04 original Summer scene/packaging regression", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d04_summer_scene_selftest.py")],
            label="m2d04-summer-scene-selftest", debug_on_fail=False)),
        ("Consolidated ElizaWy historical evidence (zero promotions)", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/audit_consolidated_mapping.py")],
            label="elizawy-consolidated-source-audit", debug_on_fail=False)),
        ("Unified ElizaWy asset authority deterministic check", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_elizawy_asset_lane.py"), "--root", str(ROOT), "--check"],
            label="elizawy-asset-lane-check", debug_on_fail=False)),
        ("M2D07 unified asset lane contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d07_asset_lane_selftest.py")], label="m2d07-asset-lane-selftest", debug_on_fail=False)),
        ("M2D07 world composition contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d07_world_contract_selftest.py")], label="m2d07-world-contract-selftest", debug_on_fail=False)),
        ("M2D080 Native Summer source authority", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d080_native_summer_authority_selftest.py")], label="m2d080-native-summer-selftest", debug_on_fail=False)),
        ("M2D081C Summer profile / static-water authority check", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_summer_complete_runtime.py"), "--check"], label="m2d081-summer-profile-check", debug_on_fail=False)),
        ("M2D081C Summer completion + water authority repair", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d081_summer_complete_selftest.py")], label="m2d081-summer-complete-selftest", debug_on_fail=False)),
        ("M2D082 Summer cliff source authority deterministic check", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_cliff_summer_authority.py"), "--check"], label="m2d082-cliff-source-check", debug_on_fail=False)),
        ("M2D082 Summer cliff source authority contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082_cliff_source_authority_selftest.py")], label="m2d082-cliff-source-selftest", debug_on_fail=False)),
        ("M2D082C complete ElizaWy worldgen source catalog", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/build_worldgen_asset_catalog.py"), "--check"], label="m2d082c-worldgen-asset-catalog", debug_on_fail=False)),
        ("M2D082C Generated World consolidation / shoreline / tree contract", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082c_generated_world_selftest.py")], label="m2d082c-generated-world-selftest", debug_on_fail=False)),
        ("M2D082D dual-grid semantic/render coordinate authority", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082d_dual_grid_alignment_selftest.py")], label="m2d082d-dual-grid-alignment-selftest", debug_on_fail=False)),
        ("M2D082F land-connectivity + extended Summer materials", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d082f_land_connectivity_materials_selftest.py")], label="m2d082f-land-connectivity-materials-selftest", debug_on_fail=False)),
        ("M2D090 ElizaWy kitchen-sink Generated World", lambda: run_stream(
            [python_cmd(), str(ROOT / "tools/m2d090_kitchen_sink_world_selftest.py")], label="m2d090-kitchen-sink-world-selftest", debug_on_fail=False)),
        ("Authored scene source-address integrity", authored_scene_check),
        ("DG contract + fixtures", lambda: run_terrain(["fixtures"], label="terrain-fixtures", debug_on_fail=False)),
        ("DG semantic resolver lab", lambda: run_terrain(["resolver"], label="terrain-resolver", debug_on_fail=False)),
        ("DG resolved semantic-to-recipe pipeline", lambda: run_terrain(["pipeline"], label="terrain-pipeline", debug_on_fail=False)),
        ("Summer mapping recovery", lambda: run_terrain(["recovery"], label="terrain-recovery", debug_on_fail=False)),
        ("Safe GitHub source refresh", lambda: run_backend(["source", "refresh"], label="source-refresh", debug_on_fail=False)),
        ("Doctor", lambda: run_backend(["doctor"], label="doctor", debug_on_fail=False)),
    ]

    for name, action in phase_specs:
        rc = phase(name, action)
        if rc:
            receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
            pcc_print("FULL GATE: FAIL")
            pcc_print("GATE RECEIPT:", receipt)
            try:
                pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason=f"full-gate:{name}"))
            except Exception:
                pcc_print("DEBUG BUNDLE CREATION FAILED")
            return rc
    rc = phase("Cargo fetch", lambda: run_backend(["fetch"], label="cargo-fetch"))
    if rc:
        receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
        pcc_print("FULL GATE: FAIL")
        pcc_print("GATE RECEIPT:", receipt)
        return rc

    rc = phase(
        "Rustfmt check",
        lambda: run_backend(["fmt"], label="cargo-fmt-check", debug_on_fail=False),
    )
    if rc:
        phases[-1]["result"] = "REPAIR_NEEDED"
        repair_receipt = phases[-1].get("phaseReceipt")
        if repair_receipt:
            write_json(ROOT / repair_receipt, {key: value for key, value in phases[-1].items() if key != "phaseReceipt"})
        pcc_print("RUSTFMT: source normalization required; applying cargo fmt and rechecking.")
        rc = phase("Rustfmt normalize", lambda: run_backend(["format"], label="cargo-fmt-write"))
        if rc == 0:
            rc = phase("Rustfmt recheck", lambda: run_backend(["fmt"], label="cargo-fmt-recheck"))
    if rc:
        receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
        pcc_print("FULL GATE: FAIL")
        pcc_print("GATE RECEIPT:", receipt)
        try:
            pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason="full-gate:rustfmt"))
        except Exception:
            pcc_print("DEBUG BUNDLE CREATION FAILED")
        return rc

    # Evidence is a distinct certified gate phase, not hidden inside py_compile.
    # It runs after full asset verification. An absent optional derivative report
    # remains explicitly SKIPPED and cannot authorize a runtime recipe.
    for name, action in [
        ("ElizaWy historical provenance/source artwork", elizawy_history_gate),
        ("ElizaWy external/complete-review evidence", elizawy_derivative_gate),
    ]:
        rc = phase(name, action)
        if rc:
            receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
            pcc_print("FULL GATE: FAIL")
            pcc_print("GATE RECEIPT:", receipt)
            return rc

    compile_phases = [
        ("Cargo check", lambda: run_backend(["check"], label="cargo-check")),
        ("Cargo tests", lambda: run_backend(["test"], label="cargo-test")),
        ("Cargo build", lambda: run_backend(["build", "--no-refresh"], label="cargo-build")),
    ]
    for name, action in compile_phases:
        rc = phase(name, action)
        if rc:
            receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
            pcc_print("FULL GATE: FAIL")
            pcc_print("GATE RECEIPT:", receipt)
            try:
                pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason=f"full-gate:{name}"))
            except Exception:
                pcc_print("DEBUG BUNDLE CREATION FAILED")
            return rc

    rc = phase("Cargo lock snapshot", cargo_lock_check)
    if rc:
        receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
        pcc_print("FULL GATE: FAIL")
        pcc_print("GATE RECEIPT:", receipt)
        return rc

    rc = phase(
        "Cargo locked dependency verification",
        lambda: run_backend(["lock-verify"], label="cargo-lock-verify"),
    )
    if rc:
        receipt = write_gate_receipt(gate_id, started_at, started, phases, "FAIL", rc)
        pcc_print("FULL GATE: FAIL")
        pcc_print("GATE RECEIPT:", receipt)
        return rc

    receipt = write_gate_receipt(gate_id, started_at, started, phases, "PASS", 0)
    pcc_print("FULL GATE: PASS")
    pcc_print("GATE RECEIPT:", receipt)
    return 0

def build_job() -> int:
    rc = updates_apply_all()
    if rc:
        return rc
    if _SELF_UPDATE_APPLIED:
        return restart_current_pcc(["build"])
    return run_backend(["build"], label="build")


def run_job(backend: str, native_frame: bool = False, verbose_gpu: bool = False) -> int:
    args = ["run", "--backend", backend]
    if native_frame:
        args.append("--native-frame")
    if verbose_gpu:
        args.append("--verbose-gpu")
    suffix = ("-native" if native_frame else "") + ("-gpu-debug" if verbose_gpu else "")
    return run_backend(args, label=f"run-{backend}{suffix}")


def doctor_job() -> int:
    dashboard()
    return run_backend(["doctor"], label="doctor")


def status_job() -> int:
    dashboard()
    run_backend(["source", "status"], label="source-status", debug_on_fail=False)
    run_backend(["assets", "status"], label="asset-status", debug_on_fail=False)
    run_terrain(["status"], label="terrain-status", debug_on_fail=False)
    return 0


def _git_inside_worktree() -> bool:
    if not shutil.which("git"):
        return False
    rc, inside = capture(["git", "rev-parse", "--is-inside-work-tree"])
    return rc == 0 and inside.lower() == "true"


def _git_ensure_identity_interactive() -> int:
    """Ensure Git has an author identity before PCC creates the local baseline."""
    name_rc, name = capture(["git", "config", "user.name"])
    email_rc, email = capture(["git", "config", "user.email"])
    if name_rc == 0 and name.strip() and email_rc == 0 and email.strip():
        pcc_print(f" Git author : {name.strip()} <{email.strip()}>")
        return 0
    pcc_print("Git needs an author identity for the first local baseline commit.")
    entered_name = input(f"Git user.name [{name.strip() if name_rc == 0 else ''}]: ").strip() or (name.strip() if name_rc == 0 else "")
    entered_email = input(f"Git user.email [{email.strip() if email_rc == 0 else ''}]: ").strip() or (email.strip() if email_rc == 0 else "")
    if not entered_name or not entered_email:
        pcc_print("GIT BOOTSTRAP BLOCKED: user.name and user.email are required for the baseline commit.")
        return 6
    rc = run_stream(["git", "config", "--local", "user.name", entered_name], label="git-bootstrap-user-name", debug_on_fail=False)
    if rc:
        return rc
    return run_stream(["git", "config", "--local", "user.email", entered_email], label="git-bootstrap-user-email", debug_on_fail=False)


def _git_remote_heads(url: str) -> tuple[int, str]:
    return capture(["git", "ls-remote", "--heads", url], timeout=60.0)


def git_configure_origin(url: str, *, branch: str | None = None, adopt_unrelated: bool = False) -> int:
    """Configure/fetch origin and optionally adopt unrelated existing history.

    Adoption uses `git reset --mixed origin/<branch>` so HEAD/index attach to the
    canonical repository while every current working-tree source file remains
    untouched. This is the safe bridge from a PCC-created local root baseline to
    an older existing GitHub repository.
    """
    if not shutil.which("git"):
        pcc_print("GIT ORIGIN: Git is unavailable.")
        return 127
    if not _git_inside_worktree():
        pcc_print("GIT ORIGIN: current folder is not a Git checkout. Run First-time Git bootstrap first.")
        return 6
    branch = branch or capture(["git", "branch", "--show-current"])[1].strip() or "main"
    probe_rc, heads = _git_remote_heads(url)
    if probe_rc:
        pcc_print("GIT ORIGIN BLOCKED: unable to read the requested remote. Check the URL/authentication first.")
        return probe_rc
    current_rc, current = capture(["git", "remote", "get-url", "origin"])
    action = ["git", "remote", "set-url", "origin", url] if current_rc == 0 else ["git", "remote", "add", "origin", url]
    rc = run_stream(action, label="git-origin-configure", debug_on_fail=False)
    if rc:
        return rc
    rc = run_stream(["git", "fetch", "--prune", "origin"], label="git-origin-fetch", debug_on_fail=False)
    if rc:
        return rc

    remote_has_branch = any(line.rstrip().endswith(f"refs/heads/{branch}") for line in heads.splitlines())
    if remote_has_branch:
        head_rc, _ = capture(["git", "rev-parse", "--verify", "HEAD"])
        related_rc = 1 if head_rc else capture(["git", "merge-base", "HEAD", f"origin/{branch}"])[0]
        if head_rc or related_rc != 0:
            if not adopt_unrelated:
                pcc_print(f"GIT ORIGIN: origin/{branch} has existing history unrelated to this local baseline.")
                pcc_print("Use the adoption path to preserve current files while attaching to canonical history.")
                return 7
            rc = run_stream(["git", "reset", "--mixed", f"origin/{branch}"], label="git-origin-adopt-history", debug_on_fail=False)
            if rc:
                return rc
            PUBLISH_STATE.unlink(missing_ok=True)
            _, status = capture(["git", "status", "--short"])
            changed_count = len([line for line in status.splitlines() if line.strip()])
            pcc_print(f"GIT ORIGIN: PASS / adopted origin/{branch}; current source preserved with {changed_count} worktree change(s).")
            return 0
        pcc_print(f"GIT ORIGIN: configured; local history is related to origin/{branch}.")
        return 0

    if heads.strip():
        available = [line.split("refs/heads/", 1)[1] for line in heads.splitlines() if "refs/heads/" in line]
        pcc_print(f"GIT ORIGIN BLOCKED: remote has history but no '{branch}' branch.")
        if available:
            pcc_print("Available remote branches:", ", ".join(available[:12]))
        return 7
    pcc_print("GIT ORIGIN: configured; remote has no branch heads yet and is ready for the certified first push.")
    return 0


def git_configure_origin_interactive(preferred_url: str | None = None) -> int:
    if not shutil.which("git"):
        pcc_print("GIT ORIGIN: Git is unavailable.")
        return 127
    if not _git_inside_worktree():
        pcc_print("GIT ORIGIN: current folder is not a Git checkout. Run First-time Git bootstrap first.")
        return 6
    current_rc, current = capture(["git", "remote", "get-url", "origin"])
    default = current.strip() if current_rc == 0 and current.strip() else (preferred_url or project_repository_url())
    url = input(f"GitHub origin URL [{default}]: ").strip() or default
    if not url:
        pcc_print("GIT ORIGIN: cancelled; no remote URL supplied.")
        return 0
    branch = capture(["git", "branch", "--show-current"])[1].strip() or "main"
    probe_rc, heads = _git_remote_heads(url)
    if probe_rc:
        pcc_print("GIT ORIGIN BLOCKED: unable to read the requested remote. Check the URL/authentication first.")
        return probe_rc
    remote_has_branch = any(line.rstrip().endswith(f"refs/heads/{branch}") for line in heads.splitlines())
    adopt = False
    if remote_has_branch:
        # Probe relationship only after a temporary fetch would be possible; when no
        # origin exists, an existing remote is expected for the canonical Havenwild repo.
        current_head_rc, _ = capture(["git", "rev-parse", "--verify", "HEAD"])
        if current_rc != 0 or current_head_rc != 0:
            pcc_print(f"Existing origin/{branch} history detected; PCC can adopt it without overwriting current files.")
            answer = input("Adopt existing GitHub history and preserve the current source tree? [Y/n]: ").strip().lower()
            adopt = answer in {"", "y", "yes"}
    rc = git_configure_origin(url, branch=branch, adopt_unrelated=adopt)
    if rc == 7 and remote_has_branch and not adopt:
        return rc
    if rc == 7 and remote_has_branch:
        # Origin may already have existed but be unrelated; offer the same safe adoption.
        answer = input("Local and GitHub histories are unrelated. Adopt origin history while preserving current files? [Y/n]: ").strip().lower()
        if answer in {"", "y", "yes"}:
            return git_configure_origin(url, branch=branch, adopt_unrelated=True)
    return rc


def git_bootstrap_interactive(preferred_url: str | None = None) -> int:
    """Create Git authority for an extracted source-only deployment.

    Empty remotes receive a local source baseline. Existing remotes are adopted
    without overwriting the extracted files: PCC fetches the selected branch,
    points the new local branch at origin/<branch> with `reset --mixed`, and
    leaves the working tree unchanged so current source becomes reviewable
    local differences for Full Gate -> certified publish.
    """
    pcc_print("\nFIRST-TIME GIT / GITHUB BOOTSTRAP")
    if not shutil.which("git"):
        pcc_print("GIT BOOTSTRAP BLOCKED: Git is unavailable.")
        return 127
    if _git_inside_worktree():
        pcc_print("GIT BOOTSTRAP: this folder is already a Git checkout.")
        return git_configure_origin_interactive(preferred_url=preferred_url or project_repository_url())

    branch = input("Working branch [main]: ").strip() or "main"
    if not re.fullmatch(r"[A-Za-z0-9._/-]+", branch) or branch.startswith("-") or ".." in branch:
        pcc_print("GIT BOOTSTRAP BLOCKED: invalid branch name.")
        return 6
    default_url = preferred_url or project_repository_url()
    url = input(f"GitHub origin URL [{default_url}]: ").strip() or default_url
    remote_heads = ""
    remote_has_branch = False
    if url:
        probe_rc, remote_heads = _git_remote_heads(url)
        if probe_rc:
            pcc_print("GIT BOOTSTRAP BLOCKED: remote URL/authentication could not be verified; no repository was initialized.")
            return probe_rc
        remote_has_branch = any(
            line.rstrip().endswith(f"refs/heads/{branch}") for line in remote_heads.splitlines()
        )
        if remote_heads.strip() and not remote_has_branch:
            available = [line.split("refs/heads/", 1)[1] for line in remote_heads.splitlines() if "refs/heads/" in line]
            pcc_print(f"GIT BOOTSTRAP BLOCKED: origin has history but no '{branch}' branch.")
            if available:
                pcc_print("Available remote branches:", ", ".join(available[:12]))
            pcc_print("Run bootstrap again and select the existing branch you want PCC to adopt.")
            return 7

    if remote_heads.strip():
        pcc_print(f"Existing origin/{branch} history detected.")
        pcc_print("PCC will fetch that history and attach this extracted folder WITHOUT replacing its files.")
        pcc_print("Your extracted source remains in the working tree and will appear as local changes against origin.")
    else:
        pcc_print("PCC will initialize Git here and create one LOCAL source baseline commit.")
        pcc_print("Nothing is pushed until a successful Full Gate followed by primary workflow option 2.")
    if input("Initialize this deployment as the Git source root? [y/N]: ").strip().lower() not in {"y", "yes"}:
        pcc_print("GIT BOOTSTRAP: cancelled.")
        return 0

    rc = run_stream(["git", "init", "-b", branch], label="git-bootstrap-init", debug_on_fail=False)
    if rc:
        return rc
    rc = _git_ensure_identity_interactive()
    if rc:
        return rc

    if url:
        rc = run_stream(["git", "remote", "add", "origin", url], label="git-bootstrap-origin", debug_on_fail=False)
        if rc:
            return rc

    if remote_heads.strip():
        rc = run_stream(["git", "fetch", "--prune", "origin"], label="git-bootstrap-fetch", debug_on_fail=False)
        if rc:
            return rc
        ref_rc, _ = capture(["git", "show-ref", "--verify", "--quiet", f"refs/remotes/origin/{branch}"])
        if ref_rc:
            pcc_print(f"GIT BOOTSTRAP BLOCKED: origin/{branch} was not available after fetch.")
            return 7
        # Mixed reset changes HEAD/index only; it deliberately does not touch the
        # extracted working files. They become normal source modifications.
        rc = run_stream(["git", "reset", "--mixed", f"origin/{branch}"], label="git-bootstrap-adopt-history", debug_on_fail=False)
        if rc:
            return rc
        _, status = capture(["git", "status", "--short"])
        changed_count = len([line for line in status.splitlines() if line.strip()])
        pcc_print(f"GIT BOOTSTRAP: PASS / adopted origin/{branch}; extracted source preserved with {changed_count} worktree change(s).")
        pcc_print("NEXT: run PRIMARY WORKFLOW 1 (Full Gate), then 2 (Commit + Push certified source).")
        return 0

    untracked_rc, raw = git_bytes(["ls-files", "--others", "--exclude-standard", "-z"])
    if untracked_rc:
        return untracked_rc
    candidates = [part.decode("utf-8", errors="surrogateescape").replace("\\", "/") for part in raw.split(b"\0") if part]
    safe: list[str] = []
    skipped: list[str] = []
    for rel in candidates:
        if git_publish_path_error(rel):
            skipped.append(rel)
        else:
            safe.append(rel)
    if not safe:
        pcc_print("GIT BOOTSTRAP BLOCKED: no publishable source files were found.")
        return 6
    for start_index in range(0, len(safe), 48):
        rc = run_stream(["git", "add", "-A", "--", *safe[start_index:start_index + 48]], label="git-bootstrap-stage", debug_on_fail=False)
        if rc:
            return rc
    staged_rc, staged_bytes = git_bytes(["diff", "--cached", "--name-only", "-z"])
    if staged_rc:
        return staged_rc
    staged = [part.decode("utf-8", errors="surrogateescape").replace("\\", "/") for part in staged_bytes.split(b"\0") if part]
    for rel in staged:
        error = git_publish_path_error(rel)
        if error:
            pcc_print("GIT BOOTSTRAP BLOCKED:", error)
            return 8
    message = f"Havenwild source baseline {dt.datetime.now().astimezone():%Y-%m-%d}"
    rc = run_stream(["git", "commit", "-m", message], label="git-bootstrap-baseline", debug_on_fail=False)
    if rc:
        return rc
    pcc_print(f"GIT BOOTSTRAP: PASS / {len(staged)} source path(s) committed locally on {branch}.")
    if skipped:
        pcc_print(f"Local/runtime paths excluded from baseline: {len(skipped)}")
    if url:
        pcc_print("NEXT: run PRIMARY WORKFLOW 1 (Full Gate), then 2 (Commit + Push certified source).")
    else:
        pcc_print("NEXT: configure origin from Source control / GitHub, then run Full Gate and Commit + Push.")
    return 0

def asset_sync_interactive() -> int:
    pcc_print("\nHYDRATE / REPAIR CORE ASSETS — GITHUB FIRST")
    pcc_print("Primary authority: exact pinned ElizaWy/LPC GitHub revision. No local source path is normally required.")
    rc = run_backend(["assets", "sync"], label="asset-sync", debug_on_fail=False)
    if rc == 0:
        return 0
    pcc_print("Pinned GitHub hydration and remembered/local fallback did not complete.")
    source = input("Optional local fallback source path (blank = cancel): ").strip()
    if not source:
        return rc
    return run_backend(["assets", "sync", "--source", source, "--local-only"], label="asset-sync-local-fallback")


def source_menu() -> int:
    while True:
        pcc_print("\nSOURCE CONTROL / GITHUB")
        ready, reason, state = git_publish_readiness()
        pcc_print(f" Publish    : {'READY' if ready else 'BLOCKED'} — {reason}")
        pcc_print(f" Target     : origin/{state.get('branch') or '-'}")
        pcc_print(" 1. Status / worktree / origin")
        pcc_print(" 2. Commit + push certified source")
        pcc_print(" 3. Refresh clean source from origin (fast-forward only)")
        pcc_print(" 4. First-time Git bootstrap / adopt existing GitHub history")
        pcc_print(" 5. Configure / change origin remote")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        if choice == "1": run_backend(["source", "status"], label="source-status", debug_on_fail=False)
        elif choice == "2": source_publish_interactive()
        elif choice == "3": run_backend(["source", "refresh"], label="source-refresh")
        elif choice == "4": git_bootstrap_interactive(preferred_url=project_repository_url())
        elif choice == "5": git_configure_origin_interactive(preferred_url=project_repository_url())
        input("Press Enter...")


def assets_menu() -> int:
    while True:
        pcc_print("\nASSET OPERATIONS — LOCAL ONLY / NEVER PACKAGED")
        pcc_print(" 1. Asset status")
        pcc_print(" 2. Hydrate / repair core assets (pinned GitHub first)")
        pcc_print(" 3. Local fallback sync (manual source path)")
        pcc_print(" 4. Hydrate characters")
        pcc_print(" 5. Verify hydrated characters")
        pcc_print(" 6. Rebuild local catalogs")
        pcc_print(" 7. Full core manifest hash verification")
        pcc_print(" 8. Force verify / repair from pinned GitHub upstream")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        if choice == "1": run_backend(["assets", "status"], label="asset-status", debug_on_fail=False)
        elif choice == "2": asset_sync_interactive()
        elif choice == "3":
            source = input("Local fallback source path (blank = cancel): ").strip()
            if source:
                run_backend(["assets", "sync", "--source", source, "--local-only"], label="asset-sync-local-fallback")
        elif choice == "4": run_backend(["assets", "hydrate-characters"], label="characters-hydrate")
        elif choice == "5": run_backend(["assets", "hydrate-characters", "--verify"], label="characters-verify")
        elif choice == "6": run_backend(["catalog"], label="asset-catalog")
        elif choice == "7": run_backend(["assets", "status", "--full"], label="asset-verify-full")
        elif choice == "8": run_backend(["assets", "refresh-upstream"], label="asset-refresh-upstream")
        input("Press Enter...")


def terrain_menu() -> int:
    while True:
        pcc_print("\nTERRAIN / DUAL-GRID / MAPPING AUTHORITY")
        pcc_print(" 1. Contract + recovery + resolver status")
        pcc_print(" 2. Fixture topology coverage")
        pcc_print(" 3. Semantic resolver lab (16-mask path)")
        pcc_print(" 4. Resolved semantic -> recipe pipeline")
        pcc_print(" 5. Strict DG recipe certification")
        pcc_print(" 6. Summer mapping recovery evidence")
        pcc_print(" 7. Find/recover older Havenwild mapping metadata")
        pcc_print(" 8. Local legacy-recovery index status")
        pcc_print(" 9. ONE-PASS Native -> Bevy Summer authority convergence")
        pcc_print("10. Native Summer convergence status")
        pcc_print("11. Legacy pixel-classifier diagnostic (not runtime authority)")
        pcc_print("12. Summer cliff source authority / evidence status")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        if choice == "1": run_terrain(["status"], label="terrain-status", debug_on_fail=False)
        elif choice == "2": run_terrain(["fixtures"], label="terrain-fixtures", debug_on_fail=False)
        elif choice == "3": run_terrain(["resolver"], label="terrain-resolver", debug_on_fail=False)
        elif choice == "4": run_terrain(["pipeline"], label="terrain-pipeline", debug_on_fail=False)
        elif choice == "5": run_terrain(["certify"], label="terrain-certify")
        elif choice == "6": run_terrain(["recovery"], label="terrain-recovery", debug_on_fail=False)
        elif choice == "7":
            source = input("Older Havenwild root (blank = auto-discover siblings): ").strip()
            args = ["recover"]
            if source:
                args += ["--source", source]
            run_legacy_mapping(args, label="terrain-legacy-recover", debug_on_fail=False)
        elif choice == "8":
            run_legacy_mapping(["status"], label="terrain-legacy-status", debug_on_fail=False)
        elif choice == "9":
            run_native_summer_convergence(["converge"], label="terrain-native-summer-converge", debug_on_fail=False)
        elif choice == "10":
            run_native_summer_convergence(["status"], label="terrain-native-summer-status", debug_on_fail=False)
        elif choice == "11":
            run_summer_flatworld_mapper(["build", "--dry-run"], label="terrain-summer-classifier-diagnostic", debug_on_fail=False)
        elif choice == "12":
            run_cliff_summer_authority(["--check"], label="terrain-cliff-source-authority", debug_on_fail=False)
        input("Press Enter...")


def updates_menu() -> int:
    while True:
        pcc_print("\nPATCH / UPDATE INBOX")
        pcc_print(" 1. Scan/status")
        pcc_print(" 2. Apply all governed patches")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        if choice == "1": updates_status()
        elif choice == "2":
            rc = updates_apply_all()
            if rc == 0 and _SELF_UPDATE_APPLIED:
                return restart_current_pcc(["menu"])
        input("Press Enter...")


def diagnostics_menu() -> int:
    while True:
        pcc_print("\nDIAGNOSTICS")
        pcc_print(" 1. Create debug bundle now")
        pcc_print(" 2. Show last job receipt")
        pcc_print(" 3. Tail last job log")
        pcc_print(" 4. Show recent job receipts")
        pcc_print(" 5. Show log locations")
        pcc_print(" 6. Environment snapshot")
        pcc_print(" 7. Prune retained PCC history")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        if choice == "1":
            pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason="manual"))
        elif choice == "2":
            pcc_print(json.dumps(read_json(LAST_JOB, {}) or {}, indent=2))
        elif choice == "3":
            last = read_json(LAST_JOB, {}) or {}
            rel = last.get("jobLog")
            path = ROOT / rel if rel else None
            if path and path.is_file():
                lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
                pcc_print("\n".join(lines[-80:]))
            else:
                pcc_print("No last job log is available.")
        elif choice == "4":
            recent = sorted(RECEIPT_ROOT.glob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)[:12]
            for path in recent:
                receipt = read_json(path, {}) or {}
                pcc_print(f"{path.name}: {receipt.get('label', receipt.get('result', '?'))} [{receipt.get('result', '?')}]")
        elif choice == "5":
            pcc_print("PCC LOGS:", LOG_ROOT)
            pcc_print("RECEIPTS:", RECEIPT_ROOT)
            pcc_print("GATES:", GATE_ROOT)
            pcc_print("DEBUG:", DEBUG_ROOT)
            pcc_print("FORGEPY LOGS:", ROOT / ".forgepy" / "logs")
        elif choice == "6":
            pcc_print(environment_snapshot())
            pcc_print(git_snapshot())
        elif choice == "7":
            prune_state()
            pcc_print("PCC retention cleanup complete.")
        input("Press Enter...")


def patch_scan_apply_workflow() -> int:
    pcc_print("\nPATCH SCAN / REVIEW / APPLY")
    updates_status()
    pending = discover_updates()
    if not pending:
        return 0
    answer = input("Apply the exact-version-compatible governed patch now? [y/N]: ").strip().lower()
    if answer not in {"y", "yes"}:
        pcc_print("PATCH APPLY: cancelled; inbox left unchanged.")
        return 0
    rc = updates_apply_all()
    if rc == 0 and _SELF_UPDATE_APPLIED:
        return restart_current_pcc(["menu"])
    if rc == 0:
        updates_status()
    return rc


def studio_menu() -> int:
    while True:
        pcc_print("\nSTUDIO / BUILD / RUN")
        pcc_print(" 1. Run Studio (DX12)")
        pcc_print(" 2. Run Studio (Vulkan)")
        pcc_print(" 3. Build Studio")
        pcc_print(" 4. Run Studio (DX12 native frame)")
        pcc_print(" 5. Run Studio (DX12 verbose GPU diagnostics)")
        pcc_print(" 6. Run Studio (Vulkan verbose GPU diagnostics)")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        action = {
            "1": lambda: run_job("dx12"),
            "2": lambda: run_job("vulkan"),
            "3": build_job,
            "4": lambda: run_job("dx12", True),
            "5": lambda: run_job("dx12", verbose_gpu=True),
            "6": lambda: run_job("vulkan", verbose_gpu=True),
        }.get(choice)
        if action:
            action()
            input("Press Enter...")


def health_menu() -> int:
    while True:
        pcc_print("\nPROJECT HEALTH")
        pcc_print(" 1. Full status dashboard")
        pcc_print(" 2. Doctor")
        pcc_print(" 3. Static project audit (no Cargo)")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        if choice == "1": status_job()
        elif choice == "2": doctor_job()
        elif choice == "3": audit_gate()
        else: continue
        input("Press Enter...")


def maintenance_menu() -> int:
    while True:
        pcc_print("\nDEVELOPER / MAINTENANCE")
        pcc_print(" 1. Cargo tests")
        pcc_print(" 2. Package source-only ZIP")
        pcc_print(" 3. Cargo clean")
        pcc_print(" 4. Patch / update inbox advanced menu")
        pcc_print(" 0. Back")
        choice = input("> ").strip()
        if choice == "0": return 0
        if choice == "1": run_backend(["test"], label="cargo-test")
        elif choice == "2": run_backend(["package"], label="package-source")
        elif choice == "3": run_backend(["clean"], label="cargo-clean")
        elif choice == "4": updates_menu()
        else: continue
        input("Press Enter...")


def menu() -> int:
    while True:
        try:
            dashboard()
            ready, reason, _ = git_publish_readiness()
            pcc_print(" PRIMARY WORKFLOW")
            pcc_print("  1. FULL QUALITY GATE / CERTIFY CURRENT SOURCE")
            pcc_print(f"  2. COMMIT + PUSH CERTIFIED SOURCE TO GITHUB [{'READY' if ready else 'BLOCKED'}]")
            if not ready:
                pcc_print(f"     {reason}")
            pcc_print("  3. PATCH SCAN / REVIEW / APPLY")
            pcc_print("\n OPERATE")
            pcc_print("  4. Studio / build / run")
            pcc_print("  5. Project health / doctor / static audit")
            pcc_print("\n AUTHORING / DATA")
            pcc_print("  6. Assets / hydration")
            pcc_print("  7. Terrain / dual-grid / mapping authority")
            pcc_print("\n SOURCE / SUPPORT")
            pcc_print("  8. Source control / GitHub")
            pcc_print("  9. Diagnostics / debug bundle")
            pcc_print(" 10. Developer / maintenance")
            pcc_print("  0. Exit")
            choice = input("> ").strip()
            if choice == "0":
                return 0
            action = {
                "1": full_gate,
                "2": source_publish_interactive,
                "3": patch_scan_apply_workflow,
                "4": studio_menu,
                "5": health_menu,
                "6": assets_menu,
                "7": terrain_menu,
                "8": source_menu,
                "9": diagnostics_menu,
                "10": maintenance_menu,
            }.get(choice)
            if action is None:
                continue
            try:
                rc = action()
            except KeyboardInterrupt:
                pcc_print("\nJOB INTERRUPTED BY USER; PCC remains active.")
                rc = 130
            except Exception:
                pcc_print("\nPCC JOB EXCEPTION; PCC remains active.")
                pcc_print(traceback.format_exc())
                try:
                    pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason="pcc-job-exception"))
                except Exception:
                    pcc_print("Debug bundle also failed:")
                    pcc_print(traceback.format_exc())
                rc = 251
            pcc_print(f"RESULT: {'PASS' if rc == 0 else 'FAIL'} ({rc})")
            input("Press Enter to return to PCC...")
        except KeyboardInterrupt:
            pcc_print("\nPCC menu interrupt; choose 0 to exit.")
        except EOFError:
            return 0
        except Exception:
            pcc_print("\nPCC MENU EXCEPTION — host remains active")
            pcc_print(traceback.format_exc())
            try:
                pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason="pcc-menu-exception"))
            except Exception:
                pass
            try:
                input("Press Enter to recover the PCC menu...")
            except EOFError:
                return 252


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(prog="PCC")
    sub = p.add_subparsers(dest="command")
    sub.add_parser("menu")
    sub.add_parser("full")
    sub.add_parser("audit")
    sub.add_parser("rust-audit")
    sub.add_parser("build")
    sub.add_parser("test")
    sub.add_parser("doctor")
    sub.add_parser("status")
    sub.add_parser("package")
    sub.add_parser("clean")
    run_p = sub.add_parser("run")
    run_p.add_argument("backend", nargs="?", choices=["dx12", "vulkan"], default="dx12")
    run_p.add_argument("--native-frame", action="store_true")
    run_p.add_argument("--verbose-gpu", action="store_true")
    source_p = sub.add_parser("source")
    source_p.add_argument("action", choices=["status", "refresh", "publish", "readiness"], nargs="?", default="status")
    source_p.add_argument("--message")
    assets_p = sub.add_parser("assets")
    assets_p.add_argument("action", choices=["status", "verify-core", "sync", "refresh-upstream", "hydrate-characters", "verify-characters", "catalog"], nargs="?", default="status")
    assets_p.add_argument("--source")
    terrain_p = sub.add_parser("terrain")
    terrain_p.add_argument("action", choices=["status", "fixtures", "resolver", "pipeline", "certify", "recovery", "recover", "recovery-index", "summer-map", "summer-map-status", "native-summer", "native-summer-status"], nargs="?", default="status")
    terrain_p.add_argument("--source")
    updates_p = sub.add_parser("updates")
    updates_p.add_argument("action", choices=["status", "apply"], nargs="?", default="status")
    diag_p = sub.add_parser("diagnostics")
    diag_p.add_argument("action", choices=["bundle", "last"], nargs="?", default="bundle")
    return p.parse_args(argv)


def dispatch(args: argparse.Namespace) -> int:
    cmd = args.command or "menu"
    if cmd == "menu": return menu()
    if cmd == "full": return full_gate()
    if cmd == "audit": return audit_gate()
    if cmd == "rust-audit": return rust_source_audit()
    if cmd == "build": return build_job()
    if cmd == "test": return run_backend(["test"], label="cargo-test")
    if cmd == "doctor": return doctor_job()
    if cmd == "status": return status_job()
    if cmd == "package": return run_backend(["package"], label="package-source")
    if cmd == "clean": return run_backend(["clean"], label="cargo-clean")
    if cmd == "run": return run_job(args.backend, args.native_frame, args.verbose_gpu)
    if cmd == "source":
        if args.action == "publish":
            return source_publish(args.message)
        if args.action == "readiness":
            ready, reason, state = git_publish_readiness()
            pcc_print(json.dumps({"ready": ready, "reason": reason, "state": state}, indent=2))
            return 0 if ready else 6
        return run_backend(["source", args.action], label=f"source-{args.action}")
    if cmd == "assets":
        mapping = {
            "status": (["assets", "status"], "asset-status"),
            "verify-core": (["assets", "status", "--full"], "asset-verify-full"),
            "sync": (["assets", "sync"], "asset-sync"),
            "refresh-upstream": (["assets", "refresh-upstream"], "asset-refresh-upstream"),
            "hydrate-characters": (["assets", "hydrate-characters"], "characters-hydrate"),
            "verify-characters": (["assets", "hydrate-characters", "--verify"], "characters-verify"),
            "catalog": (["catalog"], "asset-catalog"),
        }
        command, label = mapping[args.action]
        if args.action == "sync" and args.source:
            command += ["--source", args.source]
        return run_backend(command, label=label)
    if cmd == "terrain":
        if args.action == "recover":
            command = ["recover"]
            if args.source:
                command += ["--source", args.source]
            return run_legacy_mapping(command, label="terrain-legacy-recover", debug_on_fail=False)
        if args.action == "recovery-index":
            return run_legacy_mapping(["status"], label="terrain-legacy-status", debug_on_fail=False)
        if args.action == "native-summer":
            command = ["converge"]
            if args.source:
                command += ["--source", args.source]
            return run_native_summer_convergence(command, label="terrain-native-summer-converge", debug_on_fail=False)
        if args.action == "native-summer-status":
            return run_native_summer_convergence(["status"], label="terrain-native-summer-status", debug_on_fail=False)
        if args.action == "summer-map":
            return run_summer_flatworld_mapper(["build", "--dry-run"], label="terrain-summer-classifier-diagnostic", debug_on_fail=False)
        if args.action == "summer-map-status":
            return run_summer_flatworld_mapper(["status"], label="terrain-summer-flatworld-status", debug_on_fail=False)
        return run_terrain([args.action], label=f"terrain-{args.action}")
    if cmd == "updates": return updates_status() if args.action == "status" else updates_apply_all()
    if cmd == "diagnostics":
        if args.action == "last":
            pcc_print(json.dumps(read_json(LAST_JOB, {}) or {}, indent=2)); return 0
        pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason="manual")); return 0
    return 2


def main(argv: list[str]) -> int:
    open_session()
    try:
        args = parse_args(argv)
        return dispatch(args)
    except KeyboardInterrupt:
        pcc_print("\nPCC interrupted by user.")
        return 130
    except Exception:
        pcc_print("\nFATAL PCC EXCEPTION")
        pcc_print(traceback.format_exc())
        try:
            pcc_print("DEBUG BUNDLE:", make_debug_bundle(reason="fatal-pcc-exception"))
        except Exception:
            pass
        return 252
    finally:
        close_session()


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
