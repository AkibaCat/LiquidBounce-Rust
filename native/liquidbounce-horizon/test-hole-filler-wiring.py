from pathlib import Path
import json
files=['native/liquidbounce-horizon/engine/src/behaviors.rs','native/liquidbounce-horizon/engine/src/config.rs','native/liquidbounce-horizon/engine/src/actor.rs','native/liquidbounce-horizon/engine/src/runtime.rs','native/liquidbounce-horizon/engine/src/lib.rs']
for f in files: assert 'hole_filler' in Path(f).read_text().lower(), f
for f in ['display_zh_cn.json','display_zh_tw.json']:
 d=json.loads(Path('native/liquidbounce-interop/resources/localization',f).read_text(encoding='utf8')); assert d['moduleNames']['HoleFiller'] != 'HoleFiller'
print('HoleFiller wiring smoke passed: 5 shared engine files; no compiler invoked')
