#!/usr/bin/env python3
"""Sync verified registry coverage without changing game/differential evidence."""
from pathlib import Path
import argparse,ast,json,re
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--pending-module',action='append',default=[],help='Registered after the successful combined check; leave this module pending in the ledger.')
args=parser.parse_args()
ROOT=Path(__file__).resolve().parents[1]
config=(ROOT/'native/liquidbounce-horizon/engine/src/config.rs').read_text(encoding='utf8')
implemented={name:category for _,name,category in re.findall(r'^\s*(\w+)\s*=>\s*\("([^"]+)",\s*"([^"]+)",',config,re.M)}
if len(implemented)<103:raise ValueError('registry parser failed')
for pending in args.pending_module:
 if pending not in implemented:raise ValueError(f'unknown pending module {pending}')
 implemented.pop(pending)
ledger_path=ROOT/'research/module-parity.json'
ledger=json.loads(ledger_path.read_text(encoding='utf-8-sig'))
by_name={name.lower():name for name in implemented}
changed=[]
for module in ledger['modules']:
 name=module['symbol'].removeprefix('Module')
 if name.lower() not in by_name:continue
 name=by_name[name.lower()]
 if module['rustImplementation']!='native_logic_implemented':
  module['rustImplementation']='native_logic_implemented'
  module['parityValidation']='focused_checks_game_pending'
  snake=re.sub(r'(?<!^)(?=[A-Z][a-z])|(?<=[a-z])(?=[A-Z])','_',name).lower()
  source={
   'TriggerBot':'native/liquidbounce-behaviors/src/crosshair_clicker.rs',
   'AutoClicker':'native/liquidbounce-behaviors/src/crosshair_clicker.rs',
   'XRay':'native/liquidbounce-behaviors/src/xray.rs',
  }.get(name,f'native/liquidbounce-behaviors/src/{snake}.rs')
  if not (ROOT/source).is_file():
   source=f'native/liquidbounce-horizon/engine/src/{snake}_runtime.rs'
  if not (ROOT/source).is_file():raise ValueError(f'missing implementation for {name}: {source}')
  module['rustSource']=source
  changed.append(name)
 module['category']=implemented[name]
count=sum(m['rustImplementation']=='native_logic_implemented' for m in ledger['modules'])
if count!=len(implemented):raise ValueError(f'ledger {count} != registry {len(implemented)}')
ledger_path.write_text(json.dumps(ledger,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
verifier=ROOT/'scripts/verify-native-integration.py'
text=verifier.read_text(encoding='utf8')
render=ast.literal_eval(re.search(r'^RENDER_MODULES = (.+)$',text,re.M)[1])
text=re.sub(r'^MODULES = .+$','MODULES = '+repr(tuple(sorted(implemented,key=str.lower))),text,flags=re.M)
text=re.sub(r'^RENDER_MODULES = .+$','RENDER_MODULES = '+repr(tuple(sorted(set(render)|{n for n,c in implemented.items() if c=='Render'},key=str.lower))),text,flags=re.M)
verifier.write_text(text,encoding='utf8')
print(json.dumps({'implemented':count,'new':changed},ensure_ascii=False))
