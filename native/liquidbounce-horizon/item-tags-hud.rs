// SPDX-License-Identifier: GPL-3.0-or-later
// RoundedRectGuiElementRenderState's integer size/radius and original SDF.
impl HudBatch {
    pub fn liquidbounce_rounded_rect(&mut self,rect:[f32;4],radius:f32,top:u32,bottom:u32,stroke:f32){
        let [x0,y0,x1,y1]=rect;let w=x1-x0;let h=y1-y0;
        if w<=0.||h<=0.||top>>24==0{return;}
        let encode=|v:f32|(v.clamp(0.,32767.)+0.5).floor() as u32;
        let shape=0x80000000|encode(w)|(encode(h)<<15);
        let radius=encode(radius.max(0.).min(w.min(h)*0.5)) as f32;
        let corners=[[x0,y0],[x0,y1],[x1,y1],[x1,y0]];let uv=[[0.,0.],[0.,1.],[1.,1.],[1.,0.]];
        for i in [0,1,2,0,2,3]{self.vertices.push(HudVertex{pos:corners[i],uv:uv[i],color:if i==1||i==2{bottom}else{top},depth:0.,glint_uv:[radius,stroke.max(0.)],text:shape});}
    }
}
