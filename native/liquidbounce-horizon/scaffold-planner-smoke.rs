// SPDX-License-Identifier: GPL-3.0-or-later
// Prepared for the coordinator's centralized Rust batch; not run alone.
#[path = "../liquidbounce-behaviors/src/scaffold_movement_planner.rs"]
mod rules;
use rules::*;

fn cardinal(yaw:f32)->Position {
    // Exact cardinal vectors isolate planner semantics from native float-table trig.
    match yaw as i32 {0=>[0.,0.,1.],90=>[-1.,0.,0.],-90=>[1.,0.,0.],-180=>[0.,0.,-1.],_=>panic!("unexpected angle {yaw}")}
}
fn candidate(block:BlockPos,delta:f64,area:f64,distance:f64)->Candidate {
    Candidate{block,surface_delta:delta,overlap:area,distance_squared:distance}
}
fn main() {
    assert_eq!(movement_angle(0.,[true,false,true,false]),-45.);
    assert_eq!(movement_angle(0.,[false,true,true,false]),225.);
    assert_eq!(movement_angle(0.,[false,false,true,false]),-90.);
    assert_eq!(movement_angle(0.,[true,true,true,true]),0.);
    let mut state=Planner::default();
    assert_eq!(state.choose_angle(0.),0.);
    assert_eq!(state.choose_angle(30.),0.);
    assert_eq!(state.choose_angle(30.00001),45.);
    assert_eq!(state.choose_angle(15.),45.);
    state.reset();
    assert_eq!(state.choose_angle(22.5),0.); // kotlin.math.round ties to even
    state.reset();
    assert_eq!(state.choose_angle(67.5),90.);
    state.reset();
    assert_eq!(state.choose_angle(179.),-180.);
    assert_eq!(state.choose_angle(-151.),-180.);

    let block=[0,63,0];
    let bounds=[0.2,64.,0.2,0.8,65.8,0.8];
    let shapes=[[0.,0.,0.,0.5,1.,1.],[0.5,0.,0.,1.,1.,1.]];
    let c=Candidate::from_boxes(block,&shapes,[0.5,64.,0.5],bounds);
    assert_eq!(c.surface_delta,0.);
    assert!((c.overlap-0.36).abs()<1e-12);
    assert_eq!(c.distance_squared,0.);
    let no_overlap=Candidate::from_boxes(block,&[[0.,0.,0.,0.1,1.,0.1]],[0.5,64.,0.5],bounds);
    assert_eq!(no_overlap.surface_delta,f64::INFINITY);
    assert_eq!(no_overlap.overlap,0.);
    assert!(candidate(block,0.,0.01,3.).compare(candidate([1,63,0],0.002,1.,0.)).is_lt());
    assert!(candidate(block,0.,0.5,3.).compare(candidate([1,63,0],0.001,0.4,0.)).is_lt());
    assert!(candidate(block,0.,0.4,3.).compare(candidate([1,63,0],0.001,0.39,0.)).is_gt());

    let mut state=Planner::default();
    let a=candidate([0,63,0],0.,0.5,0.);
    let b=candidate([1,63,0],0.,0.51,1.);
    state.track(b.block);
    let line=state.line([0.8,64.,0.6],0.,&[a,b],cardinal).unwrap();
    assert_eq!(state.last_position,Some(b.block)); // history preferred within overlap tolerance
    assert_eq!(line.position,[0.8,64.,0.6]);
    assert!((state.support.unwrap().offset_x+0.7).abs()<1e-12);
    let worse=candidate(b.block,0.002,0.51,1.);
    state.line([0.8,64.,0.6],0.,&[a,worse],cardinal).unwrap();
    assert_eq!(state.last_position,Some(a.block));
    state.track([0,63,0]);state.track([0,63,1]);
    let line=state.line([0.8,64.,1.3],0.,&[a],cardinal).unwrap();
    assert_eq!(line.position,[0.5,64.,1.3]); // follows last two placements
    let line=state.line([0.8,64.,1.3],90.,&[a],cardinal).unwrap();
    assert_eq!(line.position,[0.8,64.,1.3]); // direction change anchors current support
    assert!(state.line([0.8,64.,1.3],90.,&[],cardinal).is_none());
    assert!(state.support.is_none()&&state.last_position.is_none());
    assert_eq!(state.direction_angle,90.);
    assert!(!state.history.is_empty());
    state.track([2,63,0]);state.track([3,63,0]);state.track([4,63,0]);
    assert_eq!(state.history.len(),4);
    state.track([4,63,0]);assert_eq!(state.history.len(),4);
    state.reset();
    assert!(state.history.is_empty()&&state.direction_angle.is_nan());
}
