#!/usr/bin/env python3
"""Check that quick PCC menu avoids full Gate/Publish scans, without weakening certification."""
from __future__ import annotations

import builtins
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('havenwild_m2d108_pcc', ROOT / 'ProjectControlCenter.py')
assert SPEC and SPEC.loader
PCC = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PCC)


def previous_fingerprint(root: Path) -> tuple[str, list[str]]:
    """Original v1 implementation retained only in regression code."""
    digest = hashlib.sha256()
    digest.update(b'havenwild.source.snapshot.v1\0')
    paths = []
    for path in sorted(root.rglob('*'), key=lambda p: p.as_posix().lower()):
        if not path.is_file():
            continue
        rel = path.relative_to(root).as_posix()
        if PCC.git_publish_path_error(rel):
            continue
        paths.append(rel)
        digest.update(rel.encode('utf-8', errors='surrogateescape'))
        digest.update(b'\0')
        try:
            digest.update(hashlib.sha256(path.read_bytes()).digest())
        except OSError:
            digest.update(b'<unreadable>')
        digest.update(b'\0')
    return digest.hexdigest(), paths


with tempfile.TemporaryDirectory(prefix='havenwild-fast-pcc-') as dirname:
    root = Path(dirname)
    for rel, data in {
        'Cargo.toml': b'hello\n', 'src/main.rs': b'world\n', 'tools/a.py': b'x\n',
        'assets/README.md': b'allowed\n', 'assets/elizawy/Characters/ABC.png': b'payload\n',
        '.pcc/job.json': b'job', '.forgepy/index.json': b'index',
        '.git/HEAD': b'ref', 'target/large.bin': b'target',
        'updates/inbox/a.zip': b'intake', 'updates/archive/a.txt': b'archive',
        'updates/retained-source.md': b'allow',
        'artifacts/certification/data.json': b'artifact',
        'docs/README.md': b'doc', 'content/catalog/source_index.local.json': b'not-source',
        'content/test.json': b'valid'
    }.items():
        dst = root / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(data)
    # A large hydrated library should never be reached by certification walk.
    for n in range(300):
        (root / 'assets/elizawy/Characters' / f'{n:05}.png').write_bytes(b'asset')
    for n in range(150):
        (root / 'target' / f'{n:05}.obj').write_bytes(b'build')

    expected = previous_fingerprint(root)
    previous_root, previous_updates = PCC.ROOT, PCC.UPDATE_ROOT
    PCC.ROOT, PCC.UPDATE_ROOT = root, root / 'updates'
    try:
        observed = PCC.source_snapshot_fingerprint()
    finally:
        PCC.ROOT, PCC.UPDATE_ROOT = previous_root, previous_updates
    assert expected == observed, (expected, observed)
    assert 'assets/README.md' in observed[1]
    assert not any(path.startswith('assets/elizawy/') for path in observed[1])
    print('SOURCE SNAPSHOT: PASS / byte-identical v1 certification; skip hydrated assets and target', flush=True)

# Show the menu without touching slow full read/Git state: this must not invoke
# publication readiness on a normal menu paint, including on startup.
original_dashboard = PCC.dashboard
original_readiness = PCC.git_publish_readiness
original_input = builtins.input
original_print = PCC.pcc_print
try:
    PCC.dashboard = lambda: 0
    PCC.git_publish_readiness = lambda: (_ for _ in ()).throw(AssertionError('expensive Git readiness called from normal menu'))
    builtins.input = lambda _: '0'
    lines = []
    PCC.pcc_print = lambda *parts, **kw: lines.append(' '.join(map(str, parts)))
    result = PCC.menu()
    assert result == 0
    assert any('VERIFY ON SELECTION' in line or 'FULL GATE REQUIRED' in line for line in lines)
finally:
    PCC.dashboard = original_dashboard
    PCC.git_publish_readiness = original_readiness
    PCC.pcc_print = original_print
    builtins.input = original_input
print('FAST MENU: PASS / no source fingerprint or Git readiness during normal startup', flush=True)

controller = (ROOT / 'ProjectControlCenter.py').read_text(encoding='utf-8')
assert 'ready, reason, state = git_publish_readiness()' in controller  # publish is still fail closed
assert '"sourceControl": git_worktree_state()' in controller  # Gate still certifies full source
assert '"git", "status", "--porcelain"' not in controller[controller.index('def dashboard()'):controller.index('def _zip_has_patch_manifest')]
print('CERTIFICATION GUARDED: PASS / Full Gate and Commit/Push retain full validation', flush=True)
