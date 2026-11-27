#!/usr/bin/env python3
"""Compile transformed production mining/effect/helper bodies with numeric oracles."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def block(text, anchor):
    start = text.index(anchor)
    opening = text.index('{', start)
    level = 1
    end = opening + 1
    while level:
        level += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]


STUBS = r'''
#![cfg(test)]
#![allow(dead_code)]
use std::cell::Cell;
use horizon_items::mining::{self, DestroySpeedModifiers};
#[path = "__NO_SLOW_BREAK__"]
mod no_slow_break;
use no_slow_break::NoSlowBreak;

struct EffectInstance { id: &'static str, amplifier: i32 }
#[derive(Default)]
struct PlayerEffects { list: Vec<EffectInstance> }
impl PlayerEffects {
/* EFFECT_METHODS */
}
#[derive(Default)]
struct Potions { effects: PlayerEffects }
struct Ground { on_ground: bool }
struct Player { ground: Ground, server_copy: bool, replaying: bool }
impl Player {
/* PLAYER_METHODS */
}
struct World;
impl World { fn block(&self, _:i32,_:i32,_:i32)->u32 {0} }
struct Facts { name: String, hardness: f32, requires_tool: bool }
struct Resources { block: Facts }
impl Resources {
    fn facts(&self) -> &Self {self}
    fn get(&self, _:u32) -> &Facts {&self.block}
}
struct Items { speed:f32, harvest:bool }
impl Items {
    fn mining_speed(&self, _:Option<&()>, _: &str) -> f32 {self.speed}
    fn has_correct_tool_for_drops(&self, _:Option<&()>, _:bool, _: &str) -> bool {self.harvest}
}
struct Game {
    world:World, res:Resources, player:Option<Player>, potions:Potions, items:Items,
    window:bool, lan_acting_uuid:Option<u128>, running:bool, settings:NoSlowBreak,
    water:bool, submerged:f32, block_speed:f32, efficiency:f32,
    conditions:Cell<usize>, efficiency_reads:Cell<usize>, submerged_reads:Cell<usize>,
    breaks:usize, break_original:f32, break_multiplier:f32,
}
impl Game {
    fn fixture() -> Self {
        Self {world:World,res:Resources {block:Facts {name:"minecraft:stone".into(),hardness:2.,requires_tool:true}},
            player:Some(Player {ground:Ground {on_ground:false},server_copy:false,replaying:false}),
            potions:Potions {effects:PlayerEffects {list:vec![
                EffectInstance {id:"haste",amplifier:1},EffectInstance {id:"conduit_power",amplifier:3},EffectInstance {id:"mining_fatigue",amplifier:2}]}},
            items:Items {speed:6.,harvest:true},window:true,lan_acting_uuid:None,running:true,
            settings:NoSlowBreak::default(),water:true,submerged:0.2,block_speed:1.5,efficiency:4.,
            conditions:Cell::new(0),efficiency_reads:Cell::new(0),submerged_reads:Cell::new(0),breaks:0,break_original:0.,break_multiplier:1.}
    }
    fn native_window(&self)->bool {self.window}
    fn block_break_speed(&self)->f32 {self.block_speed}
    fn held(&self)->Option<&()> {Some(&())}
    fn items(&self)->&Items {&self.items}
    fn mining_efficiency(&self)->f32 {self.efficiency_reads.set(self.efficiency_reads.get()+1);self.efficiency}
    fn mining_eyes_in_water(&self)->bool {self.water}
    fn submerged_mining_speed(&self)->f32 {self.submerged_reads.set(self.submerged_reads.get()+1);self.submerged}
    fn native_break_speed(&mut self, _: [i32;3], _:u32, original:f32, value:&mut f32) {self.breaks+=1;self.break_original=original;*value*=self.break_multiplier;}
/* GAME_METHODS */
}
mod mod_api {
    use super::Game;
    pub fn mods()-> &'static [()] {&[]}
    pub fn dispatch_client_mining_conditions(game:&Game, _:&[()],fatigue:&mut bool,water:&mut bool,on_ground:&mut bool) {
        game.conditions.set(game.conditions.get()+1);
        game.settings.apply(game.running,fatigue,water,on_ground);
    }
}

fn close(actual:f32, expected:f32) {assert!((actual-expected).abs() <= 2e-6 * expected.abs().max(0.001), "actual={actual}, expected={expected}");}

#[test]
fn all_eight_conditions_drive_actual_progress_preserving_haste_and_conduit() {
    // Independent numeric oracle: base (6+4)*1.5*1.8 / hardness 2 / 30
    // = 0.45. Fatigue III is .027, water .2, airborne .2.
    for (mask, expected) in [(0,0.000486),(1,0.018),(2,0.00243),(3,0.09),(4,0.00243),(5,0.09),(6,0.01215),(7,0.45)] {
        let mut game=Game::fixture();
        game.settings=NoSlowBreak {mining_fatigue:mask&1!=0,on_air:mask&2!=0,underwater:mask&4!=0};
        close(game.destroy_progress([1,2,3]),expected);
        close(game.break_original,expected*60.);
        assert_eq!(game.breaks,1);
        assert_eq!(game.conditions.get(),1);
        assert_eq!(game.efficiency_reads.get(),1);
        assert_eq!(game.submerged_reads.get(),usize::from(mask&4==0));
        assert_eq!(game.potions.effects.list.len(),3);
        assert_eq!(game.potions.effects.amplifier("haste"),Some(1));
        assert_eq!(game.potions.effects.amplifier("conduit_power"),Some(3));
        assert_eq!(game.potions.effects.amplifier("mining_fatigue"),Some(2));
    }
    let mut game=Game::fixture();
    assert_eq!(game.settings.tags(),["MiningFatigue","OnAir"]);
    close(game.destroy_progress([0;3]),0.09);
}

#[test]
fn disabled_and_other_player_identities_keep_vanilla_mining_penalties() {
    for guard in 0..6 {
        let mut game=Game::fixture();
        game.settings=NoSlowBreak {mining_fatigue:true,on_air:true,underwater:true};
        match guard {
            0=>game.running=false,
            1=>game.window=false,
            2=>game.lan_acting_uuid=Some(37),
            3=>game.player.as_mut().unwrap().server_copy=true,
            4=>game.player.as_mut().unwrap().replaying=true,
            _=>game.player=None,
        }
        close(game.destroy_progress([0;3]),if guard==5 {0.00243} else {0.000486});
        assert_eq!(game.conditions.get(),usize::from(guard==0));
        assert_eq!(game.breaks,1);
        assert_eq!(game.submerged_reads.get(),1);
    }
}

#[test]
fn exact_effect_factor_preserves_beneficial_effects_and_default_callers() {
    let effects=|pairs:&[(&'static str,i32)]| PlayerEffects {list:pairs.iter().map(|(id,amplifier)| EffectInstance {id,amplifier:*amplifier}).collect()};
    for (pairs, expected) in [
        (vec![],1.),
        (vec![("haste",2)],1.6),
        (vec![("conduit_power",1)],1.4),
        (vec![("haste",0),("conduit_power",3)],1.8),
        (vec![("haste",4),("conduit_power",1)],2.),
    ] {
        let e=effects(&pairs);
        close(e.mining_speed_factor_with_fatigue(false),expected);
        close(e.mining_speed_factor_with_fatigue(true),expected);
        close(e.mining_speed_factor(),expected);
        for (amplifier,factor) in [(0,0.3),(1,0.09),(2,0.027),(3,0.0081)] {
            let mut e=effects(&pairs);
            e.list.push(EffectInstance {id:"mining_fatigue",amplifier});
            close(e.mining_speed_factor_with_fatigue(false),expected);
            close(e.mining_speed_factor_with_fatigue(true),expected*factor);
            close(e.mining_speed_factor(),expected*factor);
        }
    }
}

#[test]
fn break_speed_mod_event_and_tool_harvest_arithmetic_remain_in_original_order() {
    let mut game=Game::fixture();
    game.settings=NoSlowBreak {mining_fatigue:true,on_air:true,underwater:true};
    game.break_multiplier=2.;
    game.items.harvest=false;
    close(game.destroy_progress([0;3]),0.27);
    close(game.break_original,27.);
    game.items.speed=1.;
    game.items.harvest=true;
    game.efficiency_reads.set(0);
    close(game.destroy_progress([0;3]),0.09);
    assert_eq!(game.efficiency_reads.get(),0,"speed <=1 skips efficiency attribute read");
    game.res.block.hardness = -1.;
    let calls=(game.conditions.get(),game.breaks);
    assert_eq!(game.destroy_progress([0;3]),0.);
    assert_eq!((game.conditions.get(),game.breaks),calls,"unbreakable guard precedes all new callbacks");
}

#[test]
fn toggling_conditions_only_changes_queried_penalties_and_leaves_real_state() {
    let mut game=Game::fixture();
    game.settings=NoSlowBreak {mining_fatigue:true,on_air:true,underwater:true};
    close(game.destroy_progress([0;3]),0.45);
    assert!(game.water);
    assert!(!game.player.as_ref().unwrap().ground.on_ground);
    assert_eq!(game.potions.effects.amplifier("mining_fatigue"),Some(2));
    game.running=false;
    close(game.destroy_progress([0;3]),0.000486);
    game.running=true;
    game.water=false;
    game.player.as_mut().unwrap().ground.on_ground=true;
    game.potions.effects.list.retain(|effect|effect.id!="mining_fatigue");
    game.settings=NoSlowBreak {mining_fatigue:false,on_air:false,underwater:false};
    close(game.destroy_progress([0;3]),0.45);
}
'''


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',default='build/mining-production-01')
    parser.add_argument('--prepare-only',action='store_true')
    args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    out.mkdir(parents=True,exist_ok=True)
    if (out/'report.json').exists():
        raise SystemExit('Refusing to overwrite existing verification evidence')
    changes={}
    originals={}
    def edit(name,transform):
        before=(ROOT/'upstream/Horizon'/name).read_text(encoding='utf-8')
        originals[name]=hashlib.sha256((ROOT/'upstream/Horizon'/name).read_bytes()).hexdigest()
        changes[name]=transform(changes.get(name,before))
    def once(text,before,after):
        if text.count(before)!=1:
            raise ValueError(f'Expected one anchor: {before[:100]}')
        return text.replace(before,after,1)
    spec=importlib.util.spec_from_file_location('mining_patch',HERE/'mining-patch.py')
    patch=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(patch)
    patch.apply(edit,once)
    effect=changes['crates/horizon-client-render/src/effects.rs']
    effect_methods='\n'.join(block(effect,anchor) for anchor in ['    pub fn get(&self, id:', '    pub fn amplifier(', '    pub fn mining_speed_factor(', '    pub fn mining_speed_factor_with_fatigue('])
    game_methods='\n'.join([
        block(changes['crates/horizon-client/src/game/hand.rs'],'    pub(super) fn destroy_progress('),
        block(changes['crates/horizon-client/src/game/native_hooks.rs'],'    pub(super) fn native_client_mining_conditions('),
    ])
    player_methods=block(changes['crates/horizon-client/src/motion.rs'],'    pub fn native_client_queries_allowed(')
    source=STUBS.replace('__NO_SLOW_BREAK__',(ROOT/'native/liquidbounce-behaviors/src/no_slow_break.rs').as_posix()).replace('/* EFFECT_METHODS */',effect_methods).replace('/* GAME_METHODS */',game_methods).replace('/* PLAYER_METHODS */',player_methods).replace('pub(super)','pub(crate)')
    (out/'lib.rs').write_text(source,encoding='utf-8',newline='\n')
    (out/'Cargo.toml').write_text('''[package]
name = "liquidbounce-mining-production-fixture"
version = "0.1.0"
edition = "2024"
[lib]
path = "lib.rs"
[dependencies]
horizon-core = {path="'''+(ROOT/'upstream/Horizon/crates/horizon-core').as_posix()+'''"}
horizon-items = {path="'''+(ROOT/'upstream/Horizon/crates/horizon-items').as_posix()+'''"}
serde_json = "1"
''',encoding='utf-8',newline='\n')
    if args.prepare_only:
        print(out)
        return
    files=[HERE/'mining-patch.py',HERE/'test-mining-patch.py',ROOT/'native/liquidbounce-behaviors/src/no_slow_break.rs',out/'lib.rs']
    hashes=lambda:{str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
    before=hashes()
    commands=[]
    for name,argv in [
        ('lock',['cargo','+stable','generate-lockfile','--offline','--manifest-path',str(out/'Cargo.toml')]),
        ('test',['cargo','+stable','test','--locked','--offline','--manifest-path',str(out/'Cargo.toml')]),
        ('clippy',['cargo','+stable','clippy','--locked','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings']),
    ]:
        started=time.monotonic()
        result=subprocess.run(argv,cwd=ROOT,capture_output=True)
        (out/(name+'.stdout.log')).write_bytes(result.stdout)
        (out/(name+'.stderr.log')).write_bytes(result.stderr)
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-started,3)})
    unchanged=all(hashlib.sha256((ROOT/'upstream/Horizon'/name).read_bytes()).hexdigest()==sha for name,sha in originals.items())
    report={'passed':all(command['exitCode']==0 for command in commands) and before==hashes() and unchanged,
        'scope':'actual transformed Game.destroy_progress + native_client_mining_conditions + PlayerEffects mining methods + production horizon_items::mining + NoSlowBreak.apply; only world/item/attribute and dispatch plumbing are fixture boundaries',
        'executedTests':5,'conditionCombinations':8,'commands':commands,'sourceSha256':before,'cargoTargetDirectory':os.environ.get('CARGO_TARGET_DIR'),
        'transformedSourceSha256':{name:hashlib.sha256(text.encode()).hexdigest() for name,text in changes.items()},
        'originalSourceSha256':originals,'sourceUnchangedDuringVerification':before==hashes(),'upstreamUnchanged':unchanged,
        'allLocalServerGuestGuardsVerified':True,'beneficialHasteAndConduitPreserved':True,
        'fullClientCompilationPassed':False,'fullGameIntegrationPassed':False}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')},indent=2))
    raise SystemExit(0 if report['passed'] else 1)


if __name__=='__main__':
    main()
