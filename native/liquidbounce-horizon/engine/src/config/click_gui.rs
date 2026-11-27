// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::click_gui::{ClickGui, Settings};
#[derive(Clone, Debug, PartialEq)]
pub enum ClickGuiSetting {
    Scale(f32),
    Search(bool),
    Snapping(bool),
    Grid(i32),
    Cache(bool),
    Fps(i32),
    SyncGameFps(bool),
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ClickGuiChange {
    pub scale: bool,
    pub values: bool,
}
impl ClickGuiSetting {
    pub fn apply(self, gui: &mut ClickGui) -> ClickGuiChange {
        let mut change = ClickGuiChange::default();
        match self {
            Self::Scale(v) if gui.settings.scale != v => {
                gui.settings.scale = v;
                change.scale = true;
                change.values = true;
            }
            Self::Search(v) if gui.settings.search_auto_focus != v => {
                gui.settings.search_auto_focus = v;
                change.values = true;
            }
            Self::Snapping(v) if gui.settings.snapping != v => {
                gui.settings.snapping = v;
                change.values = true;
            }
            Self::Grid(v) if gui.settings.grid_size != v => {
                gui.settings.grid_size = v;
                change.values = true;
            }
            Self::Cache(v) if gui.settings.cache != v => {
                gui.settings.cache = v;
                gui.cache_changed();
            }
            Self::Fps(v) if gui.settings.fps != v => {
                gui.settings.fps = v;
                gui.renderer_changed();
            }
            Self::SyncGameFps(v) => gui.settings.sync_game_fps = v,
            _ => {}
        }
        change
    }
}
pub fn serialize_click_gui(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
    refresh_hz: u32,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::ClickGui, c);
    values.push(t.ranged(
        "Scale",
        "FLOAT",
        float_json(s.scale, "ClickGUI.Scale")?,
        json!({"from":0.5,"to":2.}),
        "",
        "liquidbounce.module.clickGui.scale",
    ));
    values.push(t.leaf(
        "SearchBarAutoFocus",
        "BOOLEAN",
        json!(s.search_auto_focus),
        "liquidbounce.module.clickGui.searchBarAutoFocus",
        None,
    ));
    values.push(t.group(
        "Snapping",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.snapping),
                "liquidbounce.module.clickGui.snapping.enabled",
                None,
            ),
            t.ranged(
                "GridSize",
                "INT",
                json!(s.grid_size),
                json!({"from":1,"to":100}),
                "px",
                "liquidbounce.module.clickGui.snapping.gridSize",
            ),
        ],
    ));
    values.push(t.leaf(
        "Cache",
        "BOOLEAN",
        json!(s.cache),
        "liquidbounce.module.clickGui.cache",
        None,
    ));
    values.push(t.group(
        "Renderer",
        "CONFIGURABLE",
        vec![
            t.ranged(
                "Fps",
                "INT",
                json!(s.fps),
                json!({"from":0,"to":refresh_hz.max(1)}),
                "FPS",
                "liquidbounce.module.clickGui.renderer.fps",
            ),
            t.leaf(
                "SyncGameFps",
                "BOOLEAN",
                json!(s.sync_game_fps),
                "liquidbounce.module.clickGui.renderer.syncGameFps",
                None,
            ),
        ],
    ));
    Ok(t.group("ClickGUI", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
    leaf(p, n, name, &format!("ClickGUI.{path}"), |v, at| {
        Ok(SettingChange::ClickGui(match path {
            "Scale" => ClickGuiSetting::Scale(parse_float(v, at)?),
            "SearchBarAutoFocus" => ClickGuiSetting::Search(parse_bool(v, at)?),
            "Snapping.Enabled" => ClickGuiSetting::Snapping(parse_bool(v, at)?),
            "Snapping.GridSize" => ClickGuiSetting::Grid(parse_int(v, at)?),
            "Cache" => ClickGuiSetting::Cache(parse_bool(v, at)?),
            "Renderer.Fps" => ClickGuiSetting::Fps(parse_int(v, at)?),
            "Renderer.SyncGameFps" => ClickGuiSetting::SyncGameFps(parse_bool(v, at)?),
            _ => return Err(ConfigError::new(at, "unknown ClickGUI setting")),
        }))
    });
}
pub(super) fn parse_click_gui(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["Scale", "SearchBarAutoFocus", "Cache"] {
        add(p, root, name, name);
    }
    for (group, names) in [
        ("Snapping", vec!["Enabled", "GridSize"]),
        ("Renderer", vec!["Fps", "SyncGameFps"]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("ClickGUI.{group}")) {
                Ok(n) => {
                    for name in names {
                        add(p, &n, name, &format!("{group}.{name}"));
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_settings_round_trip_through_original_tree_and_parsers(){
  let settings=Settings{scale:1.5,search_auto_focus:false,snapping:false,grid_size:17,cache:false,fps:37,sync_game_fps:false};
  let common=CommonSettings{enabled:false,hidden:true,key_bind:json!({"key":"key.keyboard.right.shift","action":"Toggle","modifiers":[]})};
  for format in [JsonFormat::File,JsonFormat::Interop]{
   let tree=serialize_click_gui(&settings,&common,format,&str::to_owned,165).unwrap();
   let patch=parse_settings(ModuleKind::ClickGui,&tree).unwrap();assert!(patch.issues.is_empty(),"{:?}",patch.issues);
   let mut model=ClickGui::default();
   for change in patch.changes{if let SettingChange::ClickGui(v)=change{v.apply(&mut model);}}
   assert_eq!(model.settings,settings);
  }
 }
 #[test]fn changes_emit_only_required_ui_or_page_work(){
  let mut gui=ClickGui::default();
  let change=ClickGuiSetting::Scale(1.5).apply(&mut gui);assert!(change.scale&&change.values);
  assert!(!ClickGuiSetting::Scale(1.5).apply(&mut gui).values);
  assert!(ClickGuiSetting::Search(false).apply(&mut gui).values);
  ClickGuiSetting::Cache(false).apply(&mut gui);assert!(gui.open_requested);
  ClickGuiSetting::Fps(37).apply(&mut gui);assert!(gui.restart_requested);
  ClickGuiSetting::SyncGameFps(false).apply(&mut gui);assert!(!gui.settings.sync_game_fps);
 }
}
