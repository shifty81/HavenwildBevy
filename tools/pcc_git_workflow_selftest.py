#!/usr/bin/env python3
"""Regression test for PCC Full Gate -> commit/push source-control authority."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile

PROJECT_ROOT = Path(__file__).resolve().parents[1]
PCC_PATH = PROJECT_ROOT / "ProjectControlCenter.py"


def run(*args: str, cwd: Path, check: bool = True) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        list(args), cwd=str(cwd), text=True, encoding="utf-8", errors="replace",
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False,
    )
    if check and proc.returncode:
        raise RuntimeError(f"command failed ({proc.returncode}): {' '.join(args)}\n{proc.stdout}")
    return proc


def load_pcc():
    spec = importlib.util.spec_from_file_location("havenwild_pcc_git_selftest", PCC_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load ProjectControlCenter.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def point_pcc_at(module, root: Path) -> None:
    module.ROOT = root
    module.STATE_ROOT = root / ".pcc"
    module.LOG_ROOT = module.STATE_ROOT / "logs"
    module.RECEIPT_ROOT = module.STATE_ROOT / "receipts"
    module.DEBUG_ROOT = module.STATE_ROOT / "debug"
    module.GATE_ROOT = module.STATE_ROOT / "gates"
    module.ROLLBACK_ROOT = module.STATE_ROOT / "rollback"
    module.STAGING_ROOT = module.STATE_ROOT / "staging"
    module.LAST_JOB = module.STATE_ROOT / "last_job.json"
    module.LAST_GATE = module.STATE_ROOT / "last_gate.json"
    module.ACTIVE_JOB = module.STATE_ROOT / "active_job.json"
    module.PUBLISH_STATE = module.STATE_ROOT / "git_publish_state.json"
    module.UPDATE_ROOT = root / "updates"
    module.UPDATE_INBOX = module.UPDATE_ROOT / "inbox"
    module.UPDATE_ARCHIVE = module.UPDATE_ROOT / "archive"
    module.UPDATE_REJECTED = module.UPDATE_ROOT / "rejected"
    module._SESSION_PATH = None
    module._SESSION_HANDLE = None


def certify(module, gate_id: str) -> dict:
    state = module.git_worktree_state()
    module.ensure_dirs()
    module.write_json(module.LAST_GATE, {
        "schema": "havenwild.bevy.pcc.gate_receipt.v1",
        "pccVersion": module.PCC_VERSION,
        "gateId": gate_id,
        "result": "PASS",
        "sourceControl": state,
        "phases": [],
    })
    return state


def main() -> int:
    if not shutil.which("git"):
        print("PCC GIT WORKFLOW SELFTEST: SKIP / git not installed")
        return 0

    module = load_pcc()
    with tempfile.TemporaryDirectory(prefix="havenwild-pcc-git-") as td:
        base = Path(td)
        work = base / "work"
        remote = base / "remote.git"
        clone = base / "other"
        work.mkdir()
        run("git", "init", "--bare", str(remote), cwd=base)
        run("git", "init", "-b", "main", cwd=work)
        run("git", "config", "user.email", "pcc-selftest@example.invalid", cwd=work)
        run("git", "config", "user.name", "PCC Selftest", cwd=work)
        (work / ".gitignore").write_text(".pcc/\n", encoding="utf-8")
        (work / "hello.txt").write_text("baseline\n", encoding="utf-8")
        run("git", "add", ".", cwd=work)
        run("git", "commit", "-m", "baseline", cwd=work)
        run("git", "remote", "add", "origin", str(remote), cwd=work)
        run("git", "push", "-u", "origin", "main", cwd=work)

        point_pcc_at(module, work)

        # Windows Git may emit core.autocrlf warnings on stderr while a machine-
        # readable NUL-delimited filename list is written to stdout.  Those
        # warnings must never become fake changed paths.
        run("git", "config", "core.autocrlf", "true", cwd=work)
        (work / "Cargo.lock").write_text("baseline\n", encoding="utf-8")
        run("git", "add", "Cargo.lock", cwd=work)
        run("git", "commit", "-m", "line ending fixture", cwd=work)
        run("git", "push", "origin", "main", cwd=work)
        (work / "Cargo.lock").write_text("baseline\ncertified lock change\n", encoding="utf-8")
        warning_state = module.git_worktree_state()
        assert warning_state["changedPaths"] == ["Cargo.lock"], warning_state["changedPaths"]
        run("git", "checkout", "--", "Cargo.lock", cwd=work)

        (work / "hello.txt").write_text("baseline\ncertified\n", encoding="utf-8")
        certified = certify(module, "SELFTEST-GATE-1")
        assert certified["changedPaths"] == ["hello.txt"]
        ready, reason, _ = module.git_publish_readiness()
        assert ready, reason
        rc = module.source_publish("PCC selftest certified publish")
        assert rc == 0, f"publish returned {rc}"
        assert not run("git", "status", "--porcelain", cwd=work).stdout.strip()
        assert "PCC selftest certified publish" in run("git", "--git-dir", str(remote), "log", "--oneline", "--all", "-3", cwd=base).stdout

        ready, reason, _ = module.git_publish_readiness()
        assert not ready and "already published" in reason

        # Retry safety: a prior failed publish may already have staged a deletion
        # before stopping on a later path. Re-publishing must not pass that already-
        # staged deleted pathname back to `git add` as a now-invalid pathspec.
        (work / "obsolete.txt").write_text("remove me\n", encoding="utf-8")
        run("git", "add", "obsolete.txt", cwd=work)
        run("git", "commit", "-m", "deletion retry fixture", cwd=work)
        run("git", "push", "origin", "main", cwd=work)
        (work / "hello.txt").write_text("baseline\ncertified\nretry-safe edit\n", encoding="utf-8")
        (work / "obsolete.txt").unlink()
        certified_retry = certify(module, "SELFTEST-GATE-RETRY-DELETE")
        assert set(certified_retry["changedPaths"]) == {"hello.txt", "obsolete.txt"}
        run("git", "add", "-u", "--", "obsolete.txt", cwd=work)
        assert "obsolete.txt" in run("git", "diff", "--cached", "--name-only", cwd=work).stdout
        rc = module.source_publish("PCC selftest retry-safe staged deletion")
        assert rc == 0, f"retry-safe staged-deletion publish returned {rc}"
        assert not run("git", "status", "--porcelain", cwd=work).stdout.strip()
        assert "retry-safe staged deletion" in run("git", "--git-dir", str(remote), "log", "--oneline", "--all", "-3", cwd=base).stdout

        # A post-gate edit must invalidate certification.
        (work / "hello.txt").write_text("baseline\ncertified\nlate edit\n", encoding="utf-8")
        ready, reason, _ = module.git_publish_readiness()
        assert not ready and "source changed" in reason
        run("git", "checkout", "--", "hello.txt", cwd=work)

        # Even a freshly certified gate may not publish root handoff archives.
        (work / "unsafe.patch").write_text("handoff", encoding="utf-8")
        certify(module, "SELFTEST-GATE-2")
        ready, reason, _ = module.git_publish_readiness()
        assert not ready and "handoff/archive" in reason
        (work / "unsafe.patch").unlink()

        # A clean certified branch that becomes behind origin must fail closed.
        certify(module, "SELFTEST-GATE-3")
        run("git", "clone", "-b", "main", str(remote), str(clone), cwd=base)
        run("git", "config", "user.email", "other@example.invalid", cwd=clone)
        run("git", "config", "user.name", "Other Writer", cwd=clone)
        (clone / "remote.txt").write_text("remote change\n", encoding="utf-8")
        run("git", "add", "remote.txt", cwd=clone)
        run("git", "commit", "-m", "remote advance", cwd=clone)
        run("git", "push", "origin", "main", cwd=clone)
        before = run("git", "rev-parse", "HEAD", cwd=work).stdout.strip()
        rc = module.source_publish(None)
        after = run("git", "rev-parse", "HEAD", cwd=work).stdout.strip()
        assert rc == 7, f"behind-origin publish should return 7, got {rc}"
        assert before == after, "PCC must not merge/rebase/reset when origin is ahead"

        # A PCC-created local root baseline may safely adopt an older unrelated
        # canonical remote after Full Gate without invalidating source-byte
        # certification. `reset --mixed` must preserve the working files.
        history_remote = base / "history.git"
        seed = base / "history-seed"
        adopt = base / "adopt"
        run("git", "init", "--bare", str(history_remote), cwd=base)
        seed.mkdir()
        run("git", "init", "-b", "main", cwd=seed)
        run("git", "config", "user.email", "history@example.invalid", cwd=seed)
        run("git", "config", "user.name", "History Seed", cwd=seed)
        (seed / "remote_only.txt").write_text("older canonical history\n", encoding="utf-8")
        run("git", "add", ".", cwd=seed)
        run("git", "commit", "-m", "older remote baseline", cwd=seed)
        run("git", "remote", "add", "origin", str(history_remote), cwd=seed)
        run("git", "push", "-u", "origin", "main", cwd=seed)

        adopt.mkdir()
        run("git", "init", "-b", "main", cwd=adopt)
        run("git", "config", "user.email", "adopt@example.invalid", cwd=adopt)
        run("git", "config", "user.name", "Adopt Local", cwd=adopt)
        (adopt / ".gitignore").write_text(".pcc/\n", encoding="utf-8")
        (adopt / "current.txt").write_text("certified current source\n", encoding="utf-8")
        run("git", "add", ".", cwd=adopt)
        run("git", "commit", "-m", "Havenwild source baseline 2026-10-01", cwd=adopt)
        point_pcc_at(module, adopt)
        certified_adopt = certify(module, "SELFTEST-GATE-ADOPT")
        before_source = certified_adopt["sourceFingerprint"]
        before_bytes = (adopt / "current.txt").read_bytes()
        rc = module.git_configure_origin(str(history_remote), branch="main", adopt_unrelated=True)
        assert rc == 0, f"unrelated-history adoption returned {rc}"
        assert (adopt / "current.txt").read_bytes() == before_bytes, "mixed adoption overwrote current source bytes"
        after_adopt = module.git_worktree_state()
        assert after_adopt["sourceFingerprint"] == before_source, "Git history adoption invalidated unchanged source fingerprint"
        assert run("git", "rev-parse", "HEAD", cwd=adopt).stdout.strip() == run("git", "rev-parse", "origin/main", cwd=adopt).stdout.strip()
        ready, reason, _ = module.git_publish_readiness()
        assert ready, f"certification should survive history adoption: {reason}"
        rc = module.source_publish("PCC selftest canonical-history adoption")
        assert rc == 0, f"adopted publish returned {rc}"
        assert "canonical-history adoption" in run("git", "--git-dir", str(history_remote), "log", "--oneline", "--all", "-3", cwd=base).stdout

    print("PCC GIT WORKFLOW SELFTEST: PASS / certified publish, source-byte certification, machine-readable stderr isolation, retry-safe partially staged deletion, canonical-history mixed adoption, stale-edit guard, handoff exclusion, behind-origin fail-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
