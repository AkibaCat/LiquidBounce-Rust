// SPDX-License-Identifier: GPL-3.0-or-later
//! The actual global BlockShapeEvent policy, after BlockWalk/AirJump assignment.
use horizon_core::geometry::{Aabb,Axis,ShapeOp,Vec3,VoxelShape};
use std::sync::{RwLock,OnceLock};
#[path="spider_geometry.rs"]mod geometry;
static ACTIVE:OnceLock<fn()->bool>=OnceLock::new();
static POLICY:RwLock<(f64,bool,bool)>=RwLock::new((0.0,false,false));
pub fn install(active:fn()->bool){let _=ACTIVE.set(active);}
pub fn player(y:f64,shift:bool,ground:bool){*POLICY.write().unwrap_or_else(|e|e.into_inner())=(y,shift,ground);}
pub fn shape(block_y:i32,original:VoxelShape)->VoxelShape{
    if crate::block_walk::shape_changes_suppressed()||!ACTIVE.get().is_some_and(|active|active()){return original;}
    let (y,shift,ground)=*POLICY.read().unwrap_or_else(|e|e.into_inner());
    if !geometry::should_shrink(block_y,y,shift,ground)||original.is_empty(){return original;}
    let mut result=VoxelShape::empty();
    for b in original.boxes(){if let Some(b)=geometry::shrink_box([b.min(Axis::X),b.min(Axis::Y),b.min(Axis::Z),b.max(Axis::X),b.max(Axis::Y),b.max(Axis::Z)]){
        let shape=VoxelShape::from_box(Aabb::new(Vec3::new(b[0],b[1],b[2]),Vec3::new(b[3],b[4],b[5]))).expect("valid shrunk source box");
        result=result.join_unoptimized(&shape,ShapeOp::OR).expect("finite source shape union");
    }}
    result.optimized().expect("finite source shape optimization")
}
