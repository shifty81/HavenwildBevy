#!/usr/bin/env python3
"""M2D131C PCC: useful compiler errors, one ZIP, reopen, no blocking GUI on CI.

This uses a temporary PCC state without invoking Windows applications or Cargo.
"""
from __future__ import annotations
import importlib.util
import json
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('havenwild_pcc_failure_selftest', ROOT/'ProjectControlCenter.py')
assert spec and spec.loader
pcc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pcc)
with tempfile.TemporaryDirectory(prefix='hw-pcc-failure-') as td:
    testroot=Path(td)
    old = {key:getattr(pcc,key) for key in ('ROOT','STATE_ROOT','LOG_ROOT','RECEIPT_ROOT','DEBUG_ROOT','GATE_ROOT','ROLLBACK_ROOT','STAGING_ROOT','LAST_GATE','LAST_JOB','LAST_FAILURE','LAST_FAILURE_REPORT','_SESSION_PATH','_LATEST_DEBUG_BUNDLE','installed_patch_state','make_debug_bundle','open_failure_artifacts')}
    try:
        pcc.ROOT = testroot
        pcc.STATE_ROOT = testroot/'.pcc'
        for key,sub in [('LOG_ROOT','logs'),('RECEIPT_ROOT','receipts'),('DEBUG_ROOT','debug'),('GATE_ROOT','gates'),('ROLLBACK_ROOT','rollback'),('STAGING_ROOT','staging')]:
            setattr(pcc,key,pcc.STATE_ROOT/sub)
        pcc.LAST_JOB = pcc.STATE_ROOT/'last_job.json'
        pcc.LAST_GATE = pcc.STATE_ROOT/'last_gate.json'
        pcc.LAST_FAILURE = pcc.STATE_ROOT/'last_failure.json'
        pcc.LAST_FAILURE_REPORT = pcc.STATE_ROOT/'LAST_FAILURE.txt'
        pcc._SESSION_PATH = None
        pcc.ensure_dirs()
        log=pcc.LOG_ROOT/'cargo-check.log'
        log.write_text('Checking game\nerror[E0599]: no method named `style` found\n --> src/canvas_chrome.rs:106:27\nerror[E0502]: cannot borrow `*state` as mutable\n --> src/main.rs:8603:74\ncould not compile havenwild_bevy\n',encoding='utf-8')
        pcc.LAST_JOB.write_text(json.dumps({'jobLog':str(log.relative_to(testroot)),'result':'FAIL'}))
        pcc.LAST_GATE.write_text(json.dumps({'result':'FAIL','phases':[{'name':'Cargo check','result':'FAIL','jobLog':str(log.relative_to(testroot))}]}))
        bundle=pcc.DEBUG_ROOT/'debug.zip';bundle.write_bytes(b'safe test fixture')
        pcc._LATEST_DEBUG_BUNDLE = bundle
        pcc.installed_patch_state = lambda: {'source':'0.10.57','pcc':'1.3.6','forgepy':'0.4.10'}
        pcc.make_debug_bundle = lambda **kwargs: (_ for _ in ()).throw(AssertionError('duplicate debug ZIP'))
        calls=[]
        pcc.open_failure_artifacts = lambda report, archive: calls.append((report, archive)) or False
        report=pcc.show_failure_handoff(101, gate_failure=True)
        body=report.read_text()
        for part in ('error[E0599]','error[E0502]','Cargo check','debug.zip','0.10.57'):
            assert part in body,part
        assert pcc.LAST_FAILURE.is_file()
        pcc.reopen_last_failure()
        assert len(calls)==2 and all(archive==bundle for _,archive in calls)
        assert pcc._failed_log_excerpt(None).startswith('No job log available')
        # Gate wrapper always resets the context, even when the inner gate crashes.
        pcc._full_gate_impl = lambda: 101
        assert pcc.full_gate()==101 and not pcc._IN_FULL_GATE
        def crash(): raise RuntimeError('gate test')
        pcc._full_gate_impl = crash
        try: pcc.full_gate()
        except RuntimeError: pass
        else: raise AssertionError('gate error hidden')
        assert not pcc._IN_FULL_GATE
    finally:
        for key,value in old.items():setattr(pcc,key,value)
print('M2D131C PCC FAILURE HANDOFF: PASS / compiler diagnostics extracted, one debug ZIP, reopen, safe gate reset')
