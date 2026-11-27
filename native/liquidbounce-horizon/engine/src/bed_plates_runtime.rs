// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientBedPlatesEvent;
use liquidbounce_behaviors::bed_plates::{Bed,Notice};
use serde_json::{Value,json};
impl LiquidBounceHooks{
    pub(crate) fn bed_plates(&self,event:&mut ClientBedPlatesEvent){
        let Ok(input)=serde_json::from_str::<Value>(event.input)else{return;};let state=self.state();let notice={let mut b=state.behaviors.borrow_mut();
            if !b.query_running(&state.registry,"BedPlates"){b.bed_plates.disable();return;}
            let m=&mut b.bed_plates;match input["action"].as_str().unwrap_or(""){
                "options"=>{*event.output=m.settings.options().to_string();None},
                "tick"=>{if let Ok(camera)=serde_json::from_value(input["camera"].clone()){let beds=input["beds"].as_array().into_iter().flatten().filter_map(Bed::from_json).collect();m.tick(camera,beds);}None},
                "frame"=>{let armor=serde_json::from_value(input["armor"].clone()).unwrap_or([None;4]);*event.output=m.frame(&armor).to_string();None},
                "position"=>{if let(Ok(old),Ok(raw))=(serde_json::from_value(input["old"].clone()),serde_json::from_value(input["raw"].clone())){m.packet_position(old,raw);}None},
                "key"=>{let eye=serde_json::from_value(input["eye"].clone()).unwrap_or([0.;3]);let beds=serde_json::from_value::<Vec<[i32;3]>>(input["beds"].clone()).unwrap_or_default();m.key(input["key"].as_str().unwrap_or(""),eye,&beds)},
                "worldChange"=>m.world_change(),_=>None,
            }
        };
        if let Some(notice)=notice{self.bed_plates_notice(notice);}
    }
    fn bed_plates_notice(&self,n:Notice){
        let locale=liquidbounce_interop::localization::Locale::from_game_language(&horizon_client::lang::code());
        let message=if locale==liquidbounce_interop::localization::Locale::English{n.english()}else{let mut text=liquidbounce_interop::localization::describe(locale.code(),&format!("liquidbounce.module.bedPlates.notifications.{}",n.kind));for(name,value)in["x","y","z"].into_iter().zip(n.pos){text=text.replace(&format!("{{{name}}}"),&value.to_string());}text};
        #[cfg(feature="webview")]
        self.state().events.borrow_mut().push(("notification".into(),json!({"title":if locale==liquidbounce_interop::localization::Locale::English{"SelfBed-Manual".into()}else{liquidbounce_interop::localization::describe(locale.code(),"liquidbounce.module.bedPlates.notifications.title")},"message":message,"severity":n.severity()})));
    }
}
