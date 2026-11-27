from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]

def main():
    target=HERE/'placement-esp-freeze-02.json';assert not target.exists()
    data=json.loads((HERE/'placement-esp-freeze.json').read_text(encoding='utf8'));data['revision']=2;data['revisionNote']='Register ProphuntESP aliases in functioning ModuleKind lookup; implementation unchanged; no repeated smoke.'
    for name in ['integrate-placement-esp.py','placement-esp-revision-02.md','generate-placement-esp-freeze-02.py']:
        p=HERE/name;data['files'][p.relative_to(ROOT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
    target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())

if __name__=='__main__':main()
