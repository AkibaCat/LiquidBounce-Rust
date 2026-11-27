from pathlib import Path
import json,hashlib
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
 for module in ['potion_spoof','no_clip','true_sight']:
  slug=module.replace('_','-');target=HERE/f'{slug}-freeze.json'
  if target.exists():raise SystemExit('Refusing to overwrite freeze: '+str(target))
  files=[ROOT/f'native/liquidbounce-behaviors/src/{module}.rs',HERE/f'engine/src/config/{module}.rs',HERE/f'engine/src/{module}_runtime.rs',HERE/f'{slug}-host.rs',HERE/f'{slug}-patch.py',HERE/f'{slug}-smoke.rs',HERE/'test-sight-potion-clip.py',HERE/'update-sight-potion-clip-locales.py',HERE/'sight-potion-clip-integration.md',ROOT/f'build/{slug}-smoke-01/report.json',ROOT/f'build/{slug}-smoke-01/test.log']
  if module=='potion_spoof':files.extend([ROOT/'native/liquidbounce-behaviors/src/potion_spoof_effects.rs',HERE/'generate-potion-spoof.py'])
  if module=='true_sight':files.append(HERE/'true-sight-render.rs')
  report={'module':module,'date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':json.loads((ROOT/f'build/{slug}-smoke-01/report.json').read_text(encoding='utf8')),'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
  assert report['focused']['passed'];target.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(str(target),hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
