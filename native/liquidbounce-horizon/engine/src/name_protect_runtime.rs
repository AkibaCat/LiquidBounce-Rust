// SPDX-License-Identifier: GPL-3.0-or-later
//! Font readers use an independent immutable matcher: never actor State or Host callbacks.
use crate::runtime::LiquidBounceHooks;
use horizon_client::{mod_api::Host,text::{NameProtection,Span,Text}};
use liquidbounce_behaviors::name_protect::{self as model,NameProtect,Settings};
use std::sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}};
use std::time::Instant;

pub struct NameProtectService {
    pub model:Mutex<NameProtect>,
    running:AtomicBool,
    update_thread:Mutex<Option<std::thread::ThreadId>>,
    clock:Instant,
}
impl Default for NameProtectService {
    fn default()->Self{Self{model:Mutex::new(NameProtect::default()),running:AtomicBool::new(false),update_thread:Mutex::new(None),clock:Instant::now()}}
}
impl NameProtectService {
    pub fn settings(&self)->Settings{self.model.lock().expect("NameProtect").settings.clone()}
    pub fn set_running(&self,running:bool){self.running.store(running,Ordering::Release);}
    pub fn update(&self,user:&str,friends:Vec<(String,String)>,online:Vec<String>){
        *self.update_thread.lock().expect("NameProtect thread")=Some(std::thread::current().id());
        let mut state=self.model.lock().expect("NameProtect");
        let friends=if state.settings.friends{friends}else{vec![]};
        let others=if state.settings.others{online.into_iter().filter(|n|n!=user).collect()}else{vec![]};
        state.update(user,friends,others);
    }
    fn snapshot(&self)->Option<(Arc<model::Instructions>,Settings,bool)>{
        if !self.running.load(Ordering::Acquire){return None;}
        let state=self.model.lock().expect("NameProtect");
        let instructions=state.instructions.clone()?;
        let settings=state.settings.clone();drop(state);
        let cached=self.update_thread.lock().expect("NameProtect thread").as_ref()==Some(&std::thread::current().id());
        Some((instructions,settings,cached))
    }
}
fn push_span(out:&mut Text,text:&str,style:&horizon_client::text::Style){
    if text.is_empty(){return;}
    if let Some(last)=out.spans.last_mut().filter(|s|s.style==*style){last.text.push_str(text);}else{
        out.spans.push(Span{text:text.into(),style:style.clone()});
    }
}
impl NameProtection for NameProtectService {
    fn replace(&self,text:&str)->Option<String>{
        let (instructions,_,cached)=self.snapshot()?;
        model::replace(text,&instructions.find(text,cached))
    }
    fn wrap(&self,text:&Text)->Option<Text>{
        let (instructions,settings,cached)=self.snapshot()?;
        let plain=text.chars().map(|(c,_)|c).collect::<String>();let matches=instructions.find(&plain,cached);
        if matches.is_empty(){return None;}
        let nanos=self.clock.elapsed().as_nanos().min(u64::MAX as u128)as u64;
        let mut out=Text::default();let mut source=0;let mut index=0;let mut skip_until=0;
        for (c,style) in text.chars(){
            let next=source+c.len_utf16();
            if source<skip_until{source=next;continue;}
            while matches.get(index).is_some_and(|r|r.end<=source){index+=1;}
            if let Some(r)=matches.get(index).filter(|r|r.start==source){
                index+=1;
                if !style.bypasses_name_protection {
                    let mut replacement_style=style.clone();
                    replacement_style.color=Some(settings.colors[r.kind.index()].argb(nanos));
                    replacement_style.bypasses_name_protection=false;
                    push_span(&mut out,&r.name,&replacement_style);
                    skip_until=r.end;source=next;continue;
                }
            }
            push_span(&mut out,c.encode_utf8(&mut [0;4]),style);source=next;
        }
        Some(out)
    }
}
impl LiquidBounceHooks {
    pub(crate) fn name_protect_tick(&self,host:&dyn Host){
        let (service,running,friends)={
            let state=self.state();let modules=state.behaviors.borrow();
            (modules.name_protect.clone(),modules.query_running(&state.registry,"NameProtect"),model::friend_mappings(state.friends.names.iter()))
        };
        service.set_running(running);if !running{return;}
        // The native call takes place after every actor borrow has ended.
        let Ok(facts)=serde_json::from_str::<serde_json::Value>(&host.client_name_protect_snapshot())else{return;};
        let Some(user)=facts["user"].as_str()else{return;};
        let online=facts["online"].as_array().into_iter().flatten().filter_map(|p|p["name"].as_str().map(str::to_owned)).collect();
        service.update(user,friends,online);
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use horizon_client::text::Style;
    #[test]fn spans_keep_start_style_match_across_spans_and_utf16(){
        let service=NameProtectService::default();service.set_running(true);
        service.model.lock().unwrap().settings.replacement="😀我".into();service.update("Alice",vec![],vec![]);
        let start=Style{bold:Some(true),insertion:Some("keep".into()),..Style::color(0xffffff)};
        let tail=Style{italic:Some(true),..Style::color(0x123456)};
        let text=Text::literal("😀").append(Text::styled("Ali",start.clone())).append(Text::styled("ce!",tail.clone()));
        let result=service.wrap(&text).unwrap();assert_eq!(result.plain(),"😀😀我!");
        assert_eq!(result.spans[1].style.bold,Some(true));assert_eq!(result.spans[1].style.insertion.as_deref(),Some("keep"));
        assert_eq!(result.spans[1].style.color,Some(0x32ffb348));assert_eq!(result.spans.last().unwrap().style,tail);
    }
    #[test]fn bypass_only_at_start_and_plain_string_replacement_has_no_style(){
        let service=NameProtectService::default();service.set_running(true);service.update("Alice",vec![],vec![]);
        let bypass=Style{bypasses_name_protection:true,..Style::color(0xffffff)};
        let input=Text::styled("Ali",bypass).append("ce Alice");
        assert_eq!(service.wrap(&input).unwrap().plain(),"Alice You");
        assert_eq!(service.replace("Alice").as_deref(),Some("You"));
        service.set_running(false);assert!(service.wrap(&input).is_none());assert!(service.replace("Alice").is_none());
        service.set_running(true);assert_eq!(service.replace("Alice").as_deref(),Some("You"));
    }
    #[test]fn replacement_colors_are_live_with_a_cached_match(){
        let service=NameProtectService::default();service.set_running(true);service.update("Alice",vec![],vec![]);
        let input=Text::literal("Alice");assert_eq!(service.wrap(&input).unwrap().spans[0].style.color,Some(0x32ffb348));
        service.model.lock().unwrap().settings.colors[0].color=0xff123456;
        assert_eq!(service.wrap(&input).unwrap().spans[0].style.color,Some(0xff123456));
        assert!(service.wrap(&Text::literal("alice")).is_none());
    }
}
