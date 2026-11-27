// SPDX-License-Identifier: GPL-3.0-or-later
//! Pure BlockIn target planning from ModuleBlockIn.

use crate::surround::Pos;

pub const DISABLE_FINISH: u32 = 1 << 0;
pub const DISABLE_MOVE: u32 = 1 << 1;
pub const DISABLE_ON: [&str; 2] = ["Finish", "Move"];
pub const ORDER_NORMAL: &str = "Normal";
pub const ORDER_RANDOM: &str = "Random";
pub const ORDER_BOTTOM_TOP: &str = "BottomTop";
pub const ORDER_TOP_BOTTOM: &str = "TopBottom";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub disable_on: u32,
    pub avoid_broken_blocks: bool,
    pub broken_block_window: i32,
    pub place_order: String,
    pub whitelist: bool,
    pub blocks: Vec<String>,
}
impl Default for Settings {
    fn default() -> Self { Self { disable_on: DISABLE_FINISH|DISABLE_MOVE, avoid_broken_blocks:true, broken_block_window:1000, place_order:ORDER_NORMAL.into(), whitelist:false, blocks:vec![] } }
}

pub fn positions(start: Pos, height: i32, clockwise: bool) -> Vec<Pos> {
    let mut out=Vec::with_capacity(18);
    out.push([start[0],start[1]-1,start[2]]);
    let dirs=if clockwise { [[1,0,0],[0,0,1],[-1,0,0],[0,0,-1]] } else { [[1,0,0],[0,0,-1],[-1,0,0],[0,0,1]] };
    for d in dirs { for y in 0..height.max(1) { out.push([start[0]+d[0],start[1]+y,start[2]+d[2]]); } }
    out.push([start[0],start[1]+height.max(1),start[2]]); out
}

pub fn order(mut positions: Vec<Pos>, tag: &str, mut random_index: impl FnMut(usize)->usize) -> Vec<Pos> {
    match tag { ORDER_RANDOM => { for i in (2..=positions.len()).rev() { let j=random_index(i); positions.swap(i-1,j); } }, ORDER_BOTTOM_TOP => positions.sort_by_key(|p|p[1]), ORDER_TOP_BOTTOM => positions.sort_by(|a,b|b[1].cmp(&a[1])), _=>{} }
    positions
}

#[cfg(test)]
mod tests { use super::*; #[test] fn normal_is_floor_sides_roof(){assert_eq!(positions([0,10,0],2,false).len(),10);} #[test] fn order_vertical(){let p=vec![[0,3,0],[0,1,0],[0,2,0]];assert_eq!(order(p,ORDER_BOTTOM_TOP, |_|0)[0],[0,1,0]);} }
