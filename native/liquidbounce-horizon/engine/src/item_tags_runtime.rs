// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientItemTagsEvent;
use liquidbounce_behaviors::item_tags::{Entity,Stack};
use serde_json::{Value,json};
impl LiquidBounceHooks {
    pub(crate) fn item_tags(&self,event:&mut ClientItemTagsEvent){
        let Ok(input)=serde_json::from_str::<Value>(event.input)else{return;};
        let state=self.state();let mut b=state.behaviors.borrow_mut();
        if !b.query_running(&state.registry,"ItemTags"){b.item_tags.clear();return;}
        let model=&mut b.item_tags;
        if input["action"]=="running"{*event.output="true".into();}
        else if input["action"]=="tick"{
            let entities=input["entities"].as_array().into_iter().flatten().filter_map(|e|Some(Entity{
                identity:e["identity"].as_u64()?,position:serde_json::from_value(e["position"].clone()).ok()?,stack:Stack::from_json(&e["stack"])?
            })).collect::<Vec<_>>();
            let Ok(camera)=serde_json::from_value(input["camera"].clone())else{return;};model.tick(camera,&entities);
        }else if input["action"]=="frame"{
            *event.output=json!({"settings":model.settings.draw_settings(),"offset":model.settings.render_offset,"groups":model.groups.iter().map(|g|json!({"entities":g.entities,"stacks":g.stacks.iter().map(Stack::json).collect::<Vec<_>>(),"scale":g.scale})).collect::<Vec<_>>()}).to_string();
        }else if input["action"]=="merge"{
            let stacks=input["stacks"].as_array().into_iter().flatten().filter_map(Stack::from_json).collect::<Vec<_>>();
            *event.output=json!(liquidbounce_behaviors::item_tags::merge(&stacks,model.settings.merge_mode).iter().map(Stack::json).collect::<Vec<_>>()).to_string();
        }
    }
}
