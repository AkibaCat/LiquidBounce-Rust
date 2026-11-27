// SPDX-License-Identifier: GPL-3.0-or-later
//! Synchronous real jump HEAD callback; only actual local MotionWorld calls it.
use std::sync::OnceLock;
static JUMP:OnceLock<fn(&mut f32)>=OnceLock::new();
pub fn install(callback:fn(&mut f32)){let _=JUMP.set(callback);}
fn client_high_jump(motion:&mut f32){if let Some(callback)=JUMP.get(){callback(motion);}}
/// Actual PlayerJumpEvent HEAD, registered as a free client-library hook.
#[inline(never)]
pub fn native_client_high_jump(motion:&mut f32){client_high_jump(motion);}
pub fn jump(motion:&mut f32){native_client_high_jump(motion);}
