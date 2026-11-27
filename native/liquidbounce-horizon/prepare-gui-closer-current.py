#!/usr/bin/env python3
"""Snapshot current cumulative source for focused tests, without a client build."""
from pathlib import Path
import argparse,hashlib,json
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
    if out.exists():raise SystemExit('Refusing overwrite')
    generator=HERE/'make-behavior-patch.py';ns={'__file__':str(generator),'__name__':'gui_closer_source_snapshot'}
    exec(compile(generator.read_text(encoding='utf8').split('destination = HERE /',1)[0],str(generator),'exec'),ns)
    hashes={}
    for name,(_,text) in ns['changes'].items():
        path=out/'horizon'/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text,encoding='utf8',newline='\n');hashes[name]=hashlib.sha256(text.encode()).hexdigest()
    report={'passed':True,'scope':'Current cumulative canonical transformation snapshot for focused actual-body tests only. No full Horizon copy, API generation, Cargo or game validation.','snapshotSha256':hashes}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(out)
if __name__=='__main__':main()
