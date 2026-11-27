// SPDX-License-Identifier: GPL-3.0-or-later
//! Scoped native prediction controls; no world/player pointer is retained.
use std::cell::Cell;
#[derive(Clone,Copy)]
struct Prediction{force_surface:bool,ignore_edge:bool,backed_off:bool}
thread_local!{
 static PREDICTION:Cell<Option<Prediction>>=const{Cell::new(None)};
 static SAFE_WALK:Cell<bool>=const{Cell::new(false)};
}
pub fn set_safe_walk(value:bool){SAFE_WALK.set(value);}
pub fn staying_on_surface(sneaking:bool)->bool{PREDICTION.with(|p|p.get().is_some()||sneaking||SAFE_WALK.get())}
pub fn ignores_edge()->bool{PREDICTION.with(|p|p.get().is_some_and(|p|p.ignore_edge))}
pub fn record_backoff(before:horizon_core::geometry::Vec3,after:horizon_core::geometry::Vec3){PREDICTION.with(|slot|{if let Some(mut state)=slot.get(){state.backed_off|=before.x!=after.x||before.z!=after.z;slot.set(Some(state));}});}
/// Agent SimulatedPlayer always probes the sneak back-off, then applies it only
/// when this simulated input holds the edge. LedgeSneak reads the probe result.
pub fn finish_backoff(before:horizon_core::geometry::Vec3,after:horizon_core::geometry::Vec3,sneaking:bool)->horizon_core::geometry::Vec3{
 record_backoff(before,after);PREDICTION.with(|slot|slot.get().map_or(after,|p|if !p.ignore_edge&&(sneaking||p.force_surface){after}else{before}))
}
pub fn with_prediction<R>(force_surface:bool,ignore_edge:bool,run:impl FnOnce()->R)->(R,bool){
 struct Restore(Option<Prediction>);impl Drop for Restore{fn drop(&mut self){PREDICTION.set(self.0);}}
 let _restore=Restore(PREDICTION.replace(Some(Prediction{force_surface,ignore_edge,backed_off:false})));
 let result=run();let backed=PREDICTION.with(|p|p.get().expect("prediction remains scoped").backed_off);(result,backed)
}
