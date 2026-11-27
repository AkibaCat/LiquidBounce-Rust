#!/usr/bin/env python3
"""One focused SpearKill smoke; optional native patch composition is memory-only."""
from pathlib import Path
import subprocess,tempfile,importlib.util
HERE=Path(__file__).resolve().parent
def main():
    with tempfile.TemporaryDirectory(prefix='liquidbounce-spear-') as temp:
        exe=Path(temp)/'spear-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test',str(HERE/'spear-kill-tests.rs'),'-o',str(exe)],check=True)
        subprocess.run([str(exe),'--nocapture'],check=True)
    spec=importlib.util.spec_from_file_location('spear_patch',HERE/'spear-kill-patch.py')
    mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
    text={};root=HERE.parents[1]/'build/staging/horizon'
    def once(s,a,b):
        if s.count(a)!=1:raise ValueError(f'Expected unique anchor: {a[:80]} ({s.count(a)})')
        return s.replace(a,b)
    def edit(name,change):
        source=text.get(name)
        if source is None:source=(root/name).read_text(encoding='utf8') if (root/name).is_file() else ''
        text[name]=change(source)
    result=mod.apply(edit,once)
    assert len(result['helper_names'])==8
    print(f'SpearKill native patch composed in memory: {len(text)} files; staging unchanged')
if __name__=='__main__':main()
