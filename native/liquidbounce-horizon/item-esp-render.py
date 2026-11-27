"""Independent plain no-depth ItemESP geometry; effects use shared OutlineGlow."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def source():
 return (HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('ModuleBreadcrumbs','ModuleItemESP').replace('BreadcrumbRenderer','ItemEspRenderer')
