#!/usr/bin/env python3
"""Three original authlib YggdrasilSignatureFix consumer phases."""

POLICY='''
// Module state is published by the actual registry, including world guards.
static LIQUIDBOUNCE_YGGDRASIL_POLICY: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(2);
pub fn liquidbounce_yggdrasil_policy(running: bool, force_slim: bool) {
    LIQUIDBOUNCE_YGGDRASIL_POLICY.store(u8::from(running) | (u8::from(force_slim) << 1), std::sync::atomic::Ordering::Release);
}
fn liquidbounce_yggdrasil_running() -> bool {
    LIQUIDBOUNCE_YGGDRASIL_POLICY.load(std::sync::atomic::Ordering::Acquire) & 1 != 0
}
fn liquidbounce_yggdrasil_slim() -> bool {
    LIQUIDBOUNCE_YGGDRASIL_POLICY.load(std::sync::atomic::Ordering::Acquire) & 3 == 3
}
'''

def apply(edit,once):
    def skins(source):
        anchor='pub fn unpack(value: &str, signature: Signature, valid_uris: &[impl AsRef<str>]) -> Textures {\n'
        source=once(source,anchor,POLICY+'\n'+anchor+'    let signature = if liquidbounce_yggdrasil_running() { Signature::Signed } else { signature };\n')
        source=once(source,'        if !allowed {\n','        if !liquidbounce_yggdrasil_running() && !allowed {\n')
        anchor='    out\n}\n\n/// The skin URL of a `minecraft:profile`'
        return once(source,anchor,'''    if liquidbounce_yggdrasil_slim() && out.skin.as_ref().is_some_and(|url|url.contains("127.0.0.1")) {
        out.slim = true;
    }
    out
}

/// The skin URL of a `minecraft:profile`''')
    edit('crates/horizon-client-render/src/skins.rs',skins)
    return {'hook_names':[],'helper_names':[]}
