#!/usr/bin/env python3
"""One focused Criticals smoke, plus native extension anchors in memory."""
from pathlib import Path
import runpy,subprocess,tempfile
HERE=Path(__file__).resolve().parent
def main():
 with tempfile.TemporaryDirectory(prefix='liquidbounce-criticals-')as temp:
  exe=Path(temp)/'criticals-smoke.exe';subprocess.run(['rustc','--edition=2024','--test',str(HERE/'criticals-smoke.rs'),'-o',str(exe)],check=True);subprocess.run([str(exe),'--nocapture'],check=True)
 staged=HERE.parents[1]/'build/staging/horizon';texts={}
 def edit(name,f):
  if name not in texts:texts[name]=(staged/name).read_text(encoding='utf8').replace('\r\n','\n')if(staged/name).exists()else''
  texts[name]=f(texts[name])
 def once(s,old,new):
  if s.count(old)!=1:raise RuntimeError(f'Criticals native anchor count {s.count(old)}: {old[:100]}')
  return s.replace(old,new)
 runpy.run_path(str(HERE/'criticals-patch.py'))['apply'](edit,once)
 print('Criticals native extension validated in memory:',len(texts),'files; no staging writes')
if __name__=='__main__':main()
