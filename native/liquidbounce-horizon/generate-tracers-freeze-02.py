from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 target=HERE/'tracers-freeze-02.json';assert not target.exists()
 data=json.loads((HERE/'tracers-freeze.json').read_text(encoding='utf8'));data['revision']=2;data['revisionNote']='Corrected copied ModuleKind locale key damageParticles to original tracers; implementation unchanged; no repeated Cargo test.'
 for name in ['integrate-tracers.py','tracers-revision-02.md','generate-tracers-freeze-02.py']:
  p=HERE/name;data['files'][p.relative_to(ROOT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
 target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
