// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::plugins::{Plugins,TRY_TAGS};
pub fn serialize_plugins(state:&Plugins,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::Plugins,common);values.push(tree.multi("Try",state.try_methods,&TRY_TAGS,"liquidbounce.module.plugins.try"));Ok(tree.group("Plugins","TOGGLEABLE",values))}
pub(super) fn parse_plugins(patch:&mut SettingsPatch,root:&Nodes<'_>){leaf(patch,root,"Try","Plugins.Try",|v,p|parse_multi_mask(v,p,&TRY_TAGS).map(SettingChange::Plugins));}
