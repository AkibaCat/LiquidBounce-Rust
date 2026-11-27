// Minecraft entity_sobel.fsh, bound to the isolated StorageESP mask.
#version 330
uniform sampler2D InSampler;
layout(location=0) in vec2 texCoord;
layout(location=0) out vec4 fragColor;
void main(){vec2 texel=1.0/vec2(textureSize(InSampler,0));vec4 c=texture(InSampler,texCoord);vec4 l=texture(InSampler,texCoord-vec2(texel.x,0));vec4 r=texture(InSampler,texCoord+vec2(texel.x,0));vec4 u=texture(InSampler,texCoord-vec2(0,texel.y));vec4 d=texture(InSampler,texCoord+vec2(0,texel.y));float a=clamp(abs(c.a-l.a)+abs(c.a-r.a)+abs(c.a-u.a)+abs(c.a-d.a),0.0,1.0);fragColor=vec4((c.rgb*c.a+l.rgb*l.a+r.rgb*r.a+u.rgb*u.a+d.rgb*d.a)*0.2,a);}
