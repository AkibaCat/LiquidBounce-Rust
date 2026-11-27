// SPDX-License-Identifier: GPL-3.0-or-later
// Prepared for the coordinator's centralized Rust batch; not run alone.
#[path="../liquidbounce-behaviors/src/scaffold_prediction.rs"]
mod prediction;
use prediction::*;
fn identity(v:Position,_:f32)->Position {v}
fn main() {
    let defaults=Settings::default();
    assert_eq!(defaults.bootstrap_backoff,0.2);
    assert_eq!(defaults.cutoff_distance,0.05);
    assert_eq!(defaults.warmup_placements,2);
    assert_eq!(with_length([0.001,0.,0.],2.),[0.;3]);
    assert_eq!(with_length([3.,0.,4.],10.),[6.,0.,8.]);
    let mut state=Prediction::default();
    assert_eq!(state.average(),None);
    let settings=Settings{bootstrap_backoff:0.,..defaults};
    assert_eq!(state.predict([0.,64.,0.],[1.,64.,0.],0.,None,settings,identity),[1.,64.,0.]);
    assert_eq!(state.predict([0.,64.,0.],[1.,64.,0.],0.,Some([0.2,-0.1]),settings,identity),[1.2,64.,-0.1]);
    let settings=Settings{bootstrap_backoff:0.25,..defaults};
    assert_eq!(state.predict([0.,64.,0.],[1.,64.,0.],0.,None,settings,identity),[0.75,64.,0.]);
    state.record(false,[0.8,64.,0.],Some([1.,64.,0.]),Some(0.),identity);
    state.record(true,[0.8,64.,0.],None,Some(0.),identity);
    state.record(true,[0.8,64.,0.],Some([1.,64.,0.]),None,identity);
    assert!(state.history.is_empty());
    state.record(true,[0.5,64.,0.],Some([1.,64.,0.]),Some(0.),identity);
    assert_eq!(state.average(),Some([-0.5,0.,0.]));
    assert_eq!(state.predict([0.,64.,0.],[1.,64.,0.],0.,None,settings,identity),[0.625,64.,0.]);
    state.record(true,[0.5,64.,0.],Some([1.,64.,0.]),Some(0.),identity);
    assert_eq!(state.predict([0.,64.,0.],[1.,64.,0.],0.,None,settings,identity),[0.5,64.,0.]);
    let instant=Settings{warmup_placements:0,..settings};
    state.history.pop_back();
    assert_eq!(state.predict([0.,64.,0.],[1.,64.,0.],0.,None,instant,identity),[0.5,64.,0.]);
    for x in 1..=5 {state.record(true,[f64::from(x),64.,0.],Some([0.,64.,0.]),Some(0.),identity);}
    assert_eq!(state.history.len(),4);
    assert_eq!(state.average(),Some([3.5,0.,0.]));
    let mut angles=Vec::new();
    state.record(true,[1.,64.,0.],Some([0.,64.,0.]),Some(0.7),|v,a|{angles.push(a);v});
    state.predict([0.,64.,0.],[1.,64.,0.],0.7,None,settings,|v,a|{angles.push(a);v});
    assert_eq!(angles,[0.7,-0.7]);
    state.reset();assert!(state.history.is_empty());
}
