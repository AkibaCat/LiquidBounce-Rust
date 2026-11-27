// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::item_scroller::{ClickMode, ItemScroller};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemScrollerSetting {
    ClickMode(ClickMode),
    Delay([i32; 2]),
}
impl ItemScrollerSetting {
    pub fn apply(self, state: &mut ItemScroller) {
        match self {
            Self::ClickMode(value) => state.click_mode = value,
            Self::Delay(value) => state.delay = value,
        }
    }
}

pub fn serialize_item_scroller(
    state: &ItemScroller,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::ItemScroller, common);
    let mut mode = tree.leaf("ClickMode", "CHOOSE", json!(state.click_mode.tag()),
        "liquidbounce.module.itemScroller.clickMode", None);
    if format == JsonFormat::Interop { mode["choices"] = json!(["QuickMove"]); }
    values.extend([
        mode,
        tree.ranged("Delay", "INT_RANGE", json!({"from":state.delay[0],"to":state.delay[1]}),
            json!({"from":0,"to":20}), "ticks", "liquidbounce.module.itemScroller.delay"),
    ]);
    Ok(tree.group("ItemScroller", "TOGGLEABLE", values))
}

pub(super) fn parse_item_scroller(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "ClickMode", "ItemScroller.ClickMode", |v, p| {
        scalar_string(v).and_then(|tag| ClickMode::parse(&tag))
            .map(|v| SettingChange::ItemScroller(ItemScrollerSetting::ClickMode(v)))
            .ok_or_else(|| ConfigError::new(p, "unknown choice"))
    });
    leaf(patch, root, "Delay", "ItemScroller.Delay", |v, p| {
        parse_int_pair(v, p).map(|v| SettingChange::ItemScroller(ItemScrollerSetting::Delay(v)))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings { enabled:false,hidden:false,key_bind:json!({"key":"key.keyboard.unknown","action":"Toggle"}) }
    }
    #[test]
    fn source_defaults_and_complete_file_interop_tree() {
        let state=ItemScroller::default();
        assert_eq!(state.click_mode,ClickMode::QuickMove);
        assert_eq!(state.delay,[2,3]);
        assert_eq!(state.last_update,0);
        let file=serialize_item_scroller(&state,&common(),JsonFormat::File,&str::to_owned).unwrap();
        let interop=serialize_item_scroller(&state,&common(),JsonFormat::Interop,&str::to_owned).unwrap();
        for tree in [&file,&interop] {
            let nodes=nodes(tree,"ItemScroller","ItemScroller").unwrap();
            assert_eq!(nodes["ClickMode"]["value"],"QuickMove");
            assert_eq!(nodes["Delay"]["value"],json!({"from":2,"to":3}));
        }
        let interop_nodes=nodes(&interop,"ItemScroller","ItemScroller").unwrap();
        assert_eq!(interop_nodes["ClickMode"]["valueType"],"CHOOSE");
        assert_eq!(interop_nodes["ClickMode"]["choices"],json!(["QuickMove"]));
        assert_eq!(interop_nodes["Delay"]["range"],json!({"from":0,"to":20}));
        assert_eq!(interop_nodes["Delay"]["suffix"],"ticks");
        assert!(nodes(&file,"ItemScroller","ItemScroller").unwrap()["ClickMode"].get("choices").is_none());
    }
    #[test]
    fn choice_uses_original_java_unicode_case_insensitive_lookup() {
        for tag in ["QuickMove","quickmove","QUICKMOVE","QuıckMove","QUİCKMOVE"] {
            assert_eq!(ClickMode::parse(tag),Some(ClickMode::QuickMove),"{tag}");
        }
        assert_eq!(ClickMode::parse(" QUICKMOVE"),None);
        assert_eq!(ClickMode::parse("Other"),None);
    }
    #[test]
    fn imports_keep_raw_ranges_and_do_not_reset_chronometer() {
        for value in [json!(5),json!({"from":-50,"to":100}),json!({"from":20,"to":-20}),json!({"from":4294967297_u64,"to":"-3"})] {
            let root=json!({"name":"ItemScroller","value":[{"name":"Delay","value":value}]});
            let patch=parse_settings(ModuleKind::ItemScroller,&root).unwrap();
            assert!(patch.issues.is_empty(),"{:?}",patch.issues);
            let mut state=ItemScroller { last_update:700,..Default::default() };
            for change in patch.changes {if let SettingChange::ItemScroller(value)=change {value.apply(&mut state);}}
            assert_eq!(state.delay,parse_int_pair(&value,"Delay").unwrap());
            assert_eq!(state.last_update,700);
        }
    }
    #[test]
    fn bad_choice_does_not_discard_independent_delay_leaf() {
        let root=json!({"name":"ItemScroller","value":[{"name":"ClickMode","value":"bad"},{"name":"Delay","value":0}]});
        let patch=parse_settings(ModuleKind::ItemScroller,&root).unwrap();
        assert_eq!(patch.issues.len(),1);
        assert!(patch.changes.iter().any(|change|matches!(change,SettingChange::ItemScroller(ItemScrollerSetting::Delay([0,0])))));
    }
}
