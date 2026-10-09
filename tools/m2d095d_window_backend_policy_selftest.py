from __future__ import annotations

import json
import re
from pathlib import Path
from source_version_contract import cargo_source_version, project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"M2D095D WINDOW BACKEND POLICY SELFTEST: FAIL - {message}")


def main() -> int:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    main_rs = (ROOT / "src/main.rs").read_text(encoding="utf-8")
    forge = (ROOT / "ForgePY.py").read_text(encoding="utf-8")
    pcc = (ROOT / "ProjectControlCenter.py").read_text(encoding="utf-8")
    project = json.loads((ROOT / "project/forgepy.project.json").read_text(encoding="utf-8"))

    require(source_at_least(cargo, "0.10.13"), "Cargo source predates 0.10.13")
    require(project_matches_cargo(project, cargo), "project sourceVersion does not match Cargo")
    require('fn runtime_window_resizable' in main_rs, "native window policy helper missing")
    require('HAVENWILD_DX12_ALLOW_RESIZE' in main_rs, "DX12 resize guard environment contract missing")
    require('backend.eq_ignore_ascii_case("dx12")' in main_rs, "DX12 backend guard missing")
    require('sheet.path.starts_with("Terrain/")' in main_rs, "Terrain-only hydration numerator repair missing")
    require('default="vulkan"' in forge, "ForgePY normal run is not Vulkan-first")
    require('"HAVENWILD_DX12_ALLOW_RESIZE": "1" if allow_dx12_resize else "0"' in forge, "ForgePY DX12 resize guard propagation missing")
    require('Run Studio (Vulkan - recommended / resizable)' in pcc, "PCC recommended Vulkan run missing")
    require('Run Studio (DX12 safe window - fixed size)' in pcc, "PCC DX12 safe mode missing")
    require('Run Studio (DX12 resize diagnostic - may trigger upstream crash)' in pcc, "PCC explicit DX12 resize diagnostic missing")
    require('allow_dx12_resize=True' in pcc, "PCC diagnostic does not opt into DX12 resizing")
    print("M2D095D WINDOW BACKEND POLICY SELFTEST: PASS / Vulkan normal path; DX12 fixed-size safe path; explicit resize diagnostics; Terrain-only hydration count")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
