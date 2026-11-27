from pathlib import Path
import runpy,argparse
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod plugins;','pub mod plugins;\npub mod anti_cheat_detect;','source evidence model')
 edit(E+'lib.rs','mod plugins_runtime;','mod plugins_runtime;\n#[cfg(feature="behavior-hooks")]\nmod anti_cheat_detect_runtime;','runtime')
 edit(E+'behaviors.rs','    pub plugins: std::sync::Arc<crate::plugins_runtime::PluginsService>,','    pub plugins: std::sync::Arc<crate::plugins_runtime::PluginsService>,\n    pub anti_cheat_detect:std::sync::Arc<crate::anti_cheat_detect_runtime::AntiCheatDetectService>,','observer state')
 edit(E+'behaviors.rs','            "Plugins",','            "Plugins",\n            "AntiCheatDetect",','registry')
 edit(E+'behaviors.rs','            plugins: Default::default(),','            plugins: Default::default(),\n            anti_cheat_detect:Default::default(),','observer default')
 edit(E+'behaviors.rs','            ModuleCallback::OnEnabled(id) if self.query_ids.get("BookBot") == Some(&id) => {','''            ModuleCallback::OnEnabled(id) if self.query_ids.get("AntiCheatDetect") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AntiCheatDetectEnabled);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("BookBot") == Some(&id) => {''','onEnabled alert')
 edit(E+'config.rs','    Plugins => ("Plugins", "Exploit", "plugins"),','    Plugins => ("Plugins", "Exploit", "plugins"),\n    AntiCheatDetect => ("AntiCheatDetect","Misc","antiCheatDetect"),','kind')
 edit(E+'one_shots_runtime.rs','    PluginsStart,','    PluginsStart,\n    AntiCheatDetectEnabled,','effect')
 edit(E+'one_shots_runtime.rs','            OneShotEffect::PluginsStart => self.plugins_start(host),','            OneShotEffect::PluginsStart => self.plugins_start(host),\n            OneShotEffect::AntiCheatDetectEnabled => self.anti_cheat_detect_enabled(host),','unborrowed host call')
 edit(E+'runtime.rs','            pipeline.add_listener(i32::MAX, modules.plugins.clone());','            pipeline.add_listener(i32::MAX, modules.plugins.clone());\n            pipeline.add_listener(0,modules.anti_cheat_detect.clone());','always-on observer')
 edit(E+'runtime.rs','        self.plugins_tick(host);','        self.plugins_tick(host);\n        self.anti_cheat_detect_tick(host);','completed callback')
 edit(E+'runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {", "    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {\n        if event.dimension.is_none(){self.state().behaviors.borrow().anti_cheat_detect.disconnect();}",'disconnect state')
 edit(E+'packets.rs','    fn incoming(&self, _event: &mut PacketEvent<\'_, Clientbound>) {}','''    fn incoming(&self, _event: &mut PacketEvent<'_, Clientbound>) {}
    /// Accepted final incoming state, before the actual packet handler.
    fn incoming_accepted(&self,_context:PacketContext,_packet:&Clientbound) {}''','accepted incoming observation')
 edit(E+'packets.rs','''        for handler in self.handlers() {
            handler.listener.incoming(event);
        }
''','''        let handlers=self.handlers();
        for handler in &handlers {
            handler.listener.incoming(event);
        }
        if !event.is_cancelled(){for handler in handlers{handler.listener.incoming_accepted(event.context,event.packet);}}
''','genuine final state')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'plugins-patch.py',","'plugins-patch.py', 'anti-cheat-detect-patch.py',",'native chat sink')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print(f'Validated {len(p.steps)} AntiCheatDetect edits')
 if args.apply:p.write();print('Applied AntiCheatDetect')
