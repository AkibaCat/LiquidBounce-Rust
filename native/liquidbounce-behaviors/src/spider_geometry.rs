// SPDX-License-Identifier: GPL-3.0-or-later
//! ShapeExtensions.shrink(x=0.0001,z=0.0001), box decomposition surface.
pub fn shrink_box(mut b:[f64;6])->Option<[f64;6]>{
    if !(b[3]-b[0]>0.0002&&b[5]-b[2]>0.0002){return None;}
    b[0]+=0.0001;b[2]+=0.0001;b[3]-=0.0001;b[5]-=0.0001;Some(b)
}
pub fn should_shrink(block_y:i32,player_y:f64,shift:bool,ground:bool)->bool{f64::from(block_y)>=player_y||(shift&&ground)}
