// SPDX-License-Identifier: GPL-3.0-or-later
//! Scoped render state; nested previews restore the surrounding entity.
use std::cell::Cell;
thread_local! {static COLORS:Cell<Option<[u32;2]>>=const {Cell::new(None)};static MARKERS:Cell<u8>=const {Cell::new(0)};}
pub struct Scope(Option<[u32;2]>);
impl Drop for Scope {fn drop(&mut self){COLORS.with(|s|s.set(self.0));}}
pub fn scope(colors:Option<[u32;2]>)->Scope {Scope(COLORS.with(|s|s.replace(colors)))}
pub fn active()->bool {COLORS.with(|s|s.get().is_some())}
pub fn body(original:u32)->u32 {COLORS.with(|s|s.get().map_or(original,|c|c[0]))}
pub fn feature(original:u32)->u32 {COLORS.with(|s|s.get().map_or(original,|c|c[1]))}
pub fn mark(vertices:&mut [crate::mesher::Vertex]) {if active(){for v in vertices{v.layer|=crate::gfx::OIT_ENTITY;}}}
pub struct MarkerScope(u8);impl Drop for MarkerScope{fn drop(&mut self){MARKERS.with(|s|s.set(self.0));}}
pub fn marker_scope(mask:u8)->MarkerScope{MarkerScope(MARKERS.with(|s|s.replace(mask)))}
pub fn marker_matches(original:bool,name:&str)->bool{original||MARKERS.with(|s|s.get()&2!=0&&name=="minecraft:light")}
