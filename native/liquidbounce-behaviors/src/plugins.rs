// SPDX-License-Identifier: GPL-3.0-or-later
//! ModulePlugins and its ServerObserver suggestion probe, ae0eb396.
#[path="navigation/java_case_data.rs"]mod case_data;
use std::cmp::Ordering;
pub const TRY_TAGS:[&str;3]=["Slash","ColonSlash","TabComplete"];
fn fold(c:char)->u32{let p=u32::from(c);case_data::SIMPLE_FOLD.binary_search_by_key(&p,|&(a,_)|a).map_or(p,|i|case_data::SIMPLE_FOLD[i].1)}
fn compare(a:&str,b:&str)->Ordering{a.chars().map(fold).cmp(b.chars().map(fold))}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Effect{Command(&'static str),Suggest{id:i32,input:&'static str},Found(Vec<String>),NoneFound,TimeOut,Disable}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
enum Phase{#[default]Idle,Chat{colon:bool,deadline:i64},Delay{colon:bool,deadline:i64},Version{second:bool,id:i32,deadline:i64},Root{id:i32,deadline:i64},Done}
#[derive(Clone,Debug)]
pub struct Plugins{
    pub try_methods:u32,
    pub server_commands:Vec<String>,
    /// Shared ServerObserver capture: null until a tab capture completes.
    pub captured:Option<Vec<String>>,
    phase:Phase,
    names:Vec<String>,
}
impl Default for Plugins{fn default()->Self{Self{try_methods:7,server_commands:Vec::new(),captured:None,phase:Phase::Idle,names:Vec::new()}}}
impl Plugins{
    pub fn cancel(&mut self){self.phase=Phase::Idle;}
    pub fn disconnected(&mut self){self.cancel();self.captured=None;self.server_commands.clear();}
    pub fn start(&mut self,now:i64,random_id:&mut impl FnMut()->i32)->Vec<Effect>{self.next_method(false,false,now,random_id)}
    pub fn active(&self)->bool{!matches!(self.phase,Phase::Idle|Phase::Done)}
    fn deadline(now:i64,delay:i64)->i64{now.saturating_add(delay)}
    fn next_method(&mut self,skip_slash:bool,skip_colon:bool,now:i64,random_id:&mut impl FnMut()->i32)->Vec<Effect>{
        if !skip_slash&&self.try_methods&1!=0{self.phase=Phase::Chat{colon:false,deadline:Self::deadline(now,5000)};return vec![Effect::Command("plugins")];}
        if !skip_colon&&self.try_methods&2!=0{self.phase=Phase::Chat{colon:true,deadline:Self::deadline(now,5000)};return vec![Effect::Command("bukkit:plugins")];}
        if self.try_methods&4!=0{self.captured=None;self.names.clear();return self.version(false,now,random_id);}
        self.phase=Phase::Done;vec![Effect::TimeOut,Effect::Disable]
    }
    fn version(&mut self,second:bool,now:i64,random_id:&mut impl FnMut()->i32)->Vec<Effect>{let id=random_id();self.phase=Phase::Version{second,id,deadline:Self::deadline(now,1000)};vec![Effect::Suggest{id,input:if second{"/bukkit:version "}else{"/version "}}]}
    fn insert(&mut self,value:String){if let Err(index)=self.names.binary_search_by(|name|compare(name,&value)){self.names.insert(index,value);}}
    fn prefixes(commands:&[String])->Vec<String>{commands.iter().filter_map(|v|v.split_once(':').map(|(prefix,_)|prefix.strip_prefix('/').unwrap_or(prefix)).filter(|p|!p.is_empty()).map(str::to_owned)).collect()}
    fn command_prefixes(&mut self,now:i64,random_id:&mut impl FnMut()->i32)->Vec<Effect>{
        let prefixes=Self::prefixes(&self.server_commands);
        if prefixes.is_empty(){let id=random_id();self.phase=Phase::Root{id,deadline:Self::deadline(now,5000)};vec![Effect::Suggest{id,input:"/"}]}
        else{for prefix in prefixes{self.insert(prefix);}self.finish()}
    }
    fn finish(&mut self)->Vec<Effect>{self.phase=Phase::Done;self.captured=Some(self.names.clone());vec![if self.names.is_empty(){Effect::NoneFound}else{Effect::Found(self.names.clone())},Effect::Disable]}
    pub fn game_message(&mut self,received:i64,message:&str)->Vec<Effect>{
        if let Phase::Chat{deadline,..}=self.phase&&received<=deadline&&message.contains("Plugins ("){self.phase=Phase::Done;return vec![Effect::Disable];}Vec::new()
    }
    pub fn suggestions(&mut self,received:i64,id:i32,texts:Vec<String>,now:i64,random_id:&mut impl FnMut()->i32)->Vec<Effect>{
        match self.phase{
            Phase::Version{second,id:expected,deadline}if id==expected&&received<=deadline=>{
                if texts.is_empty()&&!second{return self.version(true,now,random_id);}
                for text in texts{self.insert(text);}self.command_prefixes(now,random_id)
            }
            Phase::Root{id:expected,deadline}if id==expected&&received<=deadline=>{for prefix in Self::prefixes(&texts){self.insert(prefix);}self.finish()}
            _=>Vec::new(),
        }
    }
    pub fn poll(&mut self,now:i64,random_id:&mut impl FnMut()->i32)->Vec<Effect>{
        loop{match self.phase{
            Phase::Chat{colon,deadline}if now>=deadline=>self.phase=Phase::Delay{colon,deadline:Self::deadline(deadline,1000)},
            Phase::Delay{colon,deadline}if now>=deadline=>return self.next_method(true,colon,now,random_id),
            Phase::Version{second:false,deadline,..}if now>=deadline=>return self.version(true,now,random_id),
            Phase::Version{second:true,deadline,..}if now>=deadline=>return self.command_prefixes(now,random_id),
            Phase::Root{deadline,..}if now>=deadline=>return self.finish(),
            _=>return Vec::new(),
        }}
    }
}
pub fn is_anticheat_plugin(name:&str)->bool{["grimac","intave","vulcan","matrix","verus","nocheatplus","aac","aac5","horizon","spartan","kauri","anticheatreloaded","themis","negativity"].contains(&name.to_lowercase().as_str())}
