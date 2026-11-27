"""Transactional shared TargetRenderer wiring. Apply AutoBow ownership wiring first."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod target_renderer;','shared behavior export')
 p.replace(E+'config.rs','#[path = "config/hats.rs"]','#[path="config/target_renderer.rs"]\npub(crate) mod target_renderer;\n#[path = "config/hats.rs"]','shared owner config helper')
 p.replace(E+'lib.rs','mod hats_runtime;','mod hats_runtime;\n#[cfg(feature="behavior-hooks")]\nmod target_renderer_runtime;','runtime')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.replace(E+'runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_target_renderer(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTargetRendererEvent<\'_>){self.target_renderer(host,event);}\n'+anchor,'shared render event')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'particles-patch.py'","'particles-patch.py', 'target-renderer-patch.py'",'native patch')
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--apply',action='store_true');args=parser.parse_args();p=prepare();print(f'Validated {len(p.steps)} TargetRenderer changes across {len(p.text)} shared files')
 if args.apply:p.write();print('Integrated shared TargetRenderer')
