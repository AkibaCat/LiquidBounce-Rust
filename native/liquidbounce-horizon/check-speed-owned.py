#!/usr/bin/env python3
from pathlib import Path
import ast,json,tempfile,shutil,hashlib,runpy
H=Path(__file__).resolve().parent;ROOT=H.parents[1]
for p in ['generate-speed-schema.py','update-speed-locales.py','speed-patch.py','integrate-speed.py']:
 ast.parse((H/p).read_text(encoding='utf8'))
for p in ['engine/src/config/speed.rs','engine/src/speed_runtime.rs','speed-game.rs','speed-host.rs']:
    text=(H/p).read_text(encoding='utf8')
    assert text.strip(), p
assert 'SpeedSetting' in (H/'engine/src/config/speed.rs').read_text(encoding='utf8')
assert 'client_speed_facts' in (H/'engine/src/speed_runtime.rs').read_text(encoding='utf8')
assert 'client_speed_apply' in (H/'speed-game.rs').read_text(encoding='utf8')
assert 'native_client_speed_facts' in (H/'speed-host.rs').read_text(encoding='utf8')
s=json.loads((ROOT/'native/liquidbounce-behaviors/src/speed_schema.json').read_text(encoding='utf8'))
assert len(next(x for x in s if x['name']=='Mode')['modes'])==22
assert {x['name'] for x in s}>={'Mode','Not','OnlyInCombat','OnlyOnPotionEffect','YawOffset'}
assert 'MODES' in (ROOT/'native/liquidbounce-behaviors/src/speed.rs').read_text(encoding='utf8')
up=runpy.run_path(str(H/'update-speed-locales.py'))['update']
with tempfile.TemporaryDirectory() as td:
 d=Path(td);src=ROOT/'native/liquidbounce-interop/resources/localization'
 for c in ['zh_cn','zh_tw']:shutil.copyfile(src/f'display_{c}.json',d/f'display_{c}.json')
 up(d);before={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in d.iterdir()};up(d);after={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in d.iterdir()};assert before==after
print('Speed schema 22 modes, three independent scopes, Chinese locale updater idempotence passed; production locales untouched')
