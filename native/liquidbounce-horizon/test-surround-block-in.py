#!/usr/bin/env python3
from pathlib import Path
import ast,runpy,tempfile,shutil,json,subprocess,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 for n in ['integrate-surround-block-in.py','update-surround-block-in-locales.py',Path(__file__).name]:ast.parse((HERE/n).read_text(encoding='utf8'))
 for n in ['surround.rs','block_in.rs']:
  s=(ROOT/'native/liquidbounce-behaviors/src'/n).read_text(encoding='utf8');assert 'pub struct Settings' in s and '#[cfg(test)]' in s
 p=runpy.run_path(str(HERE/'integrate-surround-block-in.py'))['prepare']()
 merged='\n'.join(str(v) for v in p.changed.values())
 if not p.changed:
  merged='\n'.join((ROOT/'native/liquidbounce-horizon/engine/src'/n).read_text(encoding='utf8') for n in ['behaviors.rs','config.rs','actor.rs'])+'\n'+(ROOT/'native/liquidbounce-behaviors/src/lib.rs').read_text(encoding='utf8')
 for token in ['pub mod surround','pub mod block_in','SurroundSetting','BlockInSetting','ModuleKind::Surround','ModuleKind::BlockIn','RuntimeRequest::Surround','RuntimeRequest::BlockIn']: assert token in merged,token
 config='\n'.join(str(v) for k,v in p.changed.items() if str(k).endswith('config.rs'))
 if not config:
  config=(ROOT/'native/liquidbounce-horizon/engine/src/config.rs').read_text(encoding='utf8')
 assert config.count('pub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};')==1
 with tempfile.TemporaryDirectory(prefix='surround-block-in-loc-') as d:
  out=Path(d);loc=ROOT/'native/liquidbounce-interop/resources/localization';
  for lang in ('zh_cn','zh_tw'):shutil.copyfile(loc/f'display_{lang}.json',out/f'display_{lang}.json')
  runpy.run_path(str(HERE/'update-surround-block-in-locales.py'))['update'](out)
  for lang in ('zh_cn','zh_tw'):
   data=json.loads((out/f'display_{lang}.json').read_text(encoding='utf8'));assert any(k.startswith('liquidbounce.module.surround') for k in data['descriptions']);assert any(k.startswith('liquidbounce.module.blockIn') for k in data['descriptions'])
 print(f'Surround/BlockIn static fixture passed: {len(p.changed)} shared files prepared; no compiler invoked')
if __name__=='__main__':main()
