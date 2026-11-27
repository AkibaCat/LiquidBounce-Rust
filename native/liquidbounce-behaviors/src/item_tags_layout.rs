// SPDX-License-Identifier: GPL-3.0-or-later
//! Exact ItemStackListLayout and stable downward GuiOverlapRearranger.
pub fn dimensions(count:usize,row_length:usize,title_width:Option<i32>)->[i32;2]{
    let row_length=row_length.max(1);let mut size=[18*count.min(row_length) as i32,18*count.div_ceil(row_length) as i32];
    if let Some(width)=title_width{size[0]=size[0].max(width);size[1]+=9+if count==0{0}else{2};}size
}
pub fn bounds(center:[f32;2],size:[i32;2],padding:f32,scale:f32)->[f32;4]{
    let half=size.map(|v|(v as f32+padding*2.)*scale*0.5);
    [center[0]-half[0],center[1]-half[1],center[0]+half[0],center[1]+half[1]]
}
pub fn rearrange(boxes:&mut[[f32;4]]){
    for i in 1..boxes.len(){loop{
        let b=boxes[i];let mut lowest:Option<f32>=None;
        for o in &boxes[..i]{if o[0]<b[2]&&o[2]>b[0]&&o[1]<b[3]&&o[3]>b[1]&&lowest.is_none_or(|v|o[3]>v){lowest=Some(o[3]);}}
        let Some(y)=lowest else{break;};boxes[i]=[b[0],y,b[2],y+b[3]-b[1]];
    }}
}
