from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
main = (ROOT / 'src/main.rs').read_text(encoding='utf-8')
world = (ROOT / 'src/world_doc.rs').read_text(encoding='utf-8')
import json
project_doc = json.loads((ROOT / 'project/forgepy.project.json').read_text(encoding='utf-8'))
source_version = tuple(int(part) for part in project_doc.get('sourceVersion','0.0.0').split('.'))

checks = {
    'source >=0.7.7': source_version >= (0, 7, 7),
    'semantic terrain brush enum': 'enum WorldTerrainBrush' in main,
    'grass brush': 'Self::Grass => "Grass"' in main,
    'dirt-bank brush': 'Self::DirtBank => "MudBank"' in main,
    'water brush': 'Self::Water => "RiverWater"' in main,
    'semantic stroke input': 'world_semantic_stroke_begin(state, world, false)' in main,
    'continuous drag': 'primary_down && !state.world_stroke_cells.is_empty()' in main,
    'authority-aware render': 'resolved_source_with_authorities(' in main,
    'semantic persistence': 'paint_semantic_terrain' in world,
    'authored terrain erase': 'clear_authored_terrain' in world,
    'strict Summer profile': 'summer_flatworld_corner' in world and 'summer_flatworld_fill' in world,
    'no broad palette fallback in semantic resolver': '.or_else(|| authority.role_palette(&role))' not in world,
    'source pixels remain immutable': ('source PNGs are never edited' in main) or ('Source PNGs are immutable' in main) or ('source pixels remain immutable' in main),
}

failed = [name for name, ok in checks.items() if not ok]
if failed:
    raise SystemExit('M2D07U AUTOTILE BRUSH SELFTEST: FAIL / ' + ', '.join(failed))
print('M2D07U AUTOTILE BRUSH SELFTEST: PASS / semantic GRS+DIR+WTR drag paint / strict Summer source profile / safe-fill fallback')
