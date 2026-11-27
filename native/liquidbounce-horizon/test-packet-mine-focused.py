#!/usr/bin/env python3
"""Prepare production PacketMine model tests; --run is reserved for the root batch."""
from pathlib import Path
import argparse,hashlib,json,os,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def prepare(out):
 out.mkdir(parents=True,exist_ok=True);model=HERE.parent/'liquidbounce-behaviors/src/packet_mine.rs'
 report={'status':'prepared-no-compiler-invoked','tests':5,'scope':'Exact production PacketMine tests: normal threshold/update sequencing, Immediate mode cleanup, Civ air retention, missed-hit pause, and range/button-delay abort cleanup. Full engine config/native packet/slot execution remains in the unified batch.','sources':{model.relative_to(ROOT).as_posix():hashlib.sha256(model.read_bytes()).hexdigest()}}
 (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--out',default='build/fixtures/packet-mine-focused');ap.add_argument('--run',action='store_true');a=ap.parse_args();out=ROOT/a.out;prepare(out)
 if not a.run:print(f'PacketMine focused fixture prepared: {out}; no compiler invoked');raise SystemExit()
 print('PacketMine focused --run is delegated to the root unified batch');raise SystemExit(2)
