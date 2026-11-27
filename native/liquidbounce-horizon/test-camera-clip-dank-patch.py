#!/usr/bin/env python3
"""Production Camera/singleton/bob consumer + actual generated API + MC JAR oracle."""
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
UPSTREAM=ROOT/'upstream/Horizon'
OUT=ROOT/'build/camera-clip-dank-hook-tests'


def main():
    old=runpy.run_path(str(HERE/'camera-control-patch.py'))
    clip=runpy.run_path(str(HERE/'camera-clip-patch.py'))
    dank=runpy.run_path(str(HERE/'dank-bobbing-patch.py'))
    block=runpy.run_path(str(HERE/'test-camera-control-patch.py'))['block']
    changed={}
    def edit(path,fn):
        original=UPSTREAM/path
        changed[path]=fn(changed.get(path,original.read_text(encoding='utf-8') if original.exists() else ''))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one anchor, found {text.count(before)}: {before!r}')
        return text.replace(before,after,1)
    # Import only the existing fov() transform and its hook literal. Executing
    # the main generator itself would mutate the parent's shipping patch.
    tree=ast.parse((HERE/'make-behavior-patch.py').read_text(encoding='utf-8'))
    fov_node=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='fov')
    original_hooks=next(ast.literal_eval(node.value) for node in tree.body if isinstance(node,ast.Assign)
        and any(isinstance(t,ast.Name) and t.id=='hooks' for t in node.targets))
    namespace={'replace_once':once}
    exec(compile(ast.Module(body=[fov_node],type_ignores=[]),'<actual-main-fov-transform>','exec'),namespace)
    edit('crates/horizon-client/src/game/fov.rs',namespace['fov'])
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        original_hooks+'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    contracts=[old['apply'](edit,once),clip['apply'](edit,once),dank['apply'](edit,once)]
    OUT.mkdir(parents=True,exist_ok=True)
    camera=changed['crates/horizon-client-render/src/camera.rs'].split('\n#[cfg(test)]\nmod tests {',1)[0]
    (OUT/'camera.rs').write_text(camera,encoding='utf-8')
    (OUT/'camera_control.rs').write_text(old['policy_source'](),encoding='utf-8')
    for name,source in [('easing','easing'),('numeric','numeric'),('camera_clip','camera_clip'),('camera_clip_keys','camera_clip_keys'),
        ('dank_bobbing','dank_bobbing'),('camera_behavior','camera_control'),('auto_f5','auto_f5')]:
        (OUT/f'{name}.rs').write_text((ROOT/f'native/liquidbounce-behaviors/src/{source}.rs').read_text(encoding='utf-8'),encoding='utf-8')
    (OUT/'vanilla_key_names.rs').write_text((UPSTREAM/'crates/horizon-client-input/src/key_names.rs').read_text(encoding='utf-8'),encoding='utf-8')

    # Full InputConstants exported tables plus numerical/canonical/error cases.
    key_source=(ROOT/'native/liquidbounce-behaviors/src/camera_clip_keys.rs').read_text(encoding='utf-8')
    key_names=re.findall(r'\(\d+, "([^"]+)"\)',key_source)
    key_names+=['key.keyboard.0','key.keyboard.224','key.keyboardx004','key.keyboard.-4','key.keyboard.+004',
        'key.keyboard.2147483647','key.keyboard.2147483648','key.mouse.1','key.mouse.0','key.mouse.99','none','KEY.KEYBOARD.A','key.keyboard.left.ctrl']
    (OUT/'key-inputs.txt').write_text('\n'.join(key_names)+'\n',encoding='utf-8')
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    jars=[cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result=subprocess.run([str(java),'-Duser.language=en','-Duser.country=US','-Dfile.encoding=UTF-8',
        '--class-path',os.pathsep.join(map(str,jars)),str(HERE/'oracles/CameraClipDankFacts.java'),
        str(OUT/'camera-dank-oracle.json'),str(OUT/'key-inputs.txt')],capture_output=True,text=True,errors='replace')
    (OUT/'java-oracle.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    if result.returncode:raise SystemExit(result.stdout+result.stderr)
    oracle=json.loads((OUT/'camera-dank-oracle.json').read_text(encoding='utf-8'))
    assert oracle['leftControl']==224 and oracle['unknown']==0
    rust=lambda value:json.dumps(value,ensure_ascii=False)
    assertions=''
    for index,row in enumerate(oracle['keys']):
        call=f'camera_clip::ModifierKey::from_name({rust(row["input"])})'
        if 'error' in row:assertions+=f'assert!({call}.is_none(),"key case {index}");\n'
        else:
            assertions+=f'{{let key={call}.unwrap();assert_eq!((key.name.as_str(),key.value,key.keyboard),({rust(row["name"])},{row["value"]},{str(row["keyboard"]).lower()}),"key case {index}");}}\n'
    for index,row in enumerate(oracle['animations']):
        assertions+=f'''{{let c=camera_clip::CameraClip{{camera_distance:14.5,scroll_enabled:false,animation_speed:{row['speed']}f32,animation_easing:Easing::ALL[{row['choice']}],..Default::default()}};
            let(mut last,mut distance,mut no_clip)=(f32::from_bits({row['lastBits']}),4.,false);
            c.perspective(true,1,1,&mut last,&mut distance,&mut no_clip);
            assert_eq!(distance.to_bits(),{row['resultBits']},"animation oracle {index}");}}\n'''
    for index,row in enumerate(oracle['matrices']):
        living='Some(camera::HurtBob{hurt_time:3.5,hurt_duration:10,hurt_dir:32.5,death_time:None})' if row['hurt'] else 'None'
        assertions+=f'''{{let state=camera::BobState{{living:{living},player:Some(({row['walk']}f32,{row['bob']}f32))}};
            let actual=camera::bob_matrix_dank(&state,0.73,true,Some({row['motion']}f32)).m;
            assert_eq!(actual.map(f32::to_bits),{row['matrixBits']},"Dank oracle {index}");}}\n'''
    native=changed['crates/horizon-client/src/game/native_hooks.rs']
    anchors=['sync_native_camera_aspect','native_client_camera_aspect','native_raw_camera_perspective',
        'native_client_camera_position','native_client_camera_perspective','effective_camera_type']
    methods='\n'.join(block(native,'pub(super) fn '+name+'(') for name in anchors)+ '\n'+clip['HOOKS']+dank['HOOKS']
    methods+='\n'+'\n'.join(block(original_hooks,'pub(super) fn '+name+'(') for name in ['native_camera_bob_effects','native_camera_bob_result'])
    hooks=[name for c in contracts for name in c['hook_names']]+['camera_bob_effects','camera_bob_result']
    helpers=[name for c in contracts for name in c['helper_names']]
    api=OUT/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{"+methods+'}',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[{'name':name,'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name} for name in helpers],
        'hooks':[{'name':name,'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name,'mod_api':{'contract':'listen'}} for name in hooks]},indent=2)+'\n',encoding='utf-8')
    remote=changed['crates/horizon-client/src/game/remote_camera.rs']
    source=(HERE/'camera-clip-dank-fixture.rs').read_text(encoding='utf-8')
    replacements={'TICK':clip['TICK'],'NATIVE':methods,'PLAYER_BOB':dank['PLAYER_BOB'],
        'BOB_MATRIX':block(changed['crates/horizon-client/src/game/fov.rs'],'pub(super) fn bob_matrix('),
        'REMOTE_CAMERA':'\n'.join(block(remote,'pub(super) fn '+name+'(') for name in ['mob_camera_entity','stand_camera_entity']),
        'CAMERA_ENTITY':block(remote,'pub(super) fn camera_entity('),'ORACLE_ASSERTIONS':assertions}
    for name,value in replacements.items():source=source.replace('__'+name+'__',value)
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    (OUT/'build.rs').write_text('''fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}''',encoding='utf-8')
    (OUT/'Cargo.toml').write_text('''[package]
name="camera-clip-dank-hook-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[dependencies]
glam="0.30"
horizon-core={path="'''+(UPSTREAM/'crates/horizon-core').as_posix()+'''"}
[build-dependencies]
horizon-mod-api-gen={path="'''+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n',encoding='utf-8')
    guards={
        'tick_head': 'pub(super) fn tick_fov(&mut self) {\n        self.tick_native_camera_perspective();',
        'shared_bob_no_hurt': 'filtered.living = None;',
        'view_cancellation_precedes_dank': 'self.native_camera_bob_effects(&mut hurt_enabled, &mut view_enabled);',
        'native_shake_preserved':'let combined = glam::Mat4::from_cols_array(&applied) * shake;',
    }
    fov=changed['crates/horizon-client/src/game/fov.rs']
    for name,needle in guards.items():assert needle in fov,(name,needle)
    raw=changed['crates/horizon-client/src/game.rs']
    assert raw.index('self.native_client_mouse_scroll_raw(raw_y);')<raw.index('let discrete = self.settings.discrete_scroll;')
    assert 'let view = bob * spin * horizon_client::camera::view_matrix(yaw, pitch);' in raw
    assert 'let view = bob * horizon_client::camera::view_matrix(yaw, pitch);' in raw
    assert 'self.native_client_camera_perspective(' not in block(native,'pub(super) fn effective_camera_type(')
    result=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,capture_output=True,text=True)
    (OUT/'test.log').write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr)
    lint=subprocess.run(['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings'],cwd=ROOT,capture_output=True,text=True) if result.returncode==0 else None
    if lint is not None:(OUT/'clippy.log').write_text(lint.stdout+lint.stderr,encoding='utf-8');print(lint.stdout+lint.stderr)
    report={'hookContracts':contracts,'sourceFiles':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in sorted(changed.items())},
        'testedProductionBodies':['complete Camera module','tick_native_camera_perspective','effective_camera_type','all native camera/raw-wheel hooks',
            'actual Game::camera_entity with living mounts','mob_camera_entity','stand_camera_entity','actual Game::camera_player_bob','actual Game::bob_matrix'],
        'apiGenerator':'horizon_mod_api_gen::generate(Target::Client), full Host/events/dispatch generated and compiled',
        'javaOracle':{'scope':'actual MC26.3 Mth/InputConstants/PoseStack + source-derived complete CameraClip easing and Dank formula',
            'keyRows':len(oracle['keys']),'animationRows':len(oracle['animations']),'matrixRows':len(oracle['matrices'])},
        'sourceGuards':guards,'fixtureResult':'passed' if result.returncode==0 else 'failed',
        'clippyResult':'passed' if lint is not None and lint.returncode==0 else 'failed',
        'boundaries':['game/window/screen state shell','Winit physical key-name mapping shell','engine module registry/setting persistence is parent integration'],
        'fullGameIntegrationPassed':False}
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode or (lint.returncode if lint is not None else 0))

if __name__=='__main__':main()
