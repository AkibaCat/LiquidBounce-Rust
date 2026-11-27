"""Validate the transaction and native patch in memory; no Rust compiler."""
from pathlib import Path
import argparse,hashlib,json,runpy,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);args=p.parse_args();out=ROOT/args.out
 assert not out.exists(),'Refusing overwrite of evidence';out.mkdir(parents=True)
 shared=runpy.run_path(str(HERE/'integrate-stronghold-finder.py'))['prepare']();texts={};base=ROOT/'build/staging/horizon'
 def edit(path,op):texts[path]=op(texts.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 def once(s,a,b):assert s.count(a)==1,(a,s.count(a));return s.replace(a,b,1)
 metadata=runpy.run_path(str(HERE/'stronghold-finder-patch.py'))['apply'](edit,once)
 for name,text in texts.items():
  path=out/'patched'/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text,encoding='utf8')
 rust=[str(out/'patched'/n)for n in texts if n.endswith('.rs')]
 result=subprocess.run(['rustfmt','+stable','--edition','2024','--config','skip_children=true',*rust],capture_output=True,text=True,encoding='utf8',errors='replace');(out/'rustfmt.log').write_text(result.stdout+result.stderr,encoding='utf8')
 assert result.returncode==0,'Native Rust syntax formatting failed'
 report={'status':'static-passed-no-compiler-invoked','sharedFiles':len(shared.changed),'nativeFiles':len(texts),'metadata':metadata,'sources':{n:hashlib.sha256(t.encode()).hexdigest()for n,t in texts.items()},'scope':'Complete settings, deterministic hypotheses and Bayesian model; local/remote consumed Item.use facts; queued actual eye-spawn packets; actual eye positions/velocity, resident-section revision tracking, world primitives and HUD. Type checking and focused tests deferred to grouped batch.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
if __name__=='__main__':main()
