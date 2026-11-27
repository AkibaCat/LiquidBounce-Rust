"""Independent Hats mesh/effect renderer and PNG/JPEG image pipe; frozen core unchanged."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def source():
 t=(HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('ModuleBreadcrumbs','ModuleHats').replace('BreadcrumbRenderer','HatsShapeRenderer')
 anchor='    pub effect: Option<Effect>,'
 assert t.count(anchor)==1
 t=t.replace(anchor,anchor+'\n    #[serde(default)]\n    pub image:Option<ImageFrame>,')
 return t+'\n'+(HERE/'hats-image-render.rs').read_text(encoding='utf8')
