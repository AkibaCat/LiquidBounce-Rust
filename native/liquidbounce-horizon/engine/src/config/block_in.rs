// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::block_in;

#[derive(Clone, Debug, PartialEq)]
pub enum BlockInSetting { Value(String, Value) }
impl BlockInSetting {
    pub fn apply(self, state: &mut liquidbounce_behaviors::block_in::Settings, blocks: &[String]) -> Result<(), ConfigError> {
        let Self::Value(path,value)=self; let p=format!("BlockIn.{path}");
        match path.as_str() {
            "DisableOn"=>state.disable_on=parse_multi_mask(&value,&p,&block_in::DISABLE_ON)?,
            "AvoidBrokenBlocks"=>state.avoid_broken_blocks=parse_bool(&value,&p)?,
            "BrokenBlockWindow"=>state.broken_block_window=parse_int(&value,&p)?,
            "PlaceOrder"=>{let tag=scalar_string(&value).ok_or_else(||ConfigError::new(&p,"expected choice"))?;if ![block_in::ORDER_NORMAL,block_in::ORDER_RANDOM,block_in::ORDER_BOTTOM_TOP,block_in::ORDER_TOP_BOTTOM].iter().any(|x|x.eq_ignore_ascii_case(&tag)){return Err(ConfigError::new(&p,"unknown place order"));}state.place_order=tag;},
            "Filter"=>state.whitelist=scalar_string(&value).is_some_and(|v|v.eq_ignore_ascii_case("Whitelist")),
            "Blocks"=>{let a=value.as_array().ok_or_else(||ConfigError::new(&p,"expected blocks array"))?;let mut out=vec![];for item in a{let id=scalar_string(item).ok_or_else(||ConfigError::new(&p,"expected block id"))?;let id=if id.contains(':'){id}else{format!("minecraft:{id}")};if !blocks.is_empty()&&!blocks.contains(&id){return Err(ConfigError::new(&p,"unknown block id"));}if !out.contains(&id){out.push(id);}}state.blocks=out;},
            _=>return Err(ConfigError::new(&p,"unknown BlockIn setting")),
        } Ok(())
    }
}
pub fn serialize_block_in(state:&liquidbounce_behaviors::block_in::Settings,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{let t=Tree{format,describe};let mut values=t.common(ModuleKind::BlockIn,common);let mut d=t.leaf("DisableOn","MULTI_CHOOSE",json!(block_in::DISABLE_ON.into_iter().enumerate().filter_map(|(i,n)|(state.disable_on&(1<<i)!=0).then_some(n)).collect::<Vec<_>>()),"liquidbounce.module.blockIn.disableOn",None);if format==JsonFormat::Interop{d["choices"]=json!(block_in::DISABLE_ON);d["canBeNone"]=json!(true);d["isOrderSensitive"]=json!(false);}values.push(d);values.push(t.leaf("AvoidBrokenBlocks","BOOLEAN",json!(state.avoid_broken_blocks),"liquidbounce.module.blockIn.avoidBrokenBlocks",None));values.push(t.ranged("BrokenBlockWindow","INT",json!(state.broken_block_window),json!({"from":0,"to":10000}),"ms","liquidbounce.module.blockIn.brokenBlockWindow"));let mut order=t.leaf("PlaceOrder","CHOOSE",json!(state.place_order),"liquidbounce.module.blockIn.placeOrder",None);if format==JsonFormat::Interop{order["choices"]=json!([block_in::ORDER_NORMAL,block_in::ORDER_RANDOM,block_in::ORDER_BOTTOM_TOP,block_in::ORDER_TOP_BOTTOM]);}values.push(order);values.push(t.leaf("Filter","CHOOSE",json!(if state.whitelist{"Whitelist"}else{"Blacklist"}),"liquidbounce.module.blockIn.filter",None));values.push(t.leaf("Blocks","BLOCK",json!(state.blocks),"liquidbounce.module.blockIn.blocks",None));Ok(t.group("BlockIn","TOGGLEABLE",values))}
pub(super) fn parse_block_in(p:&mut SettingsPatch,root:&Nodes<'_>){for name in ["DisableOn","AvoidBrokenBlocks","BrokenBlockWindow","PlaceOrder","Filter","Blocks"]{if let Some(v)=root.get(name){match value_of(v,&format!("BlockIn.{name}")){Ok(v)=>{let mut s=block_in::Settings::default();match BlockInSetting::Value(name.into(),v.clone()).apply(&mut s,&[]){Ok(())=>p.changes.push(SettingChange::BlockIn(BlockInSetting::Value(name.into(),v.clone()))),Err(e)=>p.issues.push(e)}},Err(e)=>p.issues.push(e)}}}}
