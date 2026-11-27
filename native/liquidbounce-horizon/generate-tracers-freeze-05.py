from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]

def main():
    target=HERE/'tracers-freeze-05.json';assert not target.exists()
    data=json.loads((HERE/'tracers-freeze-04.json').read_text(encoding='utf8'));data['revision']=5;data['revisionNote']='Align future focused fixture with actual SettingsPatch declaration; production remains revision04; no repeated smoke.'
    for name in ['test-tracers.py','tracers-revision-05.md','generate-tracers-freeze-05.py']:
        p=HERE/name;data['files'][p.relative_to(ROOT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
    target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())

if __name__=='__main__':main()
