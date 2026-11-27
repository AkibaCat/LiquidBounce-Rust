#!/usr/bin/env python3
"""Freeze owned ResetVL sources and preserved bounded evidence at handoff."""
from pathlib import Path
import argparse, hashlib, json
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def hashes(paths):return {p.relative_to(ROOT).as_posix():sha(p) for p in sorted(set(paths))}

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='native/liquidbounce-horizon/reset-vl-freeze.json');a=parser.parse_args();out=ROOT/a.out
    if out.exists():raise SystemExit('Refusing to overwrite a freeze manifest')
    owned=[ROOT/'native/liquidbounce-behaviors/src/reset_vl.rs',HERE/'engine/src/config/reset_vl.rs',HERE/'engine/src/reset_vl_runtime.rs',HERE/'reset-vl-host.rs',HERE/'reset-vl-patch.py']
    support=[p for p in HERE.glob('*reset-vl*') if p.is_file() and p not in owned and 'freeze.json' not in p.name]
    support+=[p for p in (HERE/'reset-vl-source-fixture').rglob('*') if p.is_file()]
    selected=['source-03','tests-03','consumption-02'];reports=[];history=[]
    for path in sorted((ROOT/'build').glob('reset-vl-*/report.json')):
        data=json.loads(path.read_text(encoding='utf8'));record={'path':path.relative_to(ROOT).as_posix(),'passed':data.get('passed'),'sha256':sha(path),'scope':data.get('scope')}
        if path.parent.name.removeprefix('reset-vl-')in selected:assert data['passed'];reports.append(record)
        else:history.append(record)
    assert len(reports)==len(selected)
    source=json.loads((ROOT/'build/reset-vl-source-03/report.json').read_text(encoding='utf8'))
    for relative,digest in source['sources'].items():assert sha(ROOT/relative)==digest,relative
    tests=json.loads((ROOT/'build/reset-vl-tests-03/report.json').read_text(encoding='utf8'))
    for relative,digest in tests['sources'].items():assert sha(ROOT/relative)==digest,relative
    consumption=json.loads((ROOT/'build/reset-vl-consumption-02/report.json').read_text(encoding='utf8'))
    for path in owned:
        relative=path.relative_to(ROOT).as_posix()
        if relative in consumption['productionSha256']:assert sha(path)==consumption['productionSha256'][relative]
    manifest={'frozenDate':'2026-11-26','revision':1,'pinnedSources':{'LiquidBounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','Horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','Horizon-Mixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},
        'scope':'Owned ResetVL files frozen before shared registration. Exact original Kotlin/full coroutine/Timer oracle, per-step behavior/Timer/FIFO differential, shared config parser extraction, exact runtime/native helper focused consumers. Explicit providers remain and no actual game proof is claimed.',
        'files':hashes(owned),'supportingFiles':hashes(support),'originalSources':source['sources'],'reports':reports,'preservedHistoricalReports':history,
        'vectors':{'scenarios':tests['scenarios'],'steps':tests['steps'],'oracleSha256':tests['oracleSha256']},
        'sharedFilesAtHandoff':hashes([HERE/'engine/src/timer_runtime.rs',HERE/'engine/src/coroutine_ticker.rs',HERE/'engine/src/config.rs',ROOT/'native/liquidbounce-core/src/module/synchronous.rs']),
        'sharedScope':'Handoff hashes only. Shared behavior/lib/config/actor/runtime and aggregate patch/production staging/package files remain root-owned; none changed by ResetVL work. Locale updater was run on copies only.',
        'fixtureBoundaries':['Original Kotlin source byte-identical. Settings/module/player/events and immediate Main dispatcher are explicit providers; atomicfu API uses actual JDK AtomicReference rather than atomicfu implementation.',
            'Differential uses unchanged shared FIFO body with declaration-only ResetVl WaitOwner variant when absent; config uses actual extracted Tree/parser bodies with pending declaration providers.',
            'Runtime consumption uses exact runtime/Host bodies, actual core registry/TimerService/FIFO/glam with explicit State/Host/Game/registration providers. TimerService PacketListener is excluded and its otherwise-unused Mode import removed only from fixture.',
            'Source03 includes raw denormal/max/infinity/NaN float payload requests as well as defaults/ranges/out-of-range, count boundary/overflow, nested tick/exclusive behavior, lifecycle/settings changes, FIFO external mutations and Timer competition/off-thread queries.'],
        'pendingChecks':['Root shared registration/config actor integration and publishing actual provider.running on every relevant state change.',
            'Root aggregate patch/API generation, complete cumulative concrete Game/engine build and source package validation.',
            'Root TestEngine/actual game verification, including complete-client shared event ordering and Chinese display.']}
    out.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps({'path':out.relative_to(ROOT).as_posix(),'sha256':sha(out),'ownedFiles':len(owned),'supportingFiles':len(support),'scenarios':tests['scenarios'],'steps':tests['steps']}))

if __name__=='__main__':main()
