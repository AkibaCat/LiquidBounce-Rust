// SPDX-License-Identifier: GPL-3.0-or-later
//! Exact LiquidFiller block/fluid/filter adapter; the pure model is shared with the engine.
use super::{Game, liquid_filler_core as core};
use horizon_core::geometry::{Aabb,Vec3};
use horizon_world::{fluid::FluidKind,level::{classes::Interface,data::{ItemKind,state_flags}}};
use serde_json::{Value,json};
struct World<'a,'g>(&'a mut Game<'g>);
impl core::World for World<'_,'_> {
    fn state(&mut self,p:core::Pos)->Option<core::State>{Some(state(self.0,p))}
    fn scan(&mut self,center:[f64;3],radius:f32)->Vec<(core::Pos,core::State)>{
        let radius=f64::from(radius);
        let lo=center.map(|v|(v-radius).floor() as i32);let hi=center.map(|v|(v+radius).floor() as i32);
        let min=self.0.world.shape.min_y;let max=self.0.world.shape.max_y()-1;
        let mut out=vec![];
        // BlockScanUtils.sectionBasedScan: chunk X, chunk Z, section Y;
        // findBlocksInSection: local Y, local Z, local X (X changes fastest).
        for sx in lo[0]>>4..=hi[0]>>4 {for sz in lo[2]>>4..=hi[2]>>4 {
            for sy in (lo[1]>>4).max(min>>4)..=(hi[1]>>4).min(max>>4) {
                for y in lo[1].max(sy<<4)..=hi[1].min((sy<<4)+15) {
                    for z in lo[2].max(sz<<4)..=hi[2].min((sz<<4)+15) {
                        for x in lo[0].max(sx<<4)..=hi[0].min((sx<<4)+15) {
                            let p=[x,y,z];out.push((p,state(self.0,p)));
                        }
                    }
                }
            }
        }}
        // The scanner appends out-of-height AIR after all real sections.
        // betweenClosed changes X, then Y, then Z.
        for (from,to) in [(lo[1],hi[1].min(min-1)),(lo[1].max(max+1),hi[1])] {
            if from>to{continue;}
            for z in lo[2]..=hi[2]{for y in from..=to{for x in lo[0]..=hi[0]{out.push(([x,y,z],core::State{replaceable:true,..Default::default()}));}}}
        }
        out
    }
    fn unobstructed(&mut self,p:core::Pos)->bool {
        let area=Aabb::new(Vec3::new(p[0]as f64,p[1]as f64,p[2]as f64),Vec3::new(p[0]as f64+1.,p[1]as f64+1.,p[2]as f64+1.));
        self.0.with_level(|_,level,_|!level.building_blocker_boxes().into_iter().any(|b|b.intersects(area)))
    }
    fn next_int(&mut self,bound:usize)->usize{super::liquid_filler_random::with_thread_random(|r|r.next_i32_bound(bound as i32))as usize}
}
pub(super) fn state(g:&Game<'_>,p:core::Pos)->core::State{
    let id=g.world.block(p[0],p[1],p[2]);let b=g.res.behaviours();let facts=b.facts(id);let block=b.data(id);
    let fluid=match facts.fluid.kind(){
        FluidKind::Empty=>core::Fluid::Empty,
        FluidKind::Water|FluidKind::FlowingWater=>core::Fluid::Water,
        FluidKind::Lava|FluidKind::FlowingLava=>core::Fluid::Lava,
        _=>core::Fluid::Other,
    };
    core::State{replaceable:facts.flags&state_flags::REPLACEABLE!=0,fluid,source:facts.fluid.is_source(),absorb_pass:block.implements(Interface::BucketPickup)||matches!(b.table.name(id),"minecraft:kelp"|"minecraft:kelp_plant"|"minecraft:seagrass"|"minecraft:tall_seagrass")}
}
pub(super) fn slots(g:&Game<'_>)->Vec<core::Slot>{
    let b=g.res.behaviours();std::iter::once(40).chain(0..9).filter_map(|index|{
        let s=g.stack_in(index).filter(|s|s.count>0)?;
        let block=b.item_binding(&s.item).filter(|i|!matches!(i.kind,ItemKind::SolidBucket|ItemKind::Other)).map(|i|b.table.name(b.table.default_state(i.block)).to_owned());
        Some(core::Slot{index:index as u8,item:s.item.to_string(),block})
    }).collect()
}
pub(super) fn settings(q:&Value)->Option<core::Settings>{Some(core::Settings{
    place_in:q["placeIn"].as_u64()?as u32,order:core::PlaceOrder::from_tag(q["order"].as_str()?)?,use_sponge:q["useSponge"].as_bool()?,whitelist:q["whitelist"].as_bool()?,blocks:serde_json::from_value(q["blocks"].clone()).ok()?
})}
impl Game<'_>{
    pub(super) fn liquid_filler_query(&mut self,text:&str)->Option<String>{
        let q:Value=serde_json::from_str(text).ok()?;let s=settings(&q["settings"])?;let slots=slots(self);let selected=self.client_hotbar_slot()as u8;
        match q["op"].as_str()?{
            "targets"=>{
                let eye=self.eye_position()?;let range=q["range"].as_f64()?;
                let available=core::slot(&s,&slots,selected,None).is_some();
                let targets=if available{core::targets(&mut World(self),&s,eye,range,core::normal_slot(&s,&slots,selected).is_some(),core::sponge_slot(&s,&slots,selected).is_some())}else{vec![]};
                Some(json!({"available":available,"positions":targets}).to_string())
            }
            "slot"=>{let target=serde_json::from_value::<core::Pos>(q["target"].clone()).ok().map(|p|state(self,p));Some(json!(core::slot(&s,&slots,selected,target)).to_string())}
            _=>None
        }
    }
}
