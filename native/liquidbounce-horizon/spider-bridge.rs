// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::OnceLock;
static JUMP:OnceLock<fn(bool,&mut f32)>=OnceLock::new();
static ACTIVE:OnceLock<fn(&mut bool)>=OnceLock::new();
pub fn install(callback:fn(bool,&mut f32),active:fn(&mut bool)){let _=JUMP.set(callback);let _=ACTIVE.set(active);}
fn client_spider_jump(horizontal_collision:bool,motion:&mut f32){if let Some(callback)=JUMP.get(){callback(horizontal_collision,motion);}}
#[inline(never)]pub fn native_client_spider_jump(horizontal_collision:bool,motion:&mut f32){client_spider_jump(horizontal_collision,motion);}
pub fn jump(horizontal_collision:bool,motion:&mut f32){native_client_spider_jump(horizontal_collision,motion);}
fn client_spider_shape_active(active:&mut bool){if let Some(callback)=ACTIVE.get(){callback(active);}}
#[inline(never)]pub fn native_client_spider_shape_active(active:&mut bool){client_spider_shape_active(active);}
pub fn active()->bool{let mut active=false;native_client_spider_shape_active(&mut active);active}
