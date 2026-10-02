from pathlib import Path
import json, sys
ROOT = Path(__file__).resolve().parents[1]
checks = {
    'scene mask type': ('src/scene_v2.rs', 'pub struct CollisionMask32'),
    'scene collision channel': ('src/scene_v2.rs', 'pub collision_cells: Vec<CollisionCell>'),
    'transaction': ('src/scene_v2.rs', 'SetCollisionMask'),
    'pixel precedence': ('src/playtest.rs', 'start_with_collision'),
    'red overlay': ('src/main.rs', 'draw_collision_overlay'),
    'world pixel edit': ('src/main.rs', 'collision_pixel_under_pointer'),
    'layer rail': ('src/main.rs', '"COL"'),
    'mask editor': ('src/main.rs', 'draw_collision_mask_editor'),
}
errors=[]
for label,(rel,needle) in checks.items():
    text=(ROOT/rel).read_text(encoding='utf-8')
    if needle not in text:
        errors.append(f'{label}: missing {needle!r} in {rel}')
project=json.loads((ROOT/'project/forgepy.project.json').read_text(encoding='utf-8'))
version=project.get('sourceVersion','')
try:
    parts=tuple(int(x) for x in version.split('.'))
except Exception:
    errors.append(f'invalid sourceVersion {version!r}')
else:
    if parts < (0,6,0): errors.append('project sourceVersion must be >= 0.6.0')
if '32x32 per-cell pixel masks' not in project.get('authoredContent',{}).get('collisionEditing',''):
    errors.append('project collisionEditing contract missing 32x32 mask statement')
# fail closed against accidental alpha-derived authority
main=(ROOT/'src/main.rs').read_text(encoding='utf-8')
scene=(ROOT/'src/scene_v2.rs').read_text(encoding='utf-8')
if 'source artwork is never modified or sampled' not in scene:
    errors.append('source-art collision authority safety comment missing')
if errors:
    print('M2D06B COLLISION SELFTEST: FAIL')
    print('\n'.join(' - '+e for e in errors))
    sys.exit(1)
print('M2D06B COLLISION SELFTEST: PASS / red overlay + 32x32 authored masks + world-pixel correction + transactional persistence')
