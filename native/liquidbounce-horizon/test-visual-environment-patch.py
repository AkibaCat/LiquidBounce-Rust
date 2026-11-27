#!/usr/bin/env python3
"""Compile complete transformed production visual functions and exercise outcomes.

Scaffolding supplies asset/world boundaries; the tested render/fog/admission,
terrain state machine and light propagation bodies are copied from production.
The full Horizon build and in-game verification remain separate checks.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
UPSTREAM = ROOT / 'upstream/Horizon'
OUT = ROOT / 'build/visual-environment-hook-tests'


def block(text, anchor):
    start = text.index(anchor)
    opening = text.index('{', start)
    level, end = 1, opening + 1
    while level:
        if text[end] == '{': level += 1
        elif text[end] == '}': level -= 1
        end += 1
    return text[start:end]


def main():
    changes = {}
    def edit(path, transform):
        original = UPSTREAM / path
        changes[path] = transform(changes.get(path, original.read_text(encoding='utf-8') if original.exists() else ''))
    def once(text, source, target):
        if text.count(source) != 1:
            raise ValueError(f'Expected exactly one anchor: {source[:100]!r}')
        return text.replace(source, target, 1)
    spec = importlib.util.spec_from_file_location('visual_environment_patch', HERE / 'visual-environment-patch.py')
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    contract = helper.apply(edit, once)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / 'visual_environment.rs').write_text(helper.policy_source(), encoding='utf-8')
    (OUT / 'visual_behavior.rs').write_text((ROOT / 'native/liquidbounce-behaviors/src/visual_environment.rs').read_text(encoding='utf-8'), encoding='utf-8')

    source = '''#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
mod visual_environment;
mod visual_behavior;
use visual_environment::{Choice, ALL, publish};
mod spans { pub fn time<T>(_: &str, run: impl FnOnce() -> T) -> T { run() } }
mod pack_palette {
    #[derive(Clone, Copy)] pub struct Environment {
        pub powder_snow_fog: Option<u32>, pub water_fog_brightening_time:f32,
        pub water_fog_brightening:f32, pub blindness_fog_distance:f32,
        pub darkness_fog_distance:f32, pub boss_fog_distance:f32,
    }
    impl Default for Environment { fn default() -> Self { Self { powder_snow_fog:None,
        water_fog_brightening_time:600.,water_fog_brightening:1.,blindness_fog_distance:5.,
        darkness_fog_distance:15.,boss_fog_distance:60. } } }
    #[derive(Clone, Copy, Default)] pub struct FluidFogs { pub lava:Option<FluidFog>, pub water:Option<FluidFog> }
    #[derive(Clone, Copy)] pub struct FluidFog { pub color:Option<u32>,pub radius:f32,pub fade:f32 }
}
mod environment {
    #[derive(Clone, Copy)] pub struct Sky { pub fog_color:[f32;3], pub sky_color:[f32;3],
        pub sun_angle:f32,pub sunrise_color:[f32;4],pub fog_start:f32,pub fog_end:f32,pub cloud_fog_end:f32 }
    impl Default for Sky { fn default()->Self {Self{fog_color:[0.2,0.3,0.4],sky_color:[0.4,0.5,0.6],
        sun_angle:0.,sunrise_color:[0.;4],fog_start:100.,fog_end:300.,cloud_fog_end:1024.}} }
}
mod fog {
    use crate::environment::Sky;
    #[derive(Default)] pub struct FogState {pub water_vision_time:i32,pub rain_fog_multiplier:f32}
'''
    fog = changes['crates/horizon-client-render/src/fog.rs']
    for anchor in ('#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum FogType',
                   'pub struct FogData', '#[derive(Default)]\npub struct FogTables', 'pub struct FogInputs', 'fn rgb(', 'pub fn setup_fog('):
        source += block(fog, anchor) + '\n'
    source += 'impl FogState {\n' + block(fog, 'pub fn water_vision(') + block(fog, 'pub fn water_vision_over(') + '}\n'
    source += '''pub fn input<'a>(sky:&'a Sky, kind:FogType) -> FogInputs<'a> { FogInputs {
        fog_type:kind,sky,water_fog_color:[0.1,0.2,0.3],biome:0,render_distance_chunks:12,
        distant_terrain:false,cloud_range_chunks:32,camera_y:64.,min_y:0,flat:false,forward:[0.,0.,1.],
        rain:0.,thunder:0.,skybox:true,blindness:None,darkness:None,darken_world:0.,boss_world_fog:false,
        spectator:false,fire_resistance:false,full_water_vision:false,night_vision:0.,
        environment:Default::default(),fluid_fog:Default::default() } }
}
mod hud_layout {
    use std::borrow::Cow;
    pub const WHITE:u32=0xFFFFFFFF;
    #[derive(Clone,Copy,Debug,PartialEq)] pub enum Blend { Textured,Vignette,Nausea }
    #[derive(Clone,Debug)] pub enum PoseOp {Translate(f32,f32),Scale(f32,f32)}
    #[derive(Clone,Debug)] pub enum GuiDraw {
        Blit {texture:Cow<'static,str>,blend:Blend,x:i32,y:i32,w:i32,h:i32,uv:[f32;2],src:[i32;2],tex:[i32;2],color:u32},
        AtlasSprite{sprite:Cow<'static,str>,x:i32,y:i32,w:i32,h:i32,color:u32},
        Fill{x0:i32,y0:i32,x1:i32,y1:i32,color:u32},Push(Vec<PoseOp>),Pop,
    }
'''
    layout = (UPSTREAM / 'crates/horizon-client-render/src/hud_layout.rs').read_text(encoding='utf-8')
    for anchor in ('pub fn fill(', 'pub fn floor(', 'pub fn as8(', 'pub fn white(', 'pub fn color_from_float(', 'pub fn argb('):
        source += block(layout, anchor) + '\n'
    source += '}\nmod hud_overlays { use std::borrow::Cow; use crate::hud_layout::{Blend,GuiDraw,PoseOp,white,fill,color_from_float};\n'
    hud = changes['crates/horizon-client-render/src/hud_overlays.rs']
    for anchor in ('fn screen_blit(', 'pub fn vignette(', 'pub fn spyglass(', 'pub fn portal_overlay(', 'pub fn confusion_overlay(',
                   '#[derive(Clone, Debug, Default, PartialEq)]\npub struct CameraOverlays', 'pub fn camera_overlays('):
        source += block(hud, anchor) + '\n'
    source += '''}
mod hud {
    #[derive(Clone,Copy)] pub struct Rect {pub u0:f32,pub v0:f32,pub u1:f32,pub v1:f32}
    #[derive(Clone,Copy,Default)] pub struct TextureFlags {pub blur:bool,pub clamp:bool}
}
mod screen_effects {
    use crate::{hud::{Rect,TextureFlags},hud_layout::GuiDraw}; use std::collections::HashMap;
    const SAMPLE_BLUR:u32=1; const SAMPLE_CLAMP:u32=2;
    #[derive(Clone,Copy,Debug)] pub struct ScreenVertex {pub pos:[f32;2],pub uv:[f32;2],pub rect:[f32;4],pub color:[f32;4]}
'''
    screen = changes['crates/horizon-client-render/src/screen_effects.rs']
    for anchor in ('#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum Blend', '#[derive(Clone, Debug)]\npub struct ScreenQuad',
                   'pub struct ScreenInputs', 'pub fn sampling(', 'fn hand_scale('):
        source += block(screen, anchor) + '\n'
    source += 'pub struct ScreenEffects; impl ScreenEffects {' + block(screen, 'pub fn build(') + '}}\n'

    loading = changes['crates/horizon-client-render/src/level_loading.rs']
    source += 'mod loading {const CLIENT_WAIT_TIMEOUT_MS:u64=30_000; #[derive(Clone,Copy,Debug)] pub enum Reason{Other}\n'
    for anchor in ('#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum ClientState',
                   '#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub struct PlayerCheck',
                   '#[derive(Clone, Copy, Debug)]\npub struct LevelLoadTracker', 'impl LevelLoadTracker'):
        source += block(loading, anchor) + '\n'
    source += '''}
mod particles {
    use std::collections::BTreeMap;
    const SPORE_BLOSSOM_LIMIT:usize=1000;
    #[derive(Clone,Copy,PartialEq,Debug)] pub enum ParticleType{Explosion,ExplosionEmitter,Smoke,SporeBlossomAir}
    impl ParticleType {fn name(self)->&'static str {match self{Self::Explosion=>"explosion",Self::ExplosionEmitter=>"explosion_emitter",Self::Smoke=>"smoke",Self::SporeBlossomAir=>"spore_blossom_air"}}
        fn override_limiter(self)->bool {true}}
    #[derive(Clone)] pub struct ParticleOptions{pub ty:ParticleType,pub mod_particle:Option<String>,pub source:Option<String>}
    #[derive(Clone)] pub struct Particle{pub ty:ParticleType}
    #[derive(Default)] struct Seeds{draws:u32} impl Seeds{fn next_long(&mut self)->i64{self.draws+=1;self.draws as i64}}
    struct Registry; impl Registry{fn modifies_particles(&self)->bool{false}}
    mod polytone {pub fn modify(_: &super::Registry,_:&str,_:Option<&str>,_:&mut super::Particle){}}
    #[derive(Default)] pub struct ParticleEngine {
        pub particles:Vec<Particle>,pub pending:Vec<Particle>,seeds:Seeds,
        pub made:BTreeMap<&'static str,u64>,created_requests:Option<Vec<(ParticleOptions,[f64;3],[f64;3])>>,
        requested:Option<Vec<(ParticleOptions,[f64;3],[f64;3])>>,polytone:Option<Registry>,camera:[f64;3],
    }
    impl ParticleEngine {
        fn make(&self,o:&ParticleOptions,_:[f64;3],_:[f64;3],_:i64)->Option<Particle>{Some(Particle{ty:o.ty})}
        fn calculate_particle_level(&mut self,_:bool)->u8{0}
        pub fn draws(&self)->u32{self.seeds.draws}
'''
    source += block(changes['crates/horizon-client-render/src/particles/spawn.rs'], 'pub fn create(') + '\n'
    source += block(changes['crates/horizon-client-render/src/particles.rs'], 'pub fn add_with(') + '\n'
    source += '''}}
mod lighting {
    use std::collections::{HashMap,VecDeque};
    type Pos=(i32,i32,i32);
    #[derive(Default)] pub struct Region {pub cells:HashMap<Pos,(u32,u8)>,pub source_y:HashMap<(i32,i32),i32>}
    impl Region {
        fn cell(&mut self,x:i32,y:i32,z:i32)->Option<(u32,u8)>{self.cells.get(&(x,y,z)).copied()}
        fn light(&mut self,x:i32,y:i32,z:i32)->Option<u8>{self.cell(x,y,z).map(|v|v.1)}
        fn state(&mut self,x:i32,y:i32,z:i32)->Option<u32>{self.cell(x,y,z).map(|v|v.0)}
        fn update_light(&mut self,x:i32,y:i32,z:i32,f:impl FnOnce(u8)->u8){if let Some(c)=self.cells.get_mut(&(x,y,z)){c.1=f(c.1)}}
    }
    pub struct Cell{pub dampening:u8,pub emission:u8}
    pub struct LightTables{pub cells:Vec<Cell>} impl LightTables{fn blocked(&self,_:usize,_:usize,_:usize)->bool{false}}
    #[derive(Default)] pub struct RelightScratch {decrease:VecDeque<(Pos,u8)>,cleared:Vec<Pos>,increase:VecDeque<Pos>,sources:HashMap<(i32,i32),i32>}
    #[derive(Default)] pub struct LightChanges{pub changed:Vec<Pos>} impl LightChanges {fn push(&mut self,p:Pos){self.changed.push(p)}}
    fn lowest_sky_source(world:&mut Region,_:&LightTables,x:i32,z:i32)->i32{world.source_y.get(&(x,z)).copied().unwrap_or(999)}
'''
    light = changes['crates/horizon-client-render/src/light.rs']
    start = light.index('const OFFSETS:')
    source += light[start:light.index('];', start)+2] + '\n'
    for anchor in ('fn level(', 'fn set_level(', 'fn relight_layer('):
        source += block(light, anchor).replace('fn relight_layer(', 'pub fn relight_layer(', 1) + '\n'
    source += '''}
use hud_layout::GuiDraw;
fn reset(){publish(ALL,100,false,false)}
fn off(choice:Choice){publish(ALL & !(1 << choice as u8),100,false,false)}

#[test]
fn every_choice_and_raw_opacity_survives_the_worker_snapshot(){
    reset(); assert_eq!(visual_environment::fire_alpha(),229./255.);
    for (index,choice) in visual_behavior::DoRender::ALL.into_iter().enumerate(){
        publish(ALL & !choice.bit(),50,true,true);
        assert!(!visual_environment::allows(unsafe_free_choice(index)));
        assert!(visual_environment::hide_particles());assert!(visual_environment::skip_terrain());
        assert_eq!(visual_environment::fire_alpha(),114./255.);
    }
    publish(ALL,-100,false,false);assert_eq!(visual_environment::fire_alpha(),27./255.);
    reset(); assert!(!visual_environment::hide_particles());assert!(!visual_environment::skip_terrain());
}
fn unsafe_free_choice(index:usize)->Choice {
    let all=[''' + ','.join('Choice::' + tag for tag in helper.TAGS) + '''];all[index]
}

#[test]
fn fluid_fog_keeps_colour_and_render_distance_and_clears_four_environment_distances(){
    let sky=environment::Sky::default();let state=fog::FogState::default();let tables=fog::FogTables::default();
    for kind in [fog::FogType::Water,fog::FogType::Lava,fog::FogType::PowderSnow]{
        reset();let input=fog::input(&sky,kind);let original=fog::setup_fog(&tables,&state,&input);
        off(if kind==fog::FogType::PowderSnow{Choice::PowderSnowFog}else{Choice::LiquidsFog});
        let clear=fog::setup_fog(&tables,&state,&input);
        assert_eq!(clear.color,original.color);assert_eq!(clear.render_distance_start,original.render_distance_start);
        assert_eq!(clear.render_distance_end,original.render_distance_end);
        assert_eq!([clear.environmental_start,clear.environmental_end,clear.sky_end,clear.cloud_end],[f32::MAX;4]);
    }
    reset();
}

#[test]
fn effect_environment_priority_is_preserved_and_effects_are_never_removed(){
    let sky=environment::Sky::default();let state=fog::FogState::default();let tables=fog::FogTables::default();
    let mut input=fog::input(&sky,fog::FogType::Water);input.blindness=Some((100,false));input.darkness=Some(1.);
    reset();let blind=fog::setup_fog(&tables,&state,&input);assert_eq!(blind.environmental_end,5.);
    off(Choice::LiquidsFog);assert_eq!(fog::setup_fog(&tables,&state,&input).environmental_end,5.);
    off(Choice::Blinding);let dark=fog::setup_fog(&tables,&state,&input);assert_eq!(dark.environmental_end,15.);
    publish(ALL & !(1<<Choice::Blinding as u8) & !(1<<Choice::Darkness as u8),100,false,false);
    let water=fog::setup_fog(&tables,&state,&input);assert_eq!(water.environmental_end,24.);
    assert_eq!(input.blindness,Some((100,false)));assert_eq!(input.darkness,Some(1.));
    reset();
}

#[test]
fn camera_overlays_filter_exact_textures_and_do_not_fall_through_a_cancelled_portal(){
    let mut i=hud_overlays::CameraOverlays{first_person:true,equipment:vec!["misc/pumpkinblur".into(),"mod:custom_overlay".into()],
        ticks_frozen:1,percent_frozen:1.,nausea:1.,portal:[1.,1.],..Default::default()};
    publish(ALL & !(1<<Choice::PumpkinBlur as u8) & !(1<<Choice::PowderSnowFog as u8) & !(1<<Choice::PortalOverlay as u8),100,false,false);
    let out=hud_overlays::camera_overlays(640,360,&i,&mut 0.5);
    assert_eq!(out.len(),1);assert!(matches!(&out[0],GuiDraw::Blit{texture,..} if texture=="mod:custom_overlay"));
    i.portal=[0.,0.];assert_eq!(hud_overlays::camera_overlays(640,360,&i,&mut 0.5).len(),4);
    publish(ALL & !(1<<Choice::Nausea as u8),100,false,false);
    let out=hud_overlays::camera_overlays(640,360,&i,&mut 0.5);
    assert_eq!(out.len(),3);assert!(out.iter().all(|d|!matches!(d,GuiDraw::Blit{blend:hud_layout::Blend::Nausea,..})));
    off(Choice::SpyglassOverlay);assert!(hud_overlays::spyglass(640,360,1.).is_empty());reset();
}

#[test]
fn screen_wall_and_fire_are_independent_from_the_underwater_overlay(){
    use std::collections::HashMap;use hud::{Rect,TextureFlags};
    let sprites:HashMap<String,Rect>=["block/stone","block/fire_1","misc/underwater"].into_iter()
        .map(|n|(n.to_owned(),Rect{u0:0.,v0:0.,u1:1.,v1:1.})).collect();
    let flags=HashMap::<String,TextureFlags>::new();
    let input=screen_effects::ScreenInputs{sprites:&sprites,flags:&flags,aspect:1.,partial:0.,
        block_overlay:Some("block/stone".into()),water:Some((1.,0.,0.)),on_fire:true,fire_lower:0.,gui_overlays:&[]};
    publish(ALL & !(1<<Choice::WallOverlay as u8),50,false,false);
    let out=screen_effects::ScreenEffects.build(&input);assert_eq!(out.len(),3);
    assert!(out[0].world_tint);assert_eq!(out[0].vertices[0].color[3],0.1);
    assert_eq!(out[1].vertices[0].color[3],114./255.);assert_eq!(out[2].vertices[0].color[3],114./255.);
    reset();assert_eq!(screen_effects::ScreenEffects.build(&input).len(),4);
}

#[test]
fn particle_admission_preserves_old_particles_and_constructor_draws(){
    use particles::*;reset();let mut engine=ParticleEngine::default();
    let smoke=ParticleOptions{ty:ParticleType::Smoke,mod_particle:None,source:None};
    engine.create(&smoke,[0.;3],[0.;3]);assert_eq!(engine.pending.len(),1);
    engine.particles.push(Particle{ty:ParticleType::Smoke});publish(ALL,100,true,false);
    engine.create(&smoke,[0.;3],[0.;3]);assert_eq!(engine.pending.len(),1);assert_eq!(engine.particles.len(),1);
    assert_eq!(engine.draws(),2);assert_eq!(engine.made.get("smoke"),Some(&2));
    off(Choice::ExplosionParticles);for ty in [ParticleType::Explosion,ParticleType::ExplosionEmitter]{
        let options=ParticleOptions{ty,mod_particle:None,source:None};engine.add_with(&options,[0.;3],[0.;3],true,true);
    }
    assert_eq!(engine.draws(),2);engine.add_with(&smoke,[0.;3],[0.;3],true,true);assert_eq!(engine.pending.len(),2);reset();
}

#[test]
fn terrain_skips_chunk_build_and_delay_only_after_server_load_start(){
    use loading::*;publish(ALL,100,false,true);let mut tracker=LevelLoadTracker::new(500);
    let player=PlayerCheck{outside_build_height:false,spectator:false,alive:true};
    tracker.start_client_load(0,Reason::Other);tracker.tick_client_load(40_000,player);
    assert!(!tracker.is_level_ready(40_000));tracker.loading_packets_received();tracker.tick_client_load(40_001,player);
    assert!(tracker.is_level_ready(40_001));reset();
    tracker.start_client_load(0,Reason::Other);tracker.loading_packets_received();tracker.tick_client_load(1,player);
    assert!(!tracker.is_level_ready(10_000));tracker.player_section_ready();tracker.tick_client_load(2,player);
    assert!(!tracker.is_level_ready(501));assert!(tracker.is_level_ready(502));
}

#[test]
fn skylight_switch_blocks_only_the_increase_pass(){
    use lighting::*;
    let tables=LightTables{cells:vec![Cell{dampening:1,emission:0},Cell{dampening:1,emission:15}]};
    let mut sky=Region::default();for x in 0..3{sky.cells.insert((x,0,0),(0,0));}sky.source_y.insert((0,0),0);
    let mut scratch=RelightScratch::default();let mut changes=LightChanges::default();off(Choice::SkylightUpdates);
    relight_layer(&mut sky,&tables,true,&[(0,0,0)],false,&mut scratch,&mut changes);
    assert_eq!(sky.cells[&(0,0,0)].1,0xF0);assert_eq!(sky.cells[&(1,0,0)].1,0);
    reset();relight_layer(&mut sky,&tables,true,&[(0,0,0)],false,&mut scratch,&mut changes);
    assert_eq!(sky.cells[&(1,0,0)].1,0xE0);
    off(Choice::SkylightUpdates);sky.source_y.clear();
    relight_layer(&mut sky,&tables,true,&[(0,0,0)],false,&mut scratch,&mut changes);
    assert!(sky.cells.values().all(|c|c.1==0));
    let mut block=Region::default();for x in 0..3{block.cells.insert((x,0,0),(if x==0{1}else{0},0));}
    relight_layer(&mut block,&tables,false,&[(0,0,0)],false,&mut scratch,&mut changes);
    assert_eq!(block.cells[&(0,0,0)].1,15);assert_eq!(block.cells[&(1,0,0)].1,14);reset();
}
'''
    (OUT / 'lib.rs').write_text(source, encoding='utf-8')
    (OUT / 'Cargo.toml').write_text('''[package]
name = "visual-environment-hook-tests"
version = "0.0.0"
edition = "2024"
[lib]
path = "lib.rs"
[dependencies]
log = "0.4"
''', encoding='utf-8')
    result = subprocess.run(['cargo', '+stable', 'test', '--offline', '--manifest-path', str(OUT / 'Cargo.toml'), '--', '--test-threads=1'], cwd=ROOT, capture_output=True, text=True)
    (OUT / 'test.log').write_text(result.stdout + result.stderr, encoding='utf-8')
    print(result.stdout + result.stderr)
    report = {'hookContract':contract,'sourceFiles':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in sorted(changes.items())},
              'fixtureResult':'passed' if result.returncode == 0 else 'failed',
              'fullGameIntegrationPassed':False,
              'skylightSemantics':'Original unguarded SkyLightEngine mixin: global increase gate, decreases and block light preserved.'}
    (OUT / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
