// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNameProtect / NameProtectMappings at LiquidBounce ae0eb396.
//! UTF-16 Aho-Corasick matching, longest-first overlap removal and immutable snapshots.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};

#[path = "name_protect_username.rs"]
mod username;
pub use username::pseudonym;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode { Static, Rainbow }
impl ColorMode {
    pub fn tag(self) -> &'static str { match self { Self::Static => "Static", Self::Rainbow => "Rainbow" } }
    pub fn parse(s: &str) -> Option<Self> { match s { "Static" => Some(Self::Static), "Rainbow" => Some(Self::Rainbow), _ => None } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coloring { pub mode: ColorMode, pub color: u32 }
impl Coloring {
    /// ColorUtils.rainbow -> java.awt.Color.HSBtoRGB, including its f32 rounding.
    pub fn argb(self, nanos: u64) -> u32 {
        if self.mode == ColorMode::Static { return self.color; }
        let hue = (nanos as f64 / 10_000_000_000.0) as f32 % 1.0;
        let h = (hue - hue.floor()) * 6.0;
        let f = h - h.floor();
        let q = 1.0 - f;
        let (r,g,b) = match h as i32 {
            0 => (1.0,f,0.0), 1 => (q,1.0,0.0), 2 => (0.0,1.0,f),
            3 => (0.0,q,1.0), 4 => (f,0.0,1.0), _ => (1.0,0.0,q),
        };
        let byte = |v:f32| (v * 255.0 + 0.5) as u32;
        0x32000000 | byte(r) << 16 | byte(g) << 8 | byte(b)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { User, Friend, Other }
impl Kind { pub fn index(self) -> usize { match self { Self::User => 0, Self::Friend => 1, Self::Other => 2 } } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub replacement: String,
    pub friends: bool,
    pub others: bool,
    pub colors: [Coloring;3],
}
impl Default for Settings {
    fn default() -> Self { Self {
        replacement: "You".into(), friends: true, others: false,
        colors: [0x32ffb348,0xff00f1ff,0xff474747].map(|color| Coloring { mode:ColorMode::Static,color }),
    } }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replacement {
    /// Start and exclusive end in Java UTF-16 code units.
    pub start: usize, pub end: usize,
    pub name: Arc<str>, pub kind: Kind,
}
#[derive(Clone)]
struct Mapping { len:usize, name:Arc<str>, kind:Kind }
#[derive(Default)]
struct Node { next:BTreeMap<u16,usize>, failure:usize, emits:Vec<usize> }
#[derive(Default)]
struct Cache { entries:BTreeMap<String,(u64,u64,Arc<Vec<Replacement>>)>, clock:u64 }
pub struct Instructions {
    nodes:Vec<Node>, mappings:Vec<Mapping>, cache:Mutex<Cache>,
}
impl Instructions {
    fn build(mapping:BTreeMap<String,(String,Kind)>) -> Self {
        let mut out=Self{nodes:vec![Node::default()],mappings:Vec::new(),cache:Mutex::new(Cache::default())};
        for (key,(name,kind)) in mapping {
            // Trie.addKeyword ignores null/empty strings; a local empty name must not match everything.
            if key.is_empty(){continue;}
            let units=key.encode_utf16().collect::<Vec<_>>();
            let id=out.mappings.len();out.mappings.push(Mapping{len:units.len(),name:name.into(),kind});
            let mut state=0;
            for c in units {
                let next=if let Some(&n)=out.nodes[state].next.get(&c){n}else{
                    let n=out.nodes.len();out.nodes.push(Node::default());out.nodes[state].next.insert(c,n);n
                };state=next;
            }
            out.nodes[state].emits.push(id);
        }
        let mut queue=VecDeque::new();
        queue.extend(out.nodes[0].next.values().copied());
        while let Some(state)=queue.pop_front(){
            let transitions=out.nodes[state].next.clone();
            for (c,next) in transitions {
                queue.push_back(next);
                let mut failure=out.nodes[state].failure;
                while failure!=0&&!out.nodes[failure].next.contains_key(&c){failure=out.nodes[failure].failure;}
                let target=out.nodes[failure].next.get(&c).copied().unwrap_or(0);
                out.nodes[next].failure=target;
                let emits=out.nodes[target].emits.clone();out.nodes[next].emits.extend(emits);
            }
        }
        out
    }
    pub fn find(&self,text:&str,cached:bool)->Arc<Vec<Replacement>> {
        if cached {
            let mut cache=self.cache.lock().expect("NameProtect cache");
            cache.clock=cache.clock.wrapping_add(1);let clock=cache.clock;
            if let Some((frequency,touched,result))=cache.entries.get_mut(text){
                *frequency=frequency.saturating_add(1);*touched=clock;return result.clone();
            }
        }
        let result=Arc::new(self.uncached(text));
        if cached {
            let mut cache=self.cache.lock().expect("NameProtect cache");
            if cache.entries.len()>=512 {
                let victim=cache.entries.iter().min_by_key(|(_,v)|(v.0,v.1)).map(|(k,_)|k.clone());
                if let Some(victim)=victim{cache.entries.remove(&victim);}
            }
            cache.clock=cache.clock.wrapping_add(1);let clock=cache.clock;
            cache.entries.insert(text.into(),(1,clock,result.clone()));
        }
        result
    }
    fn uncached(&self,text:&str)->Vec<Replacement> {
        let mut state=0;let mut found=Vec::new();
        for (index,c) in text.encode_utf16().enumerate(){
            while state!=0&&!self.nodes[state].next.contains_key(&c){state=self.nodes[state].failure;}
            state=self.nodes[state].next.get(&c).copied().unwrap_or(0);
            for &id in &self.nodes[state].emits {
                let m=&self.mappings[id];let end=index+1;
                found.push(Replacement{start:end-m.len,end,name:m.name.clone(),kind:m.kind});
            }
        }
        // IntervalableComparatorBySize: longest interval first, earlier start breaks ties.
        found.sort_by(|a,b|(b.end-b.start).cmp(&(a.end-a.start)).then(a.start.cmp(&b.start)));
        let mut accepted:BTreeMap<usize,Replacement>=BTreeMap::new();
        for r in found {
            if accepted.range(..=r.start).next_back().is_some_and(|(_,v)|v.end>r.start)
                ||accepted.range(r.start..r.end).next().is_some(){continue;}
            accepted.insert(r.start,r);
        }
        accepted.into_values().collect()
    }
}
#[derive(Default)]
pub struct NameProtect {
    pub settings:Settings,
    user:Option<(String,String)>,friends:BTreeMap<String,String>,others:BTreeSet<String>,
    pub instructions:Option<Arc<Instructions>>,
}
impl NameProtect {
    /// Colors stay live: they never participate in snapshot/cache invalidation.
    pub fn update(&mut self,user:&str,friends:Vec<(String,String)>,others:Vec<String>)->bool {
        let replacement=(user.to_owned(),self.settings.replacement.clone());
        let changed=self.user.as_ref()!=Some(&replacement)
            ||friends.iter().any(|(n,a)|self.friends.get(n)!=Some(a))
            ||friends.len()!=self.friends.len()
            ||others.iter().any(|n|!self.others.contains(n))
            ||others.is_empty()&&others.len()!=self.others.len();
        if !changed{return false;}
        let mut mappings=BTreeMap::new();
        for name in others.iter().take(200).filter(|n|(2..=20).contains(&n.encode_utf16().count())){
            mappings.insert(name.clone(),(pseudonym(name),Kind::Other));
        }
        for (name,alias) in &friends{mappings.insert(name.clone(),(alias.clone(),Kind::Friend));}
        mappings.insert(user.into(),(replacement.1.clone(),Kind::User));
        self.friends=friends.into_iter().collect();self.others=others.into_iter().collect();self.user=Some(replacement);
        self.instructions=Some(Arc::new(Instructions::build(mappings)));true
    }
}
/// FriendManager's TreeSet order is UTF-16; blank names are filtered before default numbering.
pub fn friend_mappings<'a>(friends:impl IntoIterator<Item=(&'a String,&'a Option<String>)>)->Vec<(String,String)> {
    let mut friends=friends.into_iter().filter(|(n,_)|!n.chars().all(kotlin_whitespace)).collect::<Vec<_>>();
    friends.sort_by(|a,b|a.0.encode_utf16().cmp(b.0.encode_utf16()));
    friends.into_iter().enumerate().map(|(i,(n,a))|(n.clone(),a.clone().unwrap_or_else(||format!("Friend {i}")))).collect()
}
fn kotlin_whitespace(c:char)->bool {
    // Char.isWhitespace = Character.isWhitespace || Character.isSpaceChar.
    matches!(c,'\u{0009}'..='\u{000d}'|'\u{001c}'..='\u{0020}'|'\u{0085}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}') && c!='\u{0085}'
}
pub fn replace(original:&str,replacements:&[Replacement])->Option<String> {
    if replacements.is_empty(){return None;}
    let mut bytes=Vec::new();
    for (i,c) in original.char_indices(){bytes.extend(std::iter::repeat_n(i,c.len_utf16()));}
    bytes.push(original.len());
    let mut out=String::with_capacity(original.len());let mut current=0;
    for r in replacements {out.push_str(&original[bytes[current]..bytes[r.start]]);out.push_str(&r.name);current=r.end;}
    out.push_str(&original[bytes[current]..]);Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan(user:&str,alias:&str,friends:Vec<(String,String)>,text:&str)->(NameProtect,Arc<Vec<Replacement>>) {
        let mut state=NameProtect::default();state.settings.replacement=alias.into();state.update(user,friends,vec![]);
        let found=state.instructions.as_ref().unwrap().find(text,true);(state,found)
    }
    #[test] fn replacement_is_case_sensitive_substring_and_never_recursive(){
        let (_,r)=plan("Alice","Bob",vec![("Bob".into(),"C".into())],"Alice alice BobAlice");
        assert_eq!(replace("Alice alice BobAlice",&r).as_deref(),Some("Bob alice CBob"));
    }
    #[test] fn longest_global_overlap_then_earlier_start_and_utf16_offsets(){
        let (_,r)=plan("bcde","中",vec![("abc".into(),"short".into()),("efg".into(),"last".into())],"😀abcdefg");
        assert_eq!(r.iter().map(|m|(m.start,m.end)).collect::<Vec<_>>(),[(3,7)]);
        assert_eq!(replace("😀abcdefg",&r).as_deref(),Some("😀a中fg"));
        let (_,r)=plan("abc","x",vec![("bcd".into(),"y".into())],"abcd");
        assert_eq!(replace("abcd",&r).as_deref(),Some("xd"));
    }
    #[test] fn self_overrides_friend_overrides_other_and_removal_only_rebuilds_when_empty(){
        let mut s=NameProtect::default();s.update("AB",vec![("AB".into(),"friend".into())],vec!["AB".into(),"Other".into()]);
        let old=s.instructions.clone().unwrap();assert!(!s.update("AB",vec![("AB".into(),"friend".into())],vec!["AB".into()]));
        assert!(Arc::ptr_eq(&old,s.instructions.as_ref().unwrap()));
        assert_eq!(replace("AB",&old.find("AB",false)).as_deref(),Some("You"));
        assert!(s.update("AB",vec![("AB".into(),"friend".into())],vec![]));
        assert!(s.instructions.as_ref().unwrap().find("Other",false).is_empty());
    }
    #[test] fn friend_default_indices_utf16_sort_and_empty_alias(){
        let a="\u{10000}".to_string();let b="\u{e000}".to_string();let blank="\u{a0}".to_string();let empty=Some(String::new());let none=None;
        assert_eq!(friend_mappings([(&b,&empty),(&blank,&none),(&a,&none)]),vec![(a,"Friend 0".into()),(b,String::new())]);
    }
    #[test] fn other_first_200_before_length_filter_and_colors_dont_rebuild(){
        let mut s=NameProtect::default();let mut others=vec!["!".into();200];others.push("Never".into());s.update("Self",vec![],others);
        let old=s.instructions.clone().unwrap();assert!(old.find("Never !",true).is_empty());
        s.settings.colors[0].mode=ColorMode::Rainbow;
        assert!(!s.update("Self",vec![],vec!["!".into()]));assert!(Arc::ptr_eq(&old,s.instructions.as_ref().unwrap()));
        assert_eq!(s.settings.colors[0].argb(0),0x32ff0000);
        assert_eq!(s.settings.colors[0].argb(5_000_000_000),0x3200ffff);
    }
    #[test] fn cache_is_snapshot_bound_and_has_512_entries(){
        let mut s=NameProtect::default();s.update("Self",vec![],vec![]);let old=s.instructions.clone().unwrap();
        for i in 0..600{old.find(&format!("Self {i}"),true);}
        assert_eq!(old.cache.lock().unwrap().entries.len(),512);
        s.settings.replacement="Changed".into();s.update("Self",vec![],vec![]);
        assert_eq!(s.instructions.as_ref().unwrap().cache.lock().unwrap().entries.len(),0);
        assert_eq!(replace("Self",&old.find("Self",false)).as_deref(),Some("You"));
    }
}
