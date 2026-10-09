#!/usr/bin/env python3
"""M2D109: fast Git discovery, visible publish preflight, and intact policy."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("havenwild_pcc_m2d109", ROOT / "ProjectControlCenter.py")
assert SPEC and SPEC.loader
PCC = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PCC)

# Check the actual .gitignore policy, not just the existence of a comment.
if shutil.which("git"):
    with tempfile.TemporaryDirectory(prefix="havenwild-publish-fast-") as temp:
        root = Path(temp)
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        shutil.copy2(ROOT / ".gitignore", root / ".gitignore")
        candidates = {
            ".pcc/receipts/receipt.json", ".forgepy/logs/run.log",
            "target/debug/havenwild_bevy.exe", "assets/elizawy/Characters/Bodies/12345.png",
            "assets/optional/Characters.zip", "assets/elizawy/Terrain/terrain_summer.png",
            "artifacts/report.json", "updates/inbox/a.pccpatch.zip",
            "updates/archive/a.pccpatch.zip", "updates/rejected/a.pccpatch.zip",
            "ROOT_PATCH.pccpatch.zip", "content/catalog/source_index.local.json",
            "content/catalog/characters_index.local.json",
        }
        for rel in candidates | {"assets/README.md", "src/main.rs", "README.md"}:
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"fixture")
        # A tiny stand-in for a very large character hydration/build tree.
        for i in range(400):
            file = root / "assets/elizawy/Characters/Bodies" / f"{i:05d}.png"
            file.write_bytes(b"ignore")
        for i in range(400):
            file = root / "target/debug" / f"{i:05d}.obj"
            file.write_bytes(b"ignore")
        result = subprocess.run(
            ["git", "ls-files", "--others", "--exclude-standard", "-z"],
            cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True,
        )
        untracked = {part.decode("utf-8") for part in result.stdout.split(b"\0") if part}
        assert "assets/README.md" in untracked
        assert "src/main.rs" in untracked
        assert ".gitignore" in untracked
        assert not candidates.intersection(untracked), sorted(candidates.intersection(untracked))
        assert len(untracked) == 4, sorted(untracked)
        print("GIT IGNORE: PASS / excludes all local runtime, builds, archives and hydrated assets", flush=True)
else:
    print("GIT IGNORE: SKIP / Git not installed", flush=True)

# Publish verification must print a completion/phase message before the
# expensive query. This also proves a command is not silently pushing source.
lines = []
original_readiness = PCC.git_publish_readiness
original_print = PCC.pcc_print
try:
    def readiness(*, progress=None):
        assert progress is not None
        progress("synthetic expensive Git query")
        time.sleep(0.02)
        return False, "synthetic blocked", {"changedPaths": []}

    PCC.git_publish_readiness = readiness
    PCC.pcc_print = lambda *a, **kw: lines.append(" ".join(str(x) for x in a))
    ready, reason, state = PCC._interactive_publish_readiness()
    assert not ready and reason == "synthetic blocked"
    assert "No upload has started" in lines[0]
    assert any("synthetic expensive Git query" in line for line in lines)
    assert any("Finished in " in line for line in lines)
finally:
    PCC.git_publish_readiness = original_readiness
    PCC.pcc_print = original_print
print("PUBLISH PROGRESS: PASS / visible before Git queries, phased and timed", flush=True)

# The original source authorization remains fail closed; the ignore list only
# keeps local files out of the untracked scan, it does not override the rules.
for path in ["assets/elizawy/Characters/Bodies/a.png", "target/debug/b.exe", ".pcc/logs/test.log", "updates/inbox/unsafe.pccpatch.zip"]:
    assert PCC.git_publish_path_error(path) is not None, path
for path in ["assets/README.md", "src/main.rs", "Cargo.toml"]:
    assert PCC.git_publish_path_error(path) is None, path
print("PUBLISH AUTHORIZATION: PASS / forbidden payloads remain blocked; source allowed", flush=True)
