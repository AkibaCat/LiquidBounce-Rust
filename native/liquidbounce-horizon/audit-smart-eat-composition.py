#!/usr/bin/env python3
"""Compose after the complete current production transforms without publishing.

Only build/smart-eat-tests/composition is written; upstream and shared generated
patches are untouched. Rustfmt checks syntax of every transformed Rust source.
"""
import hashlib
import json
from pathlib import Path
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/smart-eat-tests'

def main():
    generator=HERE/'make-behavior-patch.py'
    source=generator.read_text(encoding='utf-8')
    marker='\noutput = []\n'
    if source.count(marker)!=1:
        raise ValueError('Expected one production patch serialization boundary')
    # Evaluate only the in-memory transforms, before any destination mkdir/write.
    scope={'__file__':str(generator),'__name__':'smart_eat_composition_audit'}
    exec(compile(source.split(marker,1)[0],str(generator),'exec'),scope)
    helper=runpy.run_path(str(HERE/'smart-eat-patch.py'))
    if 'client_smart_eat_wait' in scope['names']:
        contract={'mode':'already_in_production','hook_names':helper['HOOK_NAMES'],'helper_names':helper['HELPER_NAMES']}
    else:
        contract=helper['apply'](scope['edit'],scope['replace_once'])
    native=scope['changes']['crates/horizon-client/src/game/native_hooks.rs'][1]
    block=runpy.run_path(str(HERE/'test-attack-patch.py'))['block']
    pre=block(native,'    pub(super) fn native_client_tick_pre(')
    assert pre.index('self.native_client_smart_eat_wait();')<pre.index('slot_bridge::tick()')<pre.index('dispatch_client_tick_pre(')
    hand=scope['changes']['crates/horizon-client/src/game/hand.rs'][1]
    assert hand.count('if self.client_use_key_down() && self.use_delay == 0 && !was_using')==1
    work=OUT/'composition'
    work.mkdir(parents=True,exist_ok=True)
    rust=[]
    hashes={}
    for name,(_,text) in scope['changes'].items():
        if not name.endswith('.rs'):continue
        dest=work/name
        dest.parent.mkdir(parents=True,exist_ok=True)
        dest.write_text(text,encoding='utf-8')
        rust.append(str(dest))
        hashes[name]=hashlib.sha256(text.encode()).hexdigest()
    check=subprocess.run(['rustfmt','+stable','--edition','2024','--config','skip_children=true','--emit','files',*rust],cwd=ROOT,capture_output=True,text=True)
    (OUT/'full-composition-syntax.log').write_text(check.stdout+check.stderr,encoding='utf-8')
    evidence={'exitCode':check.returncode,'productionGeneratorSha256':hashlib.sha256(source.encode()).hexdigest(),
        'contract':contract,'transformedRustSources':hashes,'waiterOrder':'SmartEat;SilentHotbar;GameTick',
        'scope':'all_current_production_transforms_then_smart_eat;rustfmt_syntax_only;no_native_typecheck;full_client_differential_not_run'}
    (OUT/'full-composition.json').write_text(json.dumps(evidence,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'exitCode':check.returncode,'transformedRustSources':len(rust),'waiterOrder':evidence['waiterOrder']}))
    if check.returncode:print(check.stdout+check.stderr)
    raise SystemExit(check.returncode)

if __name__=='__main__':main()
