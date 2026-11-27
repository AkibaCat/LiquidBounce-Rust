#!/usr/bin/env python3
"""HandDerp options and safe, scoped, synchronous reentrant packet sending."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPER_NAMES=['client_hand_derp_hand','client_hand_derp_send']
HOOK_NAMES=[]
HELPERS=(HERE/'hand-derp-host.rs').read_text(encoding='utf-8')

def apply(edit,replace_once):
    def native(text):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        text=replace_once(text,anchor,HELPERS+'\n'+anchor)
        for signature in ['    pub(super) fn native_client_start(&mut self) {\n','    pub(super) fn native_client_tick_pre(&mut self) {\n','    pub(super) fn native_frame(&mut self, elapsed: Duration) {\n']:
            text=replace_once(text,signature,signature+'        self.native_publish_hand_derp_options();\n')
        return text
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    edit('crates/horizon-client/src/game/remote.rs',lambda text:replace_once(text,'    pub(super) fn send(&self, packet: S) {\n','    pub(super) fn send(&self, packet: S) {\n        self.native_publish_hand_derp_options();\n'))
    edit('crates/horizon-client/src/client_lib.rs',lambda text:replace_once(text,'pub mod packet_bridge;\n','pub mod packet_bridge;\npub mod hand_derp_bridge;\n'))
    edit('crates/horizon-client/src/hand_derp_bridge.rs',lambda _:(HERE/'hand-derp-bridge.rs').read_text(encoding='utf-8'))
    def bridge(text):
        text=replace_once(text,'    cancelled: bool,\n','    cancelled: bool,\n    sender: Option<&\'a dyn Fn(P)>,\n')
        text=replace_once(text,'            cancelled: false,\n','            cancelled: false,\n            sender: None,\n')
        text=replace_once(text,'    pub fn cancel(&mut self) {\n','''    /// Nested ordinary packet emission on this exact Session, before the
    /// interrupted listener resumes. No transport lock is held by dispatch.
    pub fn send(&self, packet: P) -> bool {
        if let Some(sender) = self.sender { sender(packet); true } else { false }
    }

    pub fn cancel(&mut self) {
''')
        signature='pub fn outgoing(session: u64, packet: &mut Serverbound, silent: bool) -> bool {\n'
        text=replace_once(text,signature,'''pub fn outgoing(session: u64, packet: &mut Serverbound, silent: bool) -> bool {
    outgoing_impl(session, packet, silent, None)
}

pub fn outgoing_with_sender(session: u64, packet: &mut Serverbound, silent: bool, sender: &dyn Fn(Serverbound)) -> bool {
    outgoing_impl(session, packet, silent, Some(sender))
}

fn outgoing_impl(session: u64, packet: &mut Serverbound, silent: bool, sender: Option<&dyn Fn(Serverbound)>) -> bool {
''')
        start=text.index('fn outgoing_impl(')
        before='    if let Some(interceptor) = snapshot() {\n'
        prefix=text[:start];tail=text[start:]
        tail=tail.replace(before,'    event.sender = sender;\n'+before,1)
        return prefix+tail
    edit('crates/horizon-client/src/packet_bridge.rs',bridge)
    edit('crates/horizon-client/src/net.rs',lambda text:replace_once(text,'        if crate::packet_bridge::outgoing(self.serial, &mut packet, silent) {','        if crate::packet_bridge::outgoing_with_sender(self.serial, &mut packet, silent, &|nested| self.send(nested)) {'))
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
