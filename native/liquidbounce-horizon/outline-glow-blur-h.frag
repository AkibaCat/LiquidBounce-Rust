// MC 26.3 entity_outline_box_blur.fsh: horizontal radius 2, linear sampling.
#version 330
uniform sampler2D InSampler;
layout(location=0) in vec2 texCoord;
layout(location=0) out vec4 fragColor;
vec4 linearPixel(vec2 p){ivec2 sz=textureSize(InSampler,0);ivec2 a=ivec2(floor(p));vec2 t=fract(p);return mix(mix(texelFetch(InSampler,clamp(a,ivec2(0),sz-1),0),texelFetch(InSampler,clamp(a+ivec2(1,0),ivec2(0),sz-1),0),t.x),mix(texelFetch(InSampler,clamp(a+ivec2(0,1),ivec2(0),sz-1),0),texelFetch(InSampler,clamp(a+ivec2(1,1),ivec2(0),sz-1),0),t.x),t.y);}
void main(){vec2 p=texCoord*vec2(textureSize(InSampler,0))-0.5;vec2 step=vec2(1,0);vec4 blurred=vec4(0);for(float a=-1.5;a<=2.0;a+=2.0){blurred+=linearPixel(p+step*a);}blurred+=linearPixel(p+step*2.0)/2.0;fragColor=vec4((blurred/2.5).rgb,blurred.a);}
