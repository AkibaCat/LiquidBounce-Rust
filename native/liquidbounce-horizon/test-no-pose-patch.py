#!/usr/bin/env python3
"""Compile NoPose's full production pose state machine and real generated Host."""
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
UPSTREAM=ROOT/'upstream/Horizon'
OUT=ROOT/'build/no-pose-hook-tests'


def main():
    helper=runpy.run_path(str(HERE/'no-pose-patch.py'))
    block=runpy.run_path(str(HERE/'test-camera-control-patch.py'))['block']
    changed={}
    def edit(path,transform):
        source=UPSTREAM/path
        changed[path]=transform(changed.get(path,source.read_text(encoding='utf-8') if source.exists() else ''))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one NoPose anchor: {before!r}, found {text.count(before)}')
        return text.replace(before,after,1)
    contract=helper['apply'](edit,once)
    OUT.mkdir(parents=True,exist_ok=True)
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    jars=[cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons']:
        jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    oracle=subprocess.run([str(ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'),
        '--class-path',os.pathsep.join(map(str,jars)),str(HERE/'oracles/NoPoseDimensionsFacts.java'),str(OUT/'dimensions-oracle.json')],capture_output=True,text=True)
    (OUT/'java-oracle.log').write_text(oracle.stdout+oracle.stderr,encoding='utf-8')
    if oracle.returncode:raise SystemExit(oracle.stdout+oracle.stderr)
    facts=json.loads((OUT/'dimensions-oracle.json').read_text(encoding='utf-8'))
    assert len(facts['rows'])==18
    assert int(facts['defaultVehicleYBits'])==4603579539098121011
    rows='[\n'+''.join(f"({r['choice']}u8,{r['scale']}f32,{r['widthBits']}u32,{r['heightBits']}u32,{r['eyeBits']}u32,{r['passengerYBits']}u64,{r['vehicleYBits']}u64),\n" for r in facts['rows'])+']'
    (OUT/'pose.rs').write_text(changed['crates/horizon-client/src/pose.rs'],encoding='utf-8')
    (OUT/'no_pose.rs').write_text(helper['policy_source'](),encoding='utf-8')
    (OUT/'behavior.rs').write_text((ROOT/'native/liquidbounce-behaviors/src/no_pose.rs').read_text(encoding='utf-8'),encoding='utf-8')
    def original(path):return (UPSTREAM/'crates/horizon-client/src'/path).read_text(encoding='utf-8')
    guest=original('lan/guest.rs')
    avatar=original('mobs/living.rs')
    start=avatar.index('        let scale = self.model_scale();',avatar.index('pub fn awake_dims_in('))
    end=avatar.index('        let (mut w, mut h, mut e)',start)
    replacements={
        'FLUID_THRESHOLD':block(original('motion/environment.rs'),'pub fn fluid_jump_threshold('),
        'FLUID_EYE_Y':block(original('motion/fluids.rs'),'fn eye_y(').replace('fn eye_y','pub(crate) fn eye_y',1),
        'RIDER_POSES':block(original('game/mounts.rs'),'fn rider_poses('),
        'TRACKING_BODY':block(original('game/render_tick.rs'),'pub(super) fn player_tracking_body('),
        'GUEST_POSE':block(guest,'pub fn pose('),
        'GUEST_DIMENSIONS':block(guest,'pub fn dimensions('),
        'GUEST_BOUNDS':block(guest,'pub fn bounds('),
        'GUEST_EYE':block(guest,'pub fn eye_height('),
        'AVATAR_DIMENSION_BRANCH':avatar[start:end],
        'PASSENGER_ATTACHMENT':block(changed['crates/horizon-client/src/mobs/riders.rs'],'pub fn passenger_attachment('),
        'VEHICLE_ATTACHMENT':block(changed['crates/horizon-client/src/mobs/riders.rs'],'pub fn vehicle_attachment('),
        'GENERIC_PLAYER_SEAT':block(original('game/remote_vehicles.rs'),'fn generic_player_seat('),
        'NATIVE_HOOKS':helper['HOOKS'].replace('pub(super)','pub(crate)'),
        'ORACLE_ROWS':rows,
    }
    source=(HERE/'no-pose-fixture.rs').read_text(encoding='utf-8')
    for name,value in replacements.items():source=source.replace('__'+name+'__',value)
    (OUT/'lib.rs').write_text(source,encoding='utf-8')

    # Check unchanged consumers still take the central cached dimensions or pose.
    guards={
        'local_camera':('game/perspective.rs','eye_height: p.dimensions.eye_height'),
        'local_model':('game/perspective.rs','m.extra.avatar_pose = pose;'),
        'local_render_light':('game/perspective.rs','|p| f64::from(p.dimensions.eye_height)'),
        'visual_swimming':('game/perspective.rs','p.pose == Pose::Swimming || (!p.fall_flying && p.pose == Pose::FallFlying)'),
        'first_person_hands_camera':('game.rs','self.hand_geometry(camera_alpha)'),
        'liquid_eye_y':('motion/fluids.rs','self.player.position.y + f64::from(self.player.dimensions.eye_height)'),
        'travel_box':('motion.rs','self.player.bounds()'),
        'server_anticheat':('lan/hub.rs','let d = g.dimensions();'),
        'remote_avatar_dimensions':('mobs/living.rs','"minecraft:player" | "minecraft:mannequin"'),
        'guest_cache':('lan/guest.rs','Some(m) if m.pose == pose => m.dimensions'),
    }
    for name,(path,needle) in guards.items():assert needle in original(path),(name,path,needle)
    passengers=changed['crates/horizon-client/src/game/passengers.rs']
    assert passengers.count('p.passenger_attachment_y()')==3
    assert 'f64::from(p.dimensions.height)' not in passengers
    for anchor in ['    fn tick(&mut self) -> Result<()> {\n','    fn render_frame(&mut self) -> Result<bool> {\n']:
        assert anchor+'        self.sync_native_no_pose();' in changed['crates/horizon-client/src/game.rs']

    api=OUT/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{"+helper['HOOKS']+'}',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[],'hooks':[{'name':name,'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name,'mod_api':{'contract':'listen'}} for name in helper['HOOK_NAMES']]},indent=2)+'\n',encoding='utf-8')
    (OUT/'build.rs').write_text('''fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}''',encoding='utf-8')
    (OUT/'Cargo.toml').write_text('''[package]
name="no-pose-hook-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[dependencies]
horizon-core={path="'''+(UPSTREAM/'crates/horizon-core').as_posix()+'''"}
horizon-world={path="'''+(UPSTREAM/'crates/horizon-world').as_posix()+'''"}
glam="0.30"
[build-dependencies]
horizon-mod-api-gen={path="'''+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n',encoding='utf-8')
    command=['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--','--test-threads=1']
    result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
    (OUT/'test.log').write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr)
    lint=subprocess.run(['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings'],cwd=ROOT,capture_output=True,text=True) if result.returncode==0 else None
    if lint is not None:
        (OUT/'clippy.log').write_text(lint.stdout+lint.stderr,encoding='utf-8');print(lint.stdout+lint.stderr)
    report={'hookContract':contract,'sourceFiles':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in sorted(changed.items())},
        'testedProductionBodies':['complete pose.rs state machine']+list(replacements),'consumerSourceGuards':guards,
        'apiGenerator':'horizon_mod_api_gen::generate(Target::Client); actual generated Host and hook dispatch compiled',
        'javaOracle':'actual MC26.3 EntityDimensions/EntityAttachments with source-derived NoPose dimension constructor; 18 combinations',
        'boundaries':['world collision fixture with real Horizon Aabb intersections','game/player/guest/mob shells','render wiring source assertions; no GPU or first-person image comparison'],
        'fixtureResult':'passed' if result.returncode==0 else 'failed',
        'clippyResult':'passed' if lint is not None and lint.returncode==0 else 'failed',
        'fullGameIntegrationPassed':False}
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode or (lint.returncode if lint is not None else 0))


if __name__=='__main__':main()
