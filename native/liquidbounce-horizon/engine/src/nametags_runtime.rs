// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::{mod_api::ClientNametagsEvent,text::{NameProtection,Text,Style}};
use liquidbounce_behaviors::{combine_mobs::Key,nametags::{self,Part},targeting::{Target,VisualCamera}};
use serde_json::{json,Value};
fn word(locale:&str,key:&str,english:&str)->String{let lang=liquidbounce_interop::localization::Locale::from_game_language(locale);if lang==liquidbounce_interop::localization::Locale::English{english.into()}else{liquidbounce_interop::localization::describe(lang.code(),&format!("liquidbounce.module.nametags.messages.{key}"))}}
fn styled(text:impl Into<String>,color:u32,bold:bool)->Text{Text::styled(text,Style{bold:Some(bold),..Style::color(color)})}
fn bracket(text:String,color:u32)->Text{Text::colored("[",0xaaaaaa).append(Text::colored(text,color)).append(Text::colored("]",0xaaaaaa))}
fn name_with_color(value:&Value,color:u32)->Text{let mut component=value.clone();if let Value::Object(map)=&mut component{map.insert("color".into(),json!(format!("#{:06x}",color&0xffffff)));}horizon_client::scoreboard::comp_text(&component).resolve_legacy()}
fn format_text(settings:&nametags::Settings,e:&Value,count:i32,locale:&str,protect:&crate::name_protect_runtime::NameProtectService)->Text{
    let mut text=Text::empty();for part in &settings.parts{let next=match part{
        Part::Distance=>{if e["local"]==true{None}else{let d=e["playerDistance"].as_f64().unwrap_or(0.)as f32;Some(Text::colored(format!("{}m",(d+0.5).floor()as i32),0xaaaaaa))}},
        Part::Ping=>{if e["player"]!=true{None}else{let p=e["ping"].as_i64().unwrap_or(0)as i32;Some(bracket(format!("{p}ms"),if p>200{0xff5555}else if p>100{0xffff55}else{0x55ff55}))}},
        Part::Name=>{let name=e["nameColor"].as_u64().map_or_else(||horizon_client::scoreboard::comp_text(&e["name"]).resolve_legacy(),|color|name_with_color(&e["name"],color as u32));let name=protect.wrap(&name).unwrap_or(name);let mut v=Text::empty();if e["baby"]==true{v.push(word(locale,"baby","Baby "));}v.push(name);if count>1{v.push(styled(format!(" ({count})"),0x55ffff,true));}Some(v)},
        Part::Health=>{let h=e["health"].as_f64().unwrap_or(0.)as f32+if e["scoreboard"]==true{0.}else{e["absorption"].as_f64().unwrap_or(0.)as f32};let h=h as i32;Some(Text::colored(format!("{h} {}",word(locale,"health","HP")),if h>=14{0x55ff55}else if h>=8{0xffff55}else{0xff5555}))},
        Part::GameMode=>{if e["player"]!=true{None}else{e["mode"].as_i64().and_then(|m|{let(key,en,color)=match m{0=>("survival","S",0x55ff55),1=>("creative","C",0xff5555),2=>("adventure","A",0xffff55),3=>("spectator","S",0xaaaaaa),_=>return None};Some(bracket(word(locale,key,en),color))})}},
        Part::BotMark=>(e["bot"]==true).then(||styled(word(locale,"bot","Bot"),0xff5555,true)),
        Part::TotemPops=>{let pops=e["pops"].as_i64().unwrap_or(0)as i32;if e["player"]!=true||pops==0{None}else{Some(Text::colored(format!("{pops} {}",word(locale,if pops==1{"pop"}else{"pops"},if pops==1{"Pop"}else{"Pops"})),0xffaa00))}},
    };if let Some(next)=next{if !text.spans.is_empty(){text.push(" ");}text.push(next);}}text
}
impl LiquidBounceHooks{
    pub(crate) fn nametags(&self,event:&mut ClientNametagsEvent){
        let Ok(input)=serde_json::from_str::<Value>(event.input)else{return;};let locale=horizon_client::lang::code();let mut state=self.state();
        if !state.behaviors.borrow().query_running(&state.registry,"Nametags"){state.behaviors.borrow_mut().nametags.clear();return;}
        let perspective=input["perspective"].as_i64().unwrap_or(0)as i32;
        let free_look={let b=state.behaviors.borrow();b.query_running(&state.registry,"FreeLook")};
        let camera=VisualCamera{raw_first_person:perspective==0,free_cam:false,free_look};
        if input["action"]=="policy"{let b=state.behaviors.borrow();let m=&b.nametags;*event.output=json!({"running":true,"refresh":m.perspective.is_none()||m.locale!=locale||(m.perspective!=Some(perspective)&&state.global_targets.visual.contains(Target::SelfEntity))}).to_string();return;}
        if input["action"]=="vanilla"{
            let Some(snapshot)=crate::targeting::snapshot(&input["target"].to_string())else{return;};let id=input["id"].as_i64().unwrap_or(0)as i32;let info=state.target_info(id,&snapshot);*event.output=json!(!state.global_targets.visual.should_show(&snapshot.entity,info,camera)).to_string();return;
        }
        if input["action"]=="tick"{
            let mut combine=state.behaviors.borrow().combine_mobs.clone();combine.clear_nametags();let combine_running={let b=state.behaviors.borrow();b.query_running(&state.registry,"CombineMobs")};
            let mut accepted=Vec::new();for mut e in input["entities"].as_array().cloned().unwrap_or_default(){
                let Some(snapshot)=crate::targeting::snapshot(&e["target"].to_string())else{continue;};let id=e["id"].as_i64().unwrap_or(0)as i32;let info=state.target_info(id,&snapshot);
                if !snapshot.entity.kind.living||!state.global_targets.visual.should_show(&snapshot.entity,info,camera){continue;}
                let position=serde_json::from_value(e["position"].clone()).unwrap_or([0.;3]);let key=Key::at(e["kind"].as_str().unwrap_or(""),e["baby"]==true,position);
                if combine.track(combine_running,key.clone(),e["group"].as_u64().unwrap_or(3)as u8,true){continue;}
                let b=state.behaviors.borrow();let bot=snapshot.anti_bot.as_ref().is_some_and(|q|b.anti_bot.state.is_bot(b.query_running(&state.registry,"AntiBot"),&q.player,&q.online,q.max_health));
                let color=if bot{Some(0x00aaaa)}else if snapshot.entity.invisible{Some(0xffaa00)}else if e["crouching"]==true{Some(0xaa0000)}else{info.resolved_color()};e["nameColor"]=json!(color);e["bot"]=json!(bot);e["pops"]=json!(e["uuid"].as_str().map_or(0,|u|b.notifier.model.pop_count(u)));accepted.push((e,key));
            }
            let accepted=accepted.into_iter().map(|(e,key)|(e,combine.count(combine_running,&key))).collect();
            let mut b=state.behaviors.borrow_mut();b.combine_mobs.clear_nametags();b.nametags.collect(serde_json::from_value(input["camera"].clone()).unwrap_or([0.;3]),accepted,perspective);b.nametags.locale=locale.clone();
            let settings=b.nametags.settings.clone();let protect=b.name_protect.clone();let selected=b.nametags.selected.clone();
            let mut results=Vec::new();for mut tag in selected{tag.facts["text"]=format_text(&settings,&tag.facts,tag.count,&locale,&protect).component();let enchantments=tag.facts["enchantments"].as_array().cloned().unwrap_or_default();tag.facts["cells"]=json!(enchantments.iter().map(|v|nametags::enchantment_cells(&settings,&mut b.nametags.cache,v)).collect::<Vec<_>>());results.push(tag);}b.nametags.selected=results;
        }
        if input["action"]=="tick"||input["action"]=="frame"{let b=state.behaviors.borrow();*event.output=json!({"settings":b.nametags.settings.draw(),"tags":b.nametags.selected.iter().map(|s|json!({"identity":s.identity,"scale":s.scale,"text":s.facts["text"],"cells":s.facts["cells"],"using":s.facts["using"],"local":s.facts["local"]})).collect::<Vec<_>>()}).to_string();}
    }
}
