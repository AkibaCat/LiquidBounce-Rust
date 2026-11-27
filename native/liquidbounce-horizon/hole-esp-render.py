"""Independent plain instance of the already frozen colored quad/line renderer."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def source():
 return (HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('BreadcrumbRenderer','HoleEspRenderer').replace('ModuleBreadcrumbs','ModuleHoleESP')
