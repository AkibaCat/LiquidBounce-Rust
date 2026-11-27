"""Independent Tracers renderer derived from the frozen, verified ShaderLayer bridge."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def source():
 t=(HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('ModuleBreadcrumbs','ModuleTracers').replace('BreadcrumbRenderer','TracerRenderer')
 old='    pub effect: Option<Effect>,'
 assert t.count(old)==1
 t=t.replace(old,old+'\n    #[serde(default="default_width")]\n    pub line_width:f32,')
 t=t.replace('thread_local! {static PENDING:', 'fn default_width()->f32{1.}\nthread_local! {static PENDING:')
 old='let n = Vec2::new(-d.y, d.x) / screen;';assert t.count(old)==1
 return t.replace(old,'let n = Vec2::new(-d.y, d.x) / screen * frame.line_width;')
