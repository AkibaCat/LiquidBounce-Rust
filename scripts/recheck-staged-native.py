"""Recover a failed Cargo check against an unchanged certified staging snapshot."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,runpy
ROOT=Path(__file__).resolve().parent.parent
ap=argparse.ArgumentParser();ap.add_argument('--batch',required=True);ap.add_argument('--attempt',required=True);ap.add_argument('--stage',default='horizon3');a=ap.parse_args()
batch=ROOT/'build/batches'/a.batch
stage=ROOT/'build/staging'/a.stage
previous=json.loads((batch/'report.json').read_text(encoding='utf8'))
manifest=json.loads((stage/'.liquidbounce-staging.json').read_text(encoding='utf8'))
for key,value in previous['source'].items():
 if manifest[key]!=value:raise SystemExit(f'Staging differs from the failed batch: {key}')
log=batch/f'{a.attempt}.log';result=batch/f'{a.attempt}.json'
if log.exists()or result.exists():raise SystemExit('Recovery evidence already exists')
run=runpy.run_path(str(ROOT/'scripts/check-native-batch.py'))['run_step']
report=run(['cargo','+stable','check','-p','liquidbounce-native-game','--offline'],stage,log)
if report['passed']and'Finished'not in log.read_text(encoding='utf8',errors='replace'):
 report['passed']=False;report['error']='Cargo did not emit completion evidence';report['exitCode']=1
report.update(source=previous['source'],modules=previous['modules'],gameValidation='not_run',upstreamDifferentialValidation='not_run')
result.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
raise SystemExit(report['exitCode'])
