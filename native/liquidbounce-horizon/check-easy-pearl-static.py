"""Static composition only; Rust and Cargo are reserved for root's multi-module batch."""
from pathlib import Path
import argparse,json,runpy,shutil,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out
 if out.exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True);plan=runpy.run_path(str(HERE/'integrate-easy-pearl.py'))['prepare']()
 patched={};base=ROOT/'build/staging/horizon'
 def once(s,o,n):
  assert s.count(o)==1,(o,s.count(o));return s.replace(o,n)
 def edit(path,op):
  p=base/path;patched[path]=op(patched.get(path,p.read_text(encoding='utf8')if p.exists()else''))
 # Root may be integrating AutoPearl while these owned sources are prepared.
 if 'mod auto_pearl;' not in (base/'crates/horizon-client/src/game.rs').read_text(encoding='utf8'):
  runpy.run_path(str(HERE/'auto-pearl-patch.py'))['apply'](edit,once)
 meta=runpy.run_path(str(HERE/'easy-pearl-patch.py'))['apply'](edit,once)
 hooks=patched['crates/horizon-client/src/game/native_hooks.rs'];game=patched['crates/horizon-client/src/game/easy_pearl.rs']
 assert 'fn native_client_easy_pearl_query(&self' in hooks
 assert 'self.client_easy_pearl_local_use(hand,yaw,pitch)' in hooks
 for name in ['remote.rs','use_hand.rs']:assert 'self.client_easy_pearl_interact()'in patched[f'crates/horizon-client/src/game/{name}']
 for s in ['direction.scale(1000.)','auto_pearl_rotation','0.7','liquidbounce_target_metrics','client_easy_pearl_overlay','LOCAL_PREUSED']:
  assert s in game,s
 for name,s in patched.items():
  p=out/'patched'/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf8')
 loc=out/'localization';loc.mkdir();sys.path.insert(0,str(HERE))
 for lang in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{lang}.json',loc/f'display_{lang}.json')
 runpy.run_path(str(HERE/'update-easy-pearl-locales.py'))['update'](loc)
 keys={}
 for lang in ['zh_cn','zh_tw']:
  d=json.loads((loc/f'display_{lang}.json').read_text(encoding='utf8'));keys[lang]=len([k for k in d['descriptions']if k.startswith('liquidbounce.module.easyPearl')]);assert keys[lang]>=24
  original=json.loads((HERE.parent/f'liquidbounce-interop/resources/localization/{lang}.json').read_text(encoding='utf8'));assert original['liquidbounce.module.easyPearl.messages.noInReachWarning']
 report={'status':'static-passed-compile-deferred','sharedFiles':len(plan.changed),'patchedFiles':len(patched),'metadata':meta,'descriptions':keys,'scope':'Transactional anchors; real native useItem HEAD / OUTLINE ray / WorldPrimitives / HUD composition; display locale schema and original warning keys. No Rust/Cargo/GPU or actual-game parity claim. Font metrics inherit shared renderer and active custom-theme switching remains the next task.'}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
if __name__=='__main__':main()
