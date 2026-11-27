"""Late AntiVoid sendPosition wiring, called after Step rewrites the sender."""

def apply(edit,once):
    def remote(s):
        start=s.index('    fn no_slow_vanilla_send_state(&mut self, input: u8) {')
        end=s.index('    fn creative_stacks(',start)
        body=s[start:end]
        a='        let on_ground=horizon_client::step_bridge::ground(on_ground);'
        body=once(body,a,'        let mut pos=pos;\n        if self.riding.is_none(){self.native_client_anti_void_network(true,&mut pos[1]);}\n'+a)
        a='            let crouching = p.local_crouching || matches!(p.pose, crate::pose::Pose::Crouching);'
        body=once(body,a,'            let p=self.player.as_ref().expect("sendPosition player");\n'+a)
        a='        for (slot, stack) in creative_slots {'
        body=once(body,a,'        if !riding {let mut y=self.player.as_ref().map_or(0.,|p|p.position.y);self.native_client_anti_void_network(false,&mut y);}\n'+a)
        return s[:start]+body+s[end:]
    edit('crates/horizon-client/src/game/remote.rs',remote)
