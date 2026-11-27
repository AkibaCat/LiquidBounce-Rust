from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'macros-patch.py',","'macros-patch.py', 'auto-chat-game-patch.py',",'main loop wake')
 a='    fn client_macros_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientMacrosWakeEvent){self.macros_wake(host);}'
 p.replace('native/liquidbounce-horizon/engine/src/runtime.rs',a,a+'\n    #[cfg(feature="behavior-hooks")]\n    fn client_auto_chat_game_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientAutoChatGameWakeEvent){self.auto_chat_game_poll(host,false);}','independent wake handler')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print('Validated AutoChatGame main-loop wake')
 if args.apply:p.write()
