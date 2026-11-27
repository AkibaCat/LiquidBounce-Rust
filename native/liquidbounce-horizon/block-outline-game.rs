// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::world_primitives::{PrimitiveShape, WorldPrimitives};
impl Game<'_> {
    pub(super) fn client_block_outline_geometry(&self, primitives: &mut WorldPrimitives) {
        let target = self.pick().filter(|_| !self.entity_targeted()).filter(|h| {
            let state = self.world.block(h.pos[0], h.pos[1], h.pos[2]);
            let [min_x, min_z, max_x, max_z] = self.border.geometry().bounds(0.0);
            let [x, _, z] = h.pos.map(f64::from);
            state != self.world.air && x + 1.0 > min_x && x < max_x && z + 1.0 > min_z && z < max_z
        });
        let input=target.map_or_else(||"null".to_owned(),|h|{
   let state=self.world.block(h.pos[0],h.pos[1],h.pos[2]);let facts=self.res.facts().get(state);
   let offset=if facts.outline_offset{self.pack.context.tables.offset(state,h.pos[0],h.pos[2]).unwrap_or([0.;3])}else{[0.;3]};
   let origin=h.pos.map(f64::from);
   let edges=if let Some(edges)=self.collision.cached_shapes().context_outline_edges(state,self.outline_context_kind()){
    edges.iter().map(|e|std::array::from_fn::<_,2,_>(|end|std::array::from_fn::<_,3,_>(|a|origin[a]+offset[a]+e[end*3+a]))).collect::<Vec<_>>()
   }else if let Some(b)=self.context_outline(h.pos,state){
    horizon_client::blocks::cube_edges().iter().map(|e|std::array::from_fn(|end|std::array::from_fn(|a|origin[a]+b[a]+e[end*3+a]*(b[a+3]-b[a])))).collect()
   }else{facts.outline_edges.iter().map(|e|std::array::from_fn(|end|std::array::from_fn(|a|origin[a]+offset[a]+e[end*3+a]))).collect()};
   let side=match h.face{[0,-1,0]=>0,[0,1,0]=>1,[0,0,-1]=>2,[0,0,1]=>3,[-1,0,0]=>4,_=>5};
   serde_json::json!({"boxes":self.outline_boxes(h.pos),"edges":edges,"side":side,"hit":h.point}).to_string()
  });
        let mut geometry = String::new();
        let mut active = false;
        self.native_client_block_outline(&input, &mut geometry, &mut active);
        let Ok(g) = serde_json::from_str::<serde_json::Value>(&geometry) else {
            return;
        };
        let color = g["color"].as_u64().unwrap_or(0) as u32;
        let outline = g["outline"].as_u64().unwrap_or(0) as u32;
        if color >> 24 != 0 {
            if let Ok(quads) = serde_json::from_value::<Vec<[[f64; 3]; 4]>>(g["quads"].clone()) {
                for corners in quads {
                    primitives
                        .push(PrimitiveShape::LiquidBounceQuad {
                            corners,
                            colors: [color; 4],
                        })
                        .through_walls();
                }
            }
        }
        if outline >> 24 != 0 {
            if let Ok(lines) = serde_json::from_value::<Vec<[[f64; 3]; 2]>>(g["lines"].clone()) {
                for [from, to] in lines {
                    primitives.line(from, to, outline, 1.0).through_walls();
                }
            }
        }
    }
}
