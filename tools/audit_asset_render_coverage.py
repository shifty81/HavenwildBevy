#!/usr/bin/env python3
"""Havenwild source-to-render coverage: discoverability != usable render binding.

This is a read-only authority/report check. It NEVER promotes candidate semantic
roles or invents atlas crops. --verify-hydrated additionally hashes original
PNG bytes when a workstation has hydrated the original source collection.
"""
from __future__ import annotations
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / 'artifacts/asset_lane/render_coverage.v1.json'


def load(relative):
    return json.loads((ROOT / relative).read_text(encoding='utf-8'))


def collect():
    catalog = load('content/catalog/core_source_manifest.json')
    runtime = load('content/assets/authority/elizawy_runtime_index.v1.json')
    source = {row['path']: row for row in catalog['entries'] if row['path'].endswith('.png')}
    images = {row['sourcePath']: row for row in runtime['runtimeSourceImages']}
    regions = defaultdict(list)
    for region in runtime['canonicalRuntimeRegions']:
        regions[region['sourcePath']].append(region)
    template_use = defaultdict(list)
    for template in runtime['objectTemplates']:
        for part in template['parts']:
            template_use[part['sourcePath']].append(template['templateId'])
    issues = []
    if len(source) != 320 or set(source) != set(images):
        issues.append(f'core catalog and runtime image identity diverge: source={len(source)} runtime={len(images)}')
    if len(runtime['canonicalRuntimeRegions']) != runtime['counts']['canonicalRuntimeRegions']:
        issues.append('canonical runtime region count mismatch')
    if len(runtime['objectTemplates']) != runtime['counts']['objectTemplates']:
        issues.append('reviewed object template count mismatch')
    for path, rec in images.items():
        entry = source.get(path)
        if entry is None or rec['sourceSha256'] != entry['sha256'] or rec['imageSizePx'] != entry['imageSizePx']:
            issues.append(f'original PNG provenance mismatch: {path}')
    for path, items in regions.items():
        entry = source.get(path)
        for item in items:
            if entry is None:
                issues.append(f'region outside canonical source image catalog: {path}')
                continue
            x, y, w, h = item['sourceRectPx']
            iw, ih = entry['imageSizePx']
            if min(x, y) < 0 or min(w,h) <= 0 or x + w > iw or y + h > ih:
                issues.append(f'out-of-bounds canonical region: {path} {item["sourceRectPx"]}')
    for template in runtime['objectTemplates']:
        for part in template['parts']:
            addr = (part['sourcePath'], tuple(part['sourceRectPx']))
            registered = {(x['sourcePath'], tuple(x['sourceRectPx'])) for x in regions.get(part['sourcePath'], [])}
            if addr not in registered:
                issues.append(f'template part lacks canonical region: {template["templateId"]} {addr}')
    by_family = Counter()
    with_regions = Counter()
    with_templates = Counter()
    per_source = []
    for path in sorted(images):
        family = images[path]['family']
        by_family[family] += 1
        if regions.get(path): with_regions[family] += 1
        if template_use.get(path): with_templates[family] += 1
        # This is deliberately a stage classification rather than a single
        # misleading yes/no "fully mapped" value.
        stages = ['catalogued', 'atlas_browsable_on_demand']
        if regions.get(path): stages.append('exact_canonical_region_enumerated')
        if template_use.get(path): stages.append('reviewed_object_template_uses')
        per_source.append({
            'sourcePath': path, 'family': family,
            'semanticCategory': images[path].get('semanticCategory', family),
            'canonicalRegionCount': len(regions.get(path, [])),
            'templateIds': sorted(set(template_use.get(path, []))),
            'stages': stages,
            'sourceHash': source[path]['sha256'],
            'semanticGameplayCertification': 'not_claimed_by_this_audit',
            'rendererRuntimeVerification': 'pending_native_studio_and_pie',
        })
    summary = {
        'cataloguedPngSheets': len(images),
        'sheetsWithCanonicalRegions': sum(bool(regions.get(p)) for p in images),
        'sheetsWithoutCanonicalRegions': sum(not regions.get(p) for p in images),
        'canonicalRuntimeRegionAddresses': sum(map(len, regions.values())),
        'reviewedObjectTemplates': len(runtime['objectTemplates']),
        'sheetsUsedByReviewedTemplates': sum(bool(template_use.get(p)) for p in images),
        'byFamily': {k: {
            'catalogued': by_family[k],
            'hasCanonicalRegions': with_regions[k],
            'hasReviewedTemplate': with_templates[k],
            'withoutRegions': by_family[k] - with_regions[k],
        } for k in sorted(by_family)},
        'originalArtMutationAllowed': False,
        'rolePromotions': 0,
        'runtimeAllSheetsVerified': False,
    }
    return {'schema': 'havenwild.bevy.asset_render_coverage.v1',
            'sourceVersion': load('project/forgepy.project.json')['sourceVersion'],
            'summary':summary,'sheets':per_source,'issues':issues}


def verify_hydrated(report):
    found = 0
    missing = []
    mismatched = []
    for row in report['sheets']:
        file = ROOT / 'assets/elizawy' / row['sourcePath']
        if not file.is_file():
            missing.append(row['sourcePath'])
        elif hashlib.sha256(file.read_bytes()).hexdigest() != row['sourceHash']:
            mismatched.append(row['sourcePath'])
        else:
            found += 1
    report['hydrated'] = {'sha256VerifiedPngs':found,'missing':missing,'mismatched':mismatched}
    if missing or mismatched:
        report['issues'].append(f'hydration incomplete: {len(missing)} missing; {len(mismatched)} changed PNG(s)')


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--check',action='store_true')
    ap.add_argument('--verify-hydrated',action='store_true')
    ap.add_argument('--report', type=Path, default=REPORT)
    args=ap.parse_args()
    report=collect()
    if args.verify_hydrated: verify_hydrated(report)
    s=report['summary']
    print(f"ASSET RENDER COVERAGE: {s['cataloguedPngSheets']} catalogued sheets / {s['sheetsWithCanonicalRegions']} with region addresses / {s['sheetsWithoutCanonicalRegions']} without")
    print(f"  Source regions: {s['canonicalRuntimeRegionAddresses']} / object templates: {s['reviewedObjectTemplates']} on {s['sheetsUsedByReviewedTemplates']} source images")
    for family, counts in s['byFamily'].items():
        print(f"  {family}: {counts['hasCanonicalRegions']}/{counts['catalogued']} image(s) have source-region addresses; {counts['hasReviewedTemplate']} contain reviewed templates")
    print('  Runtime readiness of every source, source semantics, collision and actual art: NOT certified by the catalog')
    if args.report:
        out = args.report if args.report.is_absolute() else ROOT / args.report
        out.parent.mkdir(parents=True,exist_ok=True)
        out.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
        print('  Report:',out)
    for issue in report['issues'][:20]:print('FAIL:',issue)
    if report['issues']:return 1
    print('ASSET RENDER INVENTORY: PASS / source identity+addresses+templates internally consistent; unmapped assets explicitly tracked')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
