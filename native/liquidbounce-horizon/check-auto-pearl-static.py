"""Static composition only. The focused Rust harness is executed by root's module batch."""
from pathlib import Path
import argparse,json,runpy,shutil,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out
 if out.exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True);plan=runpy.run_path(str(HERE/'integrate-auto-pearl.py'))['prepare']()
 patched={};base=ROOT/'build/staging/horizon'
 def once(s,o,n):
  assert s.count(o)==1,(o,s.count(o));return s.replace(o,n)
 def edit(path,op):
  p=base/path;patched[path]=op(patched.get(path,p.read_text(encoding='utf8')if p.exists()else''))
 meta=runpy.run_path(str(HERE/'auto-pearl-patch.py'))['apply'](edit,once)
 assert 'fn native_client_auto_pearl_query' in patched['crates/horizon-client/src/game/native_hooks.rs']
 assert 'fn auto_pearl_collision_boxes' in patched['crates/horizon-client/src/game/auto_pearl.rs']
 assert 'self.auto_pearl_spawn' in patched['crates/horizon-client/src/game/remote.rs']
 for name,s in patched.items():
  p=out/'patched'/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf8')
 loc=out/'localization';loc.mkdir();sys.path.insert(0,str(HERE))
 for lang in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{lang}.json',loc/f'display_{lang}.json')
 runpy.run_path(str(HERE/'update-auto-pearl-locales.py'))['update'](loc)
 keys={}
 for lang in ['zh_cn','zh_tw']:
  d=json.loads((loc/f'display_{lang}.json').read_text(encoding='utf8'));keys[lang]=len([k for k in d['descriptions']if k.startswith('liquidbounce.module.autoPearl')]);assert keys[lang]>=len(runpy.run_path(str(HERE/'update-auto-pearl-locales.py'))['TEXTS'])+20
 report={'status':'static-passed-compile-deferred','sharedFiles':len(plan.changed),'patchedFiles':len(patched),'metadata':meta,'descriptions':keys,'scope':'Static transactional anchors, native patch composition and localization schema. Rust/Cargo checks deliberately deferred to root multi-module batch. Target mode waits for actual KillAura target dependency.'}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
if __name__=='__main__':main()
