#!/usr/bin/env python3
"""Compile transformed production camera/lens bodies and check Java arithmetic.

The complete production camera module and real core math are compiled. The
renderer/window shell and native dispatch are fixture boundaries; full Horizon
compilation and in-game verification are separate checks.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
UPSTREAM = ROOT / 'upstream/Horizon'
OUT = ROOT / 'build/camera-control-hook-tests'
JAVA = Path('D:/Develop/MCMOD/sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin')


def block(text, anchor):
    start = text.index(anchor)
    opening = text.index('{', start)
    depth, end = 1, opening + 1
    while depth:
        if text[end] == '{': depth += 1
        elif text[end] == '}': depth -= 1
        end += 1
    return text[start:end]


def main():
    changes = {}
    def edit(path, transform):
        source = UPSTREAM / path
        changes[path] = transform(changes.get(path, source.read_text(encoding='utf-8') if source.exists() else ''))
    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'Expected one camera anchor: {before[:100]!r}, got {text.count(before)}')
        return text.replace(before, after, 1)
    spec = importlib.util.spec_from_file_location('camera_control_patch', HERE / 'camera-control-patch.py')
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    contract = helper.apply(edit, once)
    OUT.mkdir(parents=True, exist_ok=True)
    production = changes['crates/horizon-client-render/src/camera.rs'].split('\n#[cfg(test)]\nmod tests {', 1)[0]
    (OUT / 'camera.rs').write_text(production, encoding='utf-8')
    (OUT / 'camera_control.rs').write_text(helper.policy_source(), encoding='utf-8')
    (OUT / 'behavior.rs').write_text((ROOT / 'native/liquidbounce-behaviors/src/camera_control.rs').read_text(encoding='utf-8'), encoding='utf-8')

    # Each row: disabled callback, running, FreeLook, raw perspective, first POV,
    # reset on perspective change, horizontal/vertical factors, original position.
    actions = [
        (False,False,False,1,False,True,.9,.93,[10.,64.,10.]),
        (False,True,False,1,False,True,.9,.93,[10.,64.,10.]),
        (False,True,False,1,False,True,.9,.93,[20.,68.,5.]),
        (False,True,False,2,False,True,.9,.93,[30.,72.,0.]),
        (False,True,False,0,False,True,.9,.93,[31.,70.,1.]),
        (False,True,False,0,True,True,.9,.93,[35.,80.,-5.]),
        (False,True,True,1,True,True,.9,.93,[0.,0.,0.]),
        (False,True,False,1,True,True,.9,.93,[38.,82.,-10.]),
        (True,False,False,1,True,True,.9,.93,[40.,86.,-20.]),
        (False,True,False,1,True,True,.9,.93,[40.,86.,-20.]),
        (False,True,False,2,True,False,.9,.93,[1.,2.,3.]),
        (False,True,False,2,True,False,0.,1.,[9.,100.,-99.]),
    ]
    java = '''public class CameraControlOracle {
  static double[] smooth = {0,0,0}; static int last = 0;
  static double[] update(boolean running, boolean freeLook, int perspective,
      boolean firstPOV, boolean reset, float horizontal, float vertical, double[] pos) {
    if (freeLook) return pos;
    if (!running) { last=perspective; return pos; }
    if (reset && last!=perspective) { smooth=pos.clone(); last=perspective; return smooth; }
    last=perspective;
    if (!firstPOV && perspective==0) { smooth=pos.clone(); return smooth; }
    double squared=smooth[0]*smooth[0]+smooth[1]*smooth[1]+smooth[2]*smooth[2];
    if (Math.abs(squared) < (double)1.0e-5f) smooth=pos.clone();
    smooth=new double[]{smooth[0]*horizontal+pos[0]*(1-horizontal),
      smooth[1]*vertical+pos[1]*(1-vertical),smooth[2]*horizontal+pos[2]*(1-horizontal)};
    return smooth;
  }
  static void row(boolean disable, boolean running, boolean freeLook, int perspective,
      boolean firstPOV, boolean reset, float horizontal, float vertical, double[] pos) {
    if(disable) smooth=new double[]{0,0,0};
    double[] result=update(running,freeLook,perspective,firstPOV,reset,horizontal,vertical,pos);
    System.out.println(Long.toUnsignedString(Double.doubleToRawLongBits(result[0]))+","+
      Long.toUnsignedString(Double.doubleToRawLongBits(result[1]))+","+
      Long.toUnsignedString(Double.doubleToRawLongBits(result[2]))+","+last);
  }
  public static void main(String[] args) {
'''
    for row in actions:
        disable, running, free, perspective, first, reset, h, v, pos = row
        boolean = lambda value: str(value).lower()
        java += '    row(' + ','.join([boolean(disable), boolean(running), boolean(free), str(perspective),
            boolean(first), boolean(reset), str(h)+'f', str(v)+'f',
            'new double[]{'+','.join(str(x) for x in pos)+'}']) + ');\n'
    java += '  }\n}\n'
    java_file = OUT / 'CameraControlOracle.java'
    java_file.write_text(java, encoding='utf-8')
    subprocess.run([str(JAVA / 'javac.exe'), '-d', str(OUT), str(java_file)], check=True, capture_output=True, text=True)
    oracle = subprocess.run([str(JAVA / 'java.exe'), '-cp', str(OUT), 'CameraControlOracle'], check=True, capture_output=True, text=True)
    expected = [[int(value) for value in line.split(',')] for line in oracle.stdout.splitlines()]
    (OUT / 'java-oracle.json').write_text(json.dumps({'actions':actions,'expectedDoubleBitsAndLastPerspective':expected}, indent=2)+'\n', encoding='utf-8')

    source = '''#![cfg(test)]
#![allow(dead_code)]
mod camera;
mod camera_control;
mod behavior;
mod gfx {
    use glam::Mat4;
    const NEAR_PLANE:f32=0.05;
'''
    graphics = (UPSTREAM / 'crates/horizon-client-render/src/gfx.rs').read_text(encoding='utf-8')
    source += block(graphics, 'pub(crate) fn world_projection(') + '\n'
    view = (UPSTREAM / 'crates/horizon-client-render/src/gfx/view.rs').read_text(encoding='utf-8').split('\n#[cfg(test)]', 1)[0]
    source += 'mod view {\n' + '\n'.join(line for line in view.splitlines() if not line.startswith('//!')) + '\n}\n'
    source += '''use view::View;
    struct Config { width:u32,height:u32 }
    struct Renderer { config:Config }
    impl Renderer {
'''
    source += block(changes['crates/horizon-client-render/src/gfx.rs'], 'fn screen_view(') + '\n}\n'
    source += '''#[test] fn screen_lens_and_culling_use_the_same_adjusted_aspect() {
        crate::camera_control::publish(100);
        let renderer=Renderer{config:Config{width:1920,height:1080}};
        let old=renderer.screen_view(70.,0.,0.,512.);
        crate::camera_control::publish(200);
        let changed=renderer.screen_view(70.,0.,0.,512.);
        assert_eq!(changed.aspect,old.aspect/2.);
        assert_eq!(changed.projection.x_axis.x,old.projection.x_axis.x*2.);
        assert_eq!(changed.projection.y_axis,old.projection.y_axis);
        let set=view::ViewSet::single(changed);
        assert_eq!(set.cull().projection,set.primary().projection);
        assert_eq!(set.cull().aspect,changed.aspect);
        crate::camera_control::publish(100);
    }
}
mod effective_game_fixture {
  pub mod perspective {
    #[derive(Clone,Copy,Debug,PartialEq,Eq)]
    pub enum CameraType {FirstPerson,ThirdPersonBack,ThirdPersonFront}
  }
  mod native_hooks {
    use super::perspective::CameraType;
    use crate::behavior::{Perspective,QuickPerspectiveSwap};
    struct Studio(bool); impl Studio {fn flying(&self)->bool {self.0}}
    struct Game {camera_type:CameraType,studio:Studio,running:bool,quick:QuickPerspectiveSwap}
    impl Game {
      fn native_client_camera_perspective(&self,original:u8,managed:bool,perspective:&mut u8) {
        *perspective=self.quick.perspective(self.running,false,managed,
            Perspective::from_code(original).unwrap()) as u8;
      }
'''
    native = changes['crates/horizon-client/src/game/native_hooks.rs']
    source += block(native, 'pub(super) fn native_raw_camera_perspective(') + '\n'
    source += block(native, 'pub(super) fn effective_camera_type(') + '\n}\n'
    source += '''#[test] fn effective_perspective_preserves_f5_option_and_releases_immediately() {
      let mut game=Game{camera_type:CameraType::FirstPerson,studio:Studio(false),running:true,quick:QuickPerspectiveSwap::default()};
      assert_eq!(game.effective_camera_type(),CameraType::ThirdPersonFront);
      assert_eq!(game.native_raw_camera_perspective(),0);
      game.camera_type=CameraType::ThirdPersonBack;
      assert_eq!(game.effective_camera_type(),CameraType::ThirdPersonFront);
      assert_eq!(game.native_raw_camera_perspective(),1);
      game.camera_type=CameraType::ThirdPersonFront;
      assert_eq!(game.effective_camera_type(),CameraType::ThirdPersonBack);
      game.running=false;assert_eq!(game.effective_camera_type(),CameraType::ThirdPersonFront);
      game.running=true;game.studio.0=true;
      assert_eq!(game.effective_camera_type(),CameraType::ThirdPersonFront);
    }
  }
}
#[test] fn transformed_camera_moves_the_smoothed_eye_before_collision_rays() {
    use camera::{Camera,CameraEntity,CameraType};
    let entity=CameraEntity{position:[10.,60.,20.],previous:[8.,58.,18.],
      x_rot:0.,y_rot:0.,x_rot_o:0.,y_rot_o:0.,eye_height:0.,living:None,mount:None};
    let mut view=Camera::default();let mut rays=Vec::new();
    view.align_with_entity_position(&entity,0.5,CameraType::ThirdPersonBack,|d|d,
      |pos|{assert_eq!(pos,[9.,59.,19.]);[40.,80.,-20.]},
      |from,to|{rays.push((from,to));None});
    assert_eq!(view.eye,[40.,80.,-20.]);assert_eq!(rays.len(),8);
    for (from,_) in rays {assert!((from[0]-40.).abs()<0.11);assert!((from[1]-80.).abs()<0.11);}
    assert!((view.position[2]+24.).abs()<1e-5);
    let mut plain=Camera::default();
    plain.align_with_entity_distance(&entity,0.5,CameraType::FirstPerson,|_|panic!(),|_,_|None);
    assert_eq!(plain.position,[9.,59.,19.]);
}
#[test] fn aspect_changes_culling_bounds_but_preserves_hand_and_fluid_planes() {
    camera_control::publish(100);
    let old=camera::near_plane_bounds(70.,1920.,1080.,0.,0.,512.);
    let hand=camera::hand_projection(70.,1920.,1080.,512.);
    let camera=camera::Camera::default();let plane=camera.near_plane(70.,1920.,1080.);
    camera_control::publish(200);
    let new=camera::near_plane_bounds(70.,1920.,1080.,0.,0.,512.);
    assert!((new.0[0]-old.0[0]/2.).abs()<1e-6);
    assert!((new.1[0]-old.1[0]/2.).abs()<1e-6);
    assert_eq!(new.0[1],old.0[1]);assert_eq!(new.1[1],old.1[1]);
    assert_eq!(camera::hand_projection(70.,1920.,1080.,512.),hand);
    assert_eq!(camera.near_plane(70.,1920.,1080.),plane);
    camera_control::publish(100);
}
#[test] fn kotlin_float_weights_match_source_derived_java_oracle_bit_for_bit() {
    use behavior::{Perspective,SmoothCamera};
    let mut camera=SmoothCamera::default();
'''
    for index, (row, bits) in enumerate(zip(actions, expected)):
        disable, running, free, perspective, first, reset, h, v, pos = row
        if disable: source += 'camera.on_disabled();\n'
        source += f'camera.enable_first_pov={str(first).lower()};camera.reset_on_perspective_change={str(reset).lower()};camera.horizontal_factor={h}f32;camera.vertical_factor={v}f32;\n'
        source += 'let result=camera.camera_update(' + ','.join([str(running).lower(),str(free).lower(),
            f'Perspective::from_code({perspective}).unwrap()', '['+','.join(str(p) for p in pos)+']']) + ');\n'
        source += f'assert_eq!(result.map(f64::to_bits),[{bits[0]}u64,{bits[1]}u64,{bits[2]}u64],"oracle row {index}");\n'
        source += f'assert_eq!(camera.last_perspective as u8,{bits[3]},"oracle perspective row {index}");\n'
    source += '}\n'
    # Exercise generate(), including the helper Host implementation, rather
    # than copying an assumed event layout into the fixture. Keep its source
    # tree isolated from the parent's staging and the pristine upstream tree.
    api_root = OUT / 'api-source'
    api_game = api_root / 'crates/horizon-client/src/game'
    api_game.mkdir(parents=True, exist_ok=True)
    (api_root / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    render_generated = api_root / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render_generated.parent.mkdir(parents=True, exist_ok=True)
    render_generated.write_text('use super::*; pub trait RenderHooks: Send + Sync {}\n', encoding='utf-8')
    (api_root / 'crates/horizon-client/src/mod_api.rs').write_text('', encoding='utf-8')
    methods = '\n'.join(block(native, anchor) for anchor in (
        'pub(super) fn sync_native_camera_aspect(', 'pub(super) fn native_client_camera_aspect(',
        'pub(super) fn native_raw_camera_perspective(', 'pub(super) fn native_client_camera_position(',
        'pub(super) fn native_client_camera_perspective(', 'pub(super) fn effective_camera_type('))
    native_impl = 'impl Game<\'_> {\n' + methods + '\n}\n'
    (api_game / 'native_hooks.rs').write_text(native_impl, encoding='utf-8')
    catalog = {'hooks': [{'name': name,
        'target': f"horizon_client::game::native_hooks::Game<'_>::native_{name}",
        'mod_api': {'contract': 'listen'}} for name in contract['hook_names']],
        'helpers': [{'name': name,
        'target': f"horizon_client::game::native_hooks::Game<'_>::native_{name}"}
        for name in contract['helper_names']]}
    (api_root / 'native-hooks.json').write_text(json.dumps(catalog, indent=2)+'\n', encoding='utf-8')
    api_fixture = (HERE / 'camera-api-fixture.rs').read_text(encoding='utf-8')
    (OUT / 'camera_api_fixture.rs').write_text(api_fixture.replace('__NATIVE_HOOK_METHODS__', native_impl), encoding='utf-8')
    source += '\ninclude!("camera_api_fixture.rs");\n'
    (OUT / 'lib.rs').write_text(source, encoding='utf-8')
    (OUT / 'build.rs').write_text('''fn main() {
        let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");
        let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();
    }\n''', encoding='utf-8')
    core = (UPSTREAM / 'crates/horizon-core').as_posix()
    (OUT / 'Cargo.toml').write_text('''[package]
name="camera-control-hook-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[dependencies]
glam="0.30"
horizon-core={path="''' + core + '"}\n[build-dependencies]\nhorizon-mod-api-gen={path="' + (UPSTREAM / 'crates/horizon-mod-api-gen').as_posix() + '"}\n', encoding='utf-8')
    result = subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,capture_output=True,text=True)
    (OUT / 'test.log').write_text(result.stdout+result.stderr, encoding='utf-8')
    print(result.stdout+result.stderr)
    report = {'hookContract':contract,'sourceFiles':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in sorted(changes.items())},
              'testedProductionBodies':['Camera (complete module)','Renderer::screen_view','View/ViewSet','Game::effective_camera_type','Game::native_raw_camera_perspective','all camera native hook/helper bodies at their actual module boundaries'],
              'apiGenerator':'horizon_mod_api_gen::generate (Target::Client); generated dispatch, event, Host and Host implementation compiled',
              'javaOracleRows':len(actions),'oracleScope':'Source-derived Java float/double arithmetic oracle; not execution of the original Kotlin module',
              'fixtureResult':'passed' if result.returncode==0 else 'failed','fullGameIntegrationPassed':False}
    (OUT / 'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
