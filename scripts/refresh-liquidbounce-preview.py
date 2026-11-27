#!/usr/bin/env python3
"""Review or apply a verified game build to an existing local preview in place.

Default: write a plan outside the package. --apply performs only listed writes,
with backups outside the package and rollback if any write fails. user-data is
never traversed, read, renamed, copied, overwritten, or removed.

--acceptance must name JSON containing:
  validationScope: "startup_menu_world_smoke", passed: true,
  executableSha256: <successful build hash>,
  checks: {menu: {passed: true}, world: {passed: true}},
  evidenceFiles: [<project-relative paths to isolated QA evidence>].
This scoped smoke validation does not claim acceptance of all 236 modules.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import tempfile

PROJECT=Path(__file__).resolve().parent.parent
HORIZON='9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'
SHA=re.compile(r'[0-9a-f]{64}')


def now():
    return datetime.now(timezone.utc).isoformat()


def no_links(path):
    for item in (path,*path.parents):
        if item.is_symlink() or (item.exists() and getattr(item.lstat(),'st_file_attributes',0)&0x400):
            raise ValueError(f'Reparse point is not accepted: {item}')


def local(root,name):
    if not isinstance(name,str) or not name or name != PurePosixPath(name).as_posix():
        raise ValueError(f'Unsafe relative path: {name!r}')
    parts=name.split('/')
    if ('\\' in name or any(p in ('','.','..') or p.endswith((' ','.')) for p in parts)
        or any(ord(c)<32 or c in ':*?<>|"' for c in name) or parts[0].casefold()=='user-data'):
        raise ValueError(f'Unsafe relative path: {name}')
    result=root.joinpath(*parts)
    no_links(result)
    result.resolve().relative_to(root.resolve())
    return result


def digest(path):
    value=hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(1024*1024),b''):
            value.update(block)
    return value.hexdigest()


def data_json(value):
    return (json.dumps(value,indent=2,ensure_ascii=False)+'\n').encode('utf-8')


def read_json(path):
    def unique(pairs):
        result={}
        for key,value in pairs:
            if key in result:
                raise ValueError(f'Duplicate JSON key in {path}: {key}')
            result[key]=value
        return result
    return json.loads(path.read_text(encoding='utf-8-sig'),object_pairs_hook=unique)


def package_files(root):
    # Exclude mutable data before inspecting any entry or reparse attributes.
    for directory,dirs,files in os.walk(root,followlinks=False):
        base=Path(directory)
        dirs[:]=[name for name in dirs if not (base==root and name.casefold()=='user-data')]
        for name in dirs:
            no_links(base/name)
        for name in files:
            path=base/name
            no_links(path)
            yield path


def main():
    parser=argparse.ArgumentParser(description=__doc__,formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--package',type=Path,required=True)
    parser.add_argument('--build-report',type=Path,default=PROJECT/'build/integrated-build.json')
    parser.add_argument('--acceptance',type=Path,required=True)
    parser.add_argument('--plan',type=Path,default=PROJECT/'build/launcher-qa-20261127/release-refresh-plan.json')
    parser.add_argument('--apply',action='store_true')
    args=parser.parse_args()
    root=args.package.absolute()
    no_links(root)
    root.resolve().relative_to((PROJECT.parent/'ReleasesPack').resolve())
    if root.resolve()==(PROJECT.parent/'ReleasesPack').resolve():
        raise ValueError('Select the existing release subfolder, not ReleasesPack itself.')
    plan_path=args.plan.absolute()
    no_links(plan_path)
    plan_path.resolve().relative_to((PROJECT/'build').resolve())
    if root.resolve() in plan_path.resolve().parents:
        raise ValueError('The review plan must be outside the package.')
    manifest_path=local(root,'package-manifest.json')
    manifest=read_json(manifest_path)
    if manifest.get('completePort') is not False or not isinstance(manifest.get('implementedModules'),list):
        raise ValueError('Expected an existing incomplete development preview manifest.')
    build_path=args.build_report.resolve()
    acceptance_path=args.acceptance.resolve()
    no_links(build_path)
    no_links(acceptance_path)
    build=read_json(build_path)
    if build.get('operation')!='build' or build.get('passed') is not True or build.get('exitCode')!=0:
        raise ValueError('A successful integrated game build report is required.')
    stage=build['staging']
    if stage.get('horizon')!=HORIZON or stage.get('originalThemeLinked') is not True:
        raise ValueError('The build must use pinned Horizon and the linked original theme.')
    game=Path(build['executable']).resolve()
    expected_game=PROJECT/'build/staging/horizon/target/debug/liquidbounce-native-game.exe'
    if game!=expected_game.resolve() or digest(game)!=build['executableSha256']:
        raise ValueError('The staging executable differs from the successful build report.')
    source_map=stage['nativeSourceFiles']
    if not isinstance(source_map,dict) or not source_map:
        raise ValueError('The complete compiled native source map is required.')
    aggregate=hashlib.sha256(json.dumps(source_map,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    if aggregate!=stage['nativeSourceSha256']:
        raise ValueError('The compiled native source aggregate does not match.')
    for name,sha in source_map.items():
        if not name.startswith(('native/','patches/','scripts/')) or not SHA.fullmatch(sha):
            raise ValueError(f'Invalid compiled native source entry: {name}')
        if digest(local(PROJECT,name))!=sha:
            raise ValueError(f'Source changed since compilation: {name}')
    lock=PROJECT/'build/staging/horizon/Cargo.lock'
    if digest(lock)!=build['lockSha256']:
        raise ValueError('Staging Cargo.lock changed since compilation.')
    acceptance=read_json(acceptance_path)
    checks=acceptance.get('checks',{})
    passed=lambda item: item is True or (isinstance(item,dict) and item.get('passed') is True)
    if (acceptance.get('validationScope')!='startup_menu_world_smoke' or acceptance.get('passed') is not True
        or acceptance.get('executableSha256')!=build['executableSha256']
        or not passed(checks.get('menu')) or not passed(checks.get('world'))):
        raise ValueError('Require passed menu+world scoped smoke validation for this exact game hash.')
    evidence=acceptance.get('evidenceFiles')
    if not isinstance(evidence,list) or not evidence:
        raise ValueError('Acceptance must list isolated QA evidenceFiles relative to the project.')
    inputs={}
    def add_source(name,source,expected=None):
        destination=local(root,name)
        source=source.resolve()
        no_links(source)
        if source.resolve()==destination.resolve():
            raise ValueError(f'Input must be outside the package: {source}')
        sha=digest(source)
        if expected is not None and sha!=expected:
            raise ValueError(f'Input changed: {source}')
        inputs[name]={'source':str(source),'sha256':sha,'bytes':source.stat().st_size}
    def add_data(name,value):
        local(root,name)
        data=data_json(value)
        inputs[name]={'data':data,'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
    add_source('runtime/liquidbounce-native-game.exe',game,build['executableSha256'])
    for name,sha in source_map.items():
        add_source('source/'+name,local(PROJECT,name),sha)
    add_source('source/rebuild-locks/horizon-Cargo.lock',lock,build['lockSha256'])
    add_source('source/rebuild-locks/horizon-Cargo.toml',PROJECT/'build/staging/horizon/Cargo.toml')
    add_source('source/scripts/refresh-liquidbounce-preview.py',Path(__file__))
    add_data('source/native-source-manifest.json',{'nativeSourceSha256':aggregate,'nativeSourceFiles':source_map})
    summary={key:build[key] for key in ('operation','passed','exitCode','startedUtc','finishedUtc','executableSha256','lockSha256')}
    summary.update({key:stage[key] for key in ('horizon','originalThemeLinked','gpuUiPatchSha256','behaviorHookPatchSha256','nativeSourceSha256')})
    summary.update(nativeSourceFiles=source_map,implementedModules=manifest['implementedModules'])
    add_data('metadata/build-summary.json',summary)
    add_source('metadata/game-startup-validation.json',acceptance_path)
    add_source('metadata/game-build-report.json',build_path)
    evidence_entries=[]
    for name in evidence:
        if not isinstance(name,str) or not name.startswith('build/launcher-qa-20261127/') or '/user-data/' in name.casefold():
            raise ValueError(f'Only explicitly listed isolated QA evidence is accepted: {name!r}')
        source=local(PROJECT,name)
        destination='metadata/game-startup-evidence/'+source.relative_to(PROJECT/'build/launcher-qa-20261127').as_posix()
        if destination in inputs:
            raise ValueError(f'Duplicate evidence path: {name}')
        add_source(destination,source)
        evidence_entries.append({'path':destination,'sha256':inputs[destination]['sha256']})
    add_data('metadata/game-startup-evidence/index.json',{'validationScope':'startup_menu_world_smoke',
        'executableSha256':build['executableSha256'],'completeModuleAcceptance':False,'files':evidence_entries})
    # Do not silently retain a full-acceptance report about a different executable.
    old_validation=local(root,'metadata/validation-summary.json')
    if old_validation.exists() and read_json(old_validation).get('executableSha256')!=build['executableSha256']:
        raise ValueError('Existing validation-summary.json belongs to another executable; archive it explicitly first.')
    changed=[]
    for name,entry in inputs.items():
        target=local(root,name)
        if not target.is_file() or target.stat().st_size!=entry['bytes'] or digest(target)!=entry['sha256']:
            changed.append(name)
    plan={'createdUtc':now(),'package':str(root),'applyRequested':args.apply,
        'oldGameSha256':digest(local(root,'runtime/liquidbounce-native-game.exe')),
        'newGameSha256':build['executableSha256'],'oldNativeSourceSha256':manifest['nativeSourceSha256'],
        'newNativeSourceSha256':aggregate,'compiledSourceFiles':len(source_map),
        'changedFiles':[{'path':name,'bytes':inputs[name]['bytes'],'sha256':inputs[name]['sha256'],
            'source':inputs[name].get('source','generated metadata')} for name in changed],
        'userDataPolicy':'Never enumerated, read, moved, overwritten, copied, or removed.',
        'validationScope':'startup_menu_world_smoke','completePort':False}
    plan_path.parent.mkdir(parents=True,exist_ok=True)
    plan_path.write_bytes(data_json(plan))
    if not args.apply:
        print(json.dumps({'plan':str(plan_path),'changedFiles':len(changed),'applied':False}))
        return
    # Keep recovery copies outside ReleasesPack. Never back up mutable user-data.
    backup=Path(tempfile.mkdtemp(prefix='release-refresh-backup-',dir=plan_path.parent))
    written=[]
    def replace(name,entry):
        target=local(root,name)
        target.parent.mkdir(parents=True,exist_ok=True)
        with tempfile.NamedTemporaryFile(prefix='.release-refresh-',suffix='.tmp',dir=target.parent,delete=False) as stream:
            temporary=Path(stream.name)
            if 'data' in entry:
                stream.write(entry['data'])
            else:
                with Path(entry['source']).open('rb') as source:
                    shutil.copyfileobj(source,stream)
        try:
            if digest(temporary)!=entry['sha256']:
                raise ValueError(f'Copy verification failed: {name}')
            os.replace(temporary,target)
        finally:
            temporary.unlink(missing_ok=True)
    try:
        # Replace the executable first: Windows refuses if an existing game is
        # using it, avoiding metadata/source updates for an unavailable runtime.
        for name in changed+['package-manifest.json']:
            target=local(root,name)
            saved=local(backup,name)
            if target.exists():
                saved.parent.mkdir(parents=True,exist_ok=True)
                shutil.copy2(target,saved)
        for name in changed:
            written.append(name)
            replace(name,inputs[name])
        manifest.update(updatedUtc=now(),nativeSourceSha256=aggregate,gameStartupValidation={
            'validationScope':'startup_menu_world_smoke','passed':True,
            'executableSha256':build['executableSha256'],'report':'metadata/game-startup-validation.json',
            'completeModuleAcceptance':False})
        records=[]
        for path in package_files(root):
            if path==manifest_path:
                continue
            records.append({'path':path.relative_to(root).as_posix(),'bytes':path.stat().st_size,'sha256':digest(path)})
        manifest['files']=sorted(records,key=lambda entry:entry['path'])
        payload=data_json(manifest)
        written.append('package-manifest.json')
        replace('package-manifest.json',{'data':payload,'sha256':hashlib.sha256(payload).hexdigest()})
        result={'applied':True,'package':str(root),'changedFiles':len(changed),'manifestFiles':len(records),
            'gameSha256':build['executableSha256'],'nativeSourceSha256':aggregate,
            'backup':str(backup),'userDataTouched':False}
        (backup/'refresh-result.json').write_bytes(data_json(result))
        print(json.dumps(result))
    except BaseException:
        for name in reversed(written):
            saved=local(backup,name)
            target=local(root,name)
            if saved.exists():
                shutil.copy2(saved,target)
            else:
                target.unlink(missing_ok=True)
        raise


if __name__=='__main__':
    main()
