// Original LiquidBounce gui/rounded_rect.fsh. Integer size/radius matches its vertex encoding.
fn liquidbounce_rounded(v:Out)->vec4<f32>{
    let size=max(vec2(f32(v.shape&32767u),f32((v.shape>>15u)&32767u)),vec2(1.));
    let half_size=size*0.5;let radius=min(max(v.glint_uv.x,0.),min(half_size.x,half_size.y));
    let p=(v.uv-vec2(0.5))*size;let q=abs(p)-half_size+vec2(radius);
    let d=length(max(q,vec2(0.)))+min(max(q.x,q.y),0.)-radius;
    let aa=max(fwidth(d),0.0001);var alpha=1.-smoothstep(-aa,0.,d);
    if v.glint_uv.y>0.{alpha*=smoothstep(-v.glint_uv.y-aa,-v.glint_uv.y+aa,d);}
    var color=v.color;color.a*=alpha;return color;
}
