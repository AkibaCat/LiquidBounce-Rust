from pathlib import Path
import json
mods=['AutoShop','AutoBuild','AutoFarm','CrystalAura','Fucker','AutoTrap','BlockTrap','Nuker','TickBase','Backtrack','Fly','LongJump']
beh=Path('native/liquidbounce-horizon/engine/src/behaviors.rs').read_text(); cfg=Path('native/liquidbounce-horizon/engine/src/config.rs').read_text(); actor=Path('native/liquidbounce-horizon/engine/src/actor.rs').read_text()
for m in mods:
 assert ('"'+m+'"') in beh
 assert ('    '+m+'=>') in cfg
 assert ('ModuleKind::'+m) in actor and ('RuntimeRequest::'+m) in actor
for f in ['display_zh_cn.json','display_zh_tw.json']:
 d=json.loads(Path('native/liquidbounce-interop/resources/localization',f).read_text(encoding='utf8')); assert all(d['moduleNames'][m]!=m for m in mods)
print('world noop registration smoke passed:', len(mods), 'modules')
