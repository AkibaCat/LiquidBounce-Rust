// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::world_noop::WorldNoop;

fn serialize(
    module: ModuleKind,
    state: &WorldNoop,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(module, common);
    values.push(t.leaf(
        "Active",
        "BOOLEAN",
        json!(state.active),
        &format!("{}.active", module.key()),
        None,
    ));
    Ok(t.group(module.name(), "TOGGLEABLE", values))
}
macro_rules! serializers {
    ($( $name:ident => $function:ident ),+ $(,)?) => {
        $(
            pub fn $function(
                state: &WorldNoop,
                common: &CommonSettings,
                format: JsonFormat,
                describe: &dyn Fn(&str) -> String,
            ) -> Result<Value, ConfigError> {
                serialize(ModuleKind::$name, state, common, format, describe)
            }
        )+
    };
}
serializers!(
    AutoShop => serialize_auto_shop,
    AutoBuild => serialize_auto_build,
    AutoFarm => serialize_auto_farm,
    CrystalAura => serialize_crystal_aura,
    Fucker => serialize_fucker,
    AutoTrap => serialize_auto_trap,
    BlockTrap => serialize_block_trap,
    Nuker => serialize_nuker,
    TickBase => serialize_tick_base,
    Backtrack => serialize_backtrack,
    Fly => serialize_fly,
    LongJump => serialize_long_jump,
);
fn parse(patch: &mut SettingsPatch, root: &Nodes<'_>, name: &str, wrap: fn(bool) -> SettingChange) {
    leaf(patch, root, "Active", &format!("{name}.Active"), |v, p| {
        parse_bool(v, p).map(wrap)
    });
}
pub(super) fn parse_auto_shop(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "AutoShop", SettingChange::AutoShop)
}
pub(super) fn parse_auto_build(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "AutoBuild", SettingChange::AutoBuild)
}
pub(super) fn parse_auto_farm(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "AutoFarm", SettingChange::AutoFarm)
}
pub(super) fn parse_crystal_aura(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "CrystalAura", SettingChange::CrystalAura)
}
pub(super) fn parse_fucker(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "Fucker", SettingChange::Fucker)
}
pub(super) fn parse_auto_trap(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "AutoTrap", SettingChange::AutoTrap)
}
pub(super) fn parse_block_trap(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "BlockTrap", SettingChange::BlockTrap)
}
pub(super) fn parse_nuker(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "Nuker", SettingChange::Nuker)
}
pub(super) fn parse_tick_base(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "TickBase", SettingChange::TickBase)
}
pub(super) fn parse_backtrack(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "Backtrack", SettingChange::Backtrack)
}
pub(super) fn parse_fly(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "Fly", SettingChange::Fly)
}
pub(super) fn parse_long_jump(p: &mut SettingsPatch, r: &Nodes<'_>) {
    parse(p, r, "LongJump", SettingChange::LongJump)
}
