"""Static composition and one grouped smoke for the new module batch only."""
from pathlib import Path
import argparse,ast,json,runpy,subprocess,tempfile,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def static(out):
 out.mkdir(parents=True,exist_ok=True);texts={};original={};stage=ROOT/'build/staging/horizon'
 def edit(name,fn):
  if name not in texts:
   p=stage/name;original[name]=p.read_bytes()if p.exists()else None;texts[name]=p.read_text(encoding='utf8')if original[name]is not None else''
  texts[name]=fn(texts[name])
 def once(s,a,b):
  assert s.count(a)==1,(a,s.count(a));return s.replace(a,b)
 fragments=[]
 for name in ['block-placement','placement-use','extinguish','auto-shoot','fake-lag']:
  path=HERE/f'{name}-patch.py';ast.parse(path.read_text(encoding='utf8'));fragments.append(runpy.run_path(str(path))['apply'](edit,once))
 assert 'dispatch_client_extinguish_success(self,'in(HERE/'extinguish-host.rs').read_text(encoding='utf8')
 assert "ClientExtinguishSuccessEvent<'_>"not in(HERE/'engine/src/runtime.rs').read_text(encoding='utf8')
 placement=texts['crates/horizon-client/src/game/placement_use.rs']
 for token in ['on_success(g, false)','on_success(g, true)','v >= 0','resume_unwind','self.forced_hit = previous_hit','self.guest_use = previous_guest']:
  assert token in placement,token
 for name,raw in original.items():
  path=stage/name;assert(path.read_bytes()if path.exists()else None)==raw,name
 generator=(HERE/'make-behavior-patch.py').read_text(encoding='utf8');assert generator.rfind("'step-patch.py'")>generator.rfind("'block-placement-patch.py'")
 for locale in ['zh_cn','zh_tw']:
  data=json.loads((HERE.parent/f'liquidbounce-interop/resources/localization/display_{locale}.json').read_text(encoding='utf8'))
  for key in ['extinguish','autoShoot','fakeLag']:
   assert f'liquidbounce.module.{key}.description'in data['descriptions'],(locale,key)
 report={'passed':True,'stage':'static-only','nativeFilesComposed':len(texts),'fragments':fragments,'rustExecuted':False,'scope':'Extinguish, AutoShoot, FakeLag and shared placement; no staging writes'}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
def run(out):
 out.mkdir(parents=True,exist_ok=True);rows=[]
 for name in ['extinguish','auto-shoot','fake-lag','placement-use']:
  source=(HERE/f'{name}-smoke.rs').read_text(encoding='utf8')
  import re
  source=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',lambda m:'#[path="'+(HERE/m[1]).resolve().as_posix()+'"]',source)
  path=out/f'{name}.rs';path.write_text(source,encoding='utf8');exe=out/f'{name}.exe'
  commands=[['rustc','+stable','--edition=2024',str(path),'-o',str(exe)],[str(exe)]]
  for index,command in enumerate(commands):
   result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace');log=out/f'{name}-{index}.log';log.write_text(result.stdout+result.stderr,encoding='utf8');rows.append({'name':name,'phase':index,'exitCode':result.returncode,'log':str(log.relative_to(ROOT))})
   if result.returncode:break
  if result.returncode:break
 report={'passed':all(r['exitCode']==0 for r in rows)and len(rows)==8,'checks':rows,'scope':'New modules and actual placement helper only'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));return report['passed']
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--run',action='store_true');a=p.parse_args();out=ROOT/a.out
 if a.run:raise SystemExit(0 if run(out)else 1)
 static(out)
