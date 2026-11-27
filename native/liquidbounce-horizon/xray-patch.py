"""Native XRay mesh snapshot, visibility graph, alpha and real renderer reload."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client-render/src/xray.rs',lambda _:(HERE/'xray-render.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/lib.rs',lambda t:once(t,'pub mod mesher;','pub mod mesher;\npub mod xray;'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'weather, widgets, world, worldborder, zip,','weather, widgets, world, worldborder, xray, zip,'))
 edit('crates/horizon-client/src/game/world_update.rs',lambda t:once(t,'        let enabled = self.far_terrain_choice().0 > 0','        let enabled = horizon_client::xray::current().is_none() && self.far_terrain_choice().0 > 0'))
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,a,(HERE/'xray-host.rs').read_text(encoding='utf8')+'\n'+a))
 def mesh(t):
  t=once(t,'        out.clear();\n        let center = region[1][1].expect("centre chunk");','        out.clear();\n        let xray=crate::xray::current(); // immutable per-build snapshot\n        let center = region[1][1].expect("centre chunk");')
  t=once(t,'        out.visibility = visibility_rows(&open);','        out.visibility = if xray.is_some(){ALL_VISIBLE}else{visibility_rows(&open)};')
  a='        let base_y = center.min_y() + sy * 16;'
  t=once(t,a,a+'''
        let xray_sample=|x:i32,y:i32,z:i32|->Option<u32>{
            let rx=((x+16)>>4) as usize;let rz=((z+16)>>4) as usize;
            region.get(rz)?.get(rx)?.map(|c|c.get(x,base_y+y,z))
        };
        let xray_render=|id:u32,x:i32,y:i32,z:i32|->bool{xray.as_ref().is_some_and(|p|p.render(id,DIRECTIONS.into_iter().filter_map(|(_,d,_)|xray_sample(x+d[0],y+d[1],z+d[2]))))};''')
  t=once(t,'                    if hidden[(y * 16 + z) as usize] >> x & 1 != 0 {','                    if xray.is_none() && hidden[(y * 16 + z) as usize] >> x & 1 != 0 {')
  a='                    let world = BlockPos::new(center.x * 16 + x, base_y + y, center.z * 16 + z);'
  t=once(t,a,'''                    let xray_background=xray.as_ref().is_some_and(|p|p.background(id));
                    if xray.is_some() && !xray_render(id,x,y,z) && !xray_background {continue;}
'''+a)
  t=once(t,'                    let ao = self.smooth_lighting && state.emission == 0 && first.ambient_occlusion;','                    let ao = self.smooth_lighting && state.emission == 0 && first.ambient_occlusion && !xray.as_ref().is_some_and(|p|p.full_bright);')
  a='                        let neighbor = pad.blocks[(cell as isize + OFFSETS[d as usize]) as usize];'
  t=once(t,a,a+'''
                        if let Some(p)=&xray {
                            let other=&info[neighbor as usize];
                            let vanilla=full_faces[neighbor as usize]>>opposite&1==0 && self.visibility.should_render_face(BlockStateId(id),BlockStateId(neighbor),d,true).unwrap_or(true);
                            let face=if xray_background{vanilla}else if !p.selected(id){false}else if p.exposed{!self.visibility.skip_rendering(BlockStateId(id),BlockStateId(neighbor),d,true).unwrap_or(false)}else{true};
                            visible[d as usize]=face || (xray_render(id,x,y,z) && (other.face[opposite as usize]!=1 || !p.same(id,neighbor) || !other.solid || !xray_render(neighbor,x+dx,y+dy,z+dz)));
                            continue;
                        }''')
  t=once(t,'                                interior_leaf,\n                            );','                                interior_leaf,\n                                xray.as_ref().is_some_and(|p|p.full_bright),\n                            );')
  a='''                    if self.shader_pack {
                        out.record_source(
                            start,
                            TerrainSource::new(id, [x, y, z], state.emission, false),'''
  t=once(t,a,'''                    if xray_background {xray.as_ref().unwrap().background_vertices(&mut out,start);}
'''+a)
  t=once(t,'        interior_leaf: bool,\n    ) {','        interior_leaf: bool,\n        xray_full_bright: bool,\n    ) {')
  t=once(t,'        let ao = match q.ambient_occlusion {','        let ao = !xray_full_bright && match q.ambient_occlusion {')
  return t
 edit('crates/horizon-client-render/src/mesher.rs',mesh)
 def shader(t):
  # Keep the 24-byte vertex layout and five wire locations. d.w was unused.
  t=once(t,'    emissive_layer: u32,','    emissive_layer: u32,\n    native_opacity: f32,')
  t=once(t,'    w.d = vec4(o.spherical, o.cylindrical, o.height, 0.0);','    w.d = vec4(o.spherical, o.cylindrical, o.height, o.native_opacity);')
  t=once(t,'    o.height = w.d.z;','    o.height = w.d.z;\n    o.native_opacity = w.d.w;')
  t=once(t,'    out.emission = 15u - (u32(round(color.a * 255.0)) & 15u);','    out.emission = 15u - (u32(round(color.a * 255.0)) & 15u);\n    out.native_opacity = color.a;')
  a='@fragment fn fs_translucent(wire: TerrainWire)'
  i=t.index(a);body=t[i:]
  body=once(body,'    var color = texel * in.color;','    var color = texel * in.color;\n    if (in.layer & 0x8000u) != 0u {color.a *= in.native_opacity;}')
  t=t[:i]+body
  t=once(t,'    if sprite(in.uv, in.layer & 0x7fffu).a < 0.6 { discard; }','    if sprite(in.uv, in.layer & 0x7fffu).a * select(1.0,in.native_opacity,(in.layer & 0x8000u)!=0u) < 0.6 { discard; }')
  return t
 edit('crates/horizon-client-render/src/terrain.wgsl',shader)
 edit('crates/horizon-client-render/src/oit_world.wgsl',lambda t:once(t,'    var color = sprite(in.uv, in.layer & 0x7fffu) * in.color;','    var color = sprite(in.uv, in.layer & 0x7fffu) * in.color;\n    if (in.layer & 0x8000u) != 0u {color.a *= in.native_opacity;}'))
 def warp(t):
  i=t.index('@fragment fn fs_warp_translucent');body=t[i:]
  a='    let color = mix(globals.fog_color * vec4(1.0, 1.0, 1.0, p.color.a), p.color, in.visibility);'
  return t[:i]+once(body,a,a.replace('let color','var color')+'\n    if (in.layer & 0x8000u) != 0u {color.a *= in.native_opacity;}')
 edit('crates/horizon-client-render/src/gfx/world_warp_fragment.wgsl',warp)
 return {'hook_names':[],'helper_names':['client_xray_policy','client_xray_blocks']}
