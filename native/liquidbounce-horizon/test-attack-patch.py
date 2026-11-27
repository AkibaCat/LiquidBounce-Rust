#!/usr/bin/env python3
"""Exercise the actual transformed attack/continue/stop algorithms as Rust.

Only the game's HandWorld is a fixture. Production definitions and method
bodies are extracted from the generated transform of the pristine source.
"""
from pathlib import Path
import hashlib
import json
import runpy
import subprocess

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
OUT = ROOT / 'build/attack-hook-tests'


def block(text, start):
    offset = text.index(start)
    brace = text.index('{', offset)
    depth = 1
    end = brace + 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[offset:end]


FIXTURE = r'''
struct World {
    hit: Hit, remove: bool, cancel: bool, retain: bool,
    busy: bool, creative: bool, spear: bool, swings: usize, resets: usize,
    attacks: usize, stages: usize,
}
impl Default for World {
    fn default() -> Self { Self { hit: Hit::Miss, remove:false, cancel:false,
        retain:false, busy:false, creative:false, spear:false, swings:0,
        resets:0, attacks:0, stages:0 } }
}
impl HandWorld for World {
    fn hit(&mut self) -> Hit { self.hit }
    fn client_attack_queries(&self) -> (bool, bool) { (self.remove, self.cancel) }
    fn client_abort_breaking(&self) -> bool { self.retain }
    fn is_air(&mut self, _: [i32;3]) -> bool { false }
    fn creative(&self) -> bool { self.creative }
    fn spectator(&self) -> bool { false }
    fn instabuild(&self) -> bool { self.creative }
    fn hands_busy(&self) -> bool { self.busy }
    fn using_item(&self) -> bool { false }
    fn item_enabled(&mut self, _:Hand) -> bool { true }
    fn held_empty(&mut self, _:Hand) -> bool { true }
    fn held_count(&mut self, _:Hand) -> i32 { 0 }
    fn infinite_materials(&self) -> bool { self.creative }
    fn cannot_attack(&mut self) -> bool { false }
    fn piercing_weapon(&mut self) -> bool { self.spear }
    fn attack_animation(&mut self) -> SwingAnimation { SwingAnimation::DEFAULT }
    fn interact_animation(&mut self, _:Hand) -> SwingAnimation { SwingAnimation::DEFAULT }
    fn piercing_attack(&mut self, _:SwingAnimation) { self.attacks+=1; }
    fn attack_entity(&mut self) -> bool { self.attacks+=1; true }
    fn block_action_restricted(&mut self, _:[i32;3]) -> bool { false }
    fn within_border(&mut self, _:[i32;3]) -> bool { true }
    fn attack_block(&mut self, _:[i32;3]) { self.attacks+=1; }
    fn destroy_progress(&mut self, _:[i32;3]) -> f32 { 0.2 }
    fn destroy_block(&mut self, _:[i32;3]) -> bool { true }
    fn main_hand_key(&mut self) -> String { "fixture".into() }
    fn destroy_stage(&mut self, _:[i32;3], _:Option<i32>) { self.stages+=1; }
    fn swing(&mut self, _:Hand, _:SwingAnimation) { self.swings+=1; }
    fn reset_attack_strength(&mut self) { self.resets+=1; }
    fn entity_within_border(&mut self) -> bool { true }
    fn use_on_entity(&mut self, _:Hand) -> Option<UseResult> { None }
    fn use_on_block(&mut self, _:Hand) -> UseResult { UseResult::Pass }
    fn use_item(&mut self, _:Hand) -> UseResult { UseResult::Pass }
    fn item_used(&mut self, _:Hand) {}
    fn release_using(&mut self) {}
}

#[test]
fn miss_head_precedes_busy_cooldown_and_spear_without_side_effects() {
    let mut world = World { cancel:true, busy:true, spear:true, ..Default::default() };
    let mut clicks = Clicks { miss_time:7, ..Default::default() };
    assert!(clicks.start_attack(&mut world));
    assert_eq!((clicks.miss_time,world.swings,world.resets,world.attacks), (7,0,0,0));
    world.hit=Hit::Entity;
    assert!(!clicks.start_attack(&mut world));
    assert_eq!(world.attacks,0);
}

#[test]
fn only_start_attack_ignores_cooldown_and_does_not_erase_the_stored_value() {
    let mut world = World { remove:true, ..Default::default() };
    let mut clicks = Clicks { miss_time:7, ..Default::default() };
    assert!(!clicks.start_attack(&mut world));
    assert_eq!((clicks.miss_time,world.swings,world.resets), (7,1,1));
    world.hit=Hit::Block {pos:[0,1,0],face:1};
    clicks.continue_attack(&mut world,true);
    assert!(!clicks.destroying.active);
    assert_eq!(world.swings,1);
    world.remove=false;
    clicks.start_attack(&mut world);
    assert_eq!(world.swings,1);
    clicks.miss_time=0;
    world.hit=Hit::Miss;
    clicks.start_attack(&mut world);
    assert_eq!((clicks.miss_time,world.swings), (10,2));
}

#[test]
fn abort_keeps_real_accumulated_progress_then_disabled_stop_resets_it() {
    let mut world = World { hit:Hit::Block{pos:[1,2,3],face:1}, retain:true, ..Default::default() };
    let mut clicks=Clicks::default();
    clicks.start_attack(&mut world);
    clicks.continue_attack(&mut world,true);
    assert!(clicks.destroying.active);
    assert_eq!(clicks.destroying.progress,0.2);
    let before=clicks.destroying.clone();
    let resets=world.resets;
    let stages=world.stages;
    world.hit=Hit::Miss;
    clicks.continue_attack(&mut world,false);
    assert_eq!(clicks.destroying,before);
    assert_eq!((world.resets,world.stages),(resets,stages));
    world.hit=Hit::Block{pos:[1,2,3],face:1};
    clicks.continue_attack(&mut world,true);
    assert_eq!(clicks.destroying.progress,0.4);
    world.retain=false;
    world.hit=Hit::Miss;
    clicks.continue_attack(&mut world,false);
    assert!(!clicks.destroying.active);
    assert_eq!(clicks.destroying.progress,0.0);
    assert_eq!(world.resets,resets+1);
}
'''


def main():
    changed = {}
    def edit(path, transform):
        current = changed.get(path)
        if current is None:
            current = (ROOT/'upstream/Horizon'/path).read_text(encoding='utf-8')
        # The attack extension inserts ahead of the camera extension's anchor.
        if path.endswith('native_hooks.rs'):
            current = current.replace('    /// Player FOV scale, after vanilla effects and before zoom and easing.\n', '    /// Integer Options.fov value before Camera.calculateFov / getNearPlane.\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
        changed[path] = transform(current)
    def replace_once(text, old, new):
        assert text.count(old) == 1, old[:100]
        return text.replace(old,new,1)
    runpy.run_path(str(HERE/'attack-patch.py'))['apply'](edit,replace_once)
    text=changed['crates/horizon-client/src/first_person.rs']
    declarations=[]
    for declaration in ['pub enum Hand','impl Hand','pub struct SwingAnimation','impl SwingAnimation','pub enum Hit','pub enum UseResult','impl UseResult','pub trait HandWorld','pub struct Destroying','impl Default for Destroying','impl Destroying','pub struct Clicks']:
        snippet=block(text,declaration)
        if declaration.startswith('pub enum') or declaration=='pub struct SwingAnimation':
            snippet='#[derive(Clone,Copy,Debug,PartialEq)]\n'+snippet
        elif declaration=='pub struct Destroying':
            snippet='#[derive(Clone,Debug,PartialEq)]\n'+snippet
        elif declaration=='pub struct Clicks':
            snippet='#[derive(Default)]\n'+snippet
        declarations.append(snippet)
    declarations.append('impl Clicks {\n'+'\n'.join(block(text,s) for s in ['pub fn start_attack','pub fn continue_attack'])+'\n}')
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'src').mkdir(exist_ok=True)
    (OUT/'Cargo.toml').write_text('[package]\nname="liquidbounce-attack-hook-tests"\nversion="0.1.0"\nedition="2024"\n',encoding='utf-8')
    source='\n'.join(declarations)+'\n'+FIXTURE
    (OUT/'src/lib.rs').write_text(source,encoding='utf-8')
    result=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml')],cwd=ROOT)
    report={'scope':'actual_generated_attack_and_stop_algorithms_with_HandWorld_fixture','exitCode':result.returncode,'sourceSha256':hashlib.sha256(source.encode()).hexdigest(),'transformedPaths':list(changed)}
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode)


if __name__=='__main__':
    main()
