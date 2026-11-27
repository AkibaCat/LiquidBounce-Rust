"""Static checks for the minimal world-module bridge; no compiler or staging build."""
from pathlib import Path
import json
ROOT=Path(__file__).parent
engine=ROOT/'engine/src'; behavior=ROOT.parent/'liquidbounce-behaviors/src'
mods=['AutoShop','AutoBuild','AutoFarm','CrystalAura','Fucker','AutoTrap','BlockTrap','Nuker']
for p in [behavior/'world_noop.rs', behavior/'lib.rs', engine/'behaviors.rs', engine/'config.rs', engine/'config/world_noop.rs', engine/'actor.rs', engine/'runtime.rs', engine/'world_noop_runtime.rs', engine/'lib.rs']:
    assert p.exists(), p
for m in mods:
    assert f'"{m}"' in (engine/'behaviors.rs').read_text()
    assert f'{m}=>' in (engine/'config.rs').read_text()
    assert f'ModuleKind::{m}' in (engine/'actor.rs').read_text()
for fn in ('display_zh_cn.json','display_zh_tw.json'):
    d=json.loads((ROOT.parent/'liquidbounce-interop/resources/localization'/fn).read_text(encoding='utf8'))
    for m in mods: assert d['moduleNames'][m] != m
print(f'world noop modules static check passed: {len(mods)} modules; no compiler invoked')
