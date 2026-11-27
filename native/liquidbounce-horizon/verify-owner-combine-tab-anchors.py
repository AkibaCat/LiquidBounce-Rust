"""Read-only final composition proof, without staging or compiling production."""
from pathlib import Path
import runpy,json,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 out=ROOT/'build/owner-combine-tab-final-anchors'
 if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);base=ROOT/'build/staging/horizon';patched={};inputs={}
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):
  if path not in patched:
   original=(base/path).read_text(encoding='utf8')if(base/path).exists()else'';patched[path]=original;inputs[path]=hashlib.sha256(original.encode()).hexdigest()
  patched[path]=op(patched[path])
 metadata={}
 for slug in ['mob-owners','combine-mobs','better-tab']:metadata[slug]=runpy.run_path(str(HERE/f'{slug}-patch.py'))['apply'](edit,once)
 for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 report={'passed':True,'scope':'Final owned patches composed against current actual staged source in memory only. No production staging mutation or compilation. Includes final read-only acting_uuid helper and hidden-row short circuit.','metadata':metadata,'inputSha256':inputs,'resultSha256':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in patched.items()}}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':True,'patchedFiles':len(patched)}))
if __name__=='__main__':main()
