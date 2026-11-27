"""Sibling visibility for KillAura's exact dragon-parent living attack tail."""


def apply(edit, once):
    edit('crates/horizon-client/src/game/dragon.rs', lambda text: once(
        text,
        '    fn knockback(&mut self, power: f64, xd: f64, zd: f64, source: HurtSource<\'_>) {',
        '    pub(super) fn knockback(&mut self, power: f64, xd: f64, zd: f64, source: HurtSource<\'_>) {',
    ))

    def mace(text):
        text = once(text,
                    '    fn mace_smash_self(&mut self, target_on_ground: bool) -> Option<f64> {',
                    '    pub(super) fn mace_smash_self(&mut self, target_on_ground: bool) -> Option<f64> {')
        return once(text,
                    '    fn smash_knockback(&mut self, center: Vec3, target: Spared, fall: f64) {',
                    '    pub(super) fn smash_knockback(&mut self, center: Vec3, target: Spared, fall: f64) {')
    edit('crates/horizon-client/src/game/mace.rs', mace)
    return {'hook_names': [], 'helper_names': []}
