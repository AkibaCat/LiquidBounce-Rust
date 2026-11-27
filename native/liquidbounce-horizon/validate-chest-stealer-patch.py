"""Generate the normal patch into owned scratch, without changing published staging."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
path=HERE/'make-behavior-patch.py'
source=path.read_text(encoding='utf8')
if "'chest-stealer-patch.py'" not in source:source=source.replace("'inventory-cleaner-patch.py',","'inventory-cleaner-patch.py', 'chest-stealer-patch.py',")
destination=HERE.parents[1]/'build/chest-stealer-smoke-01/hooks-preview.patch'
source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(destination)!r})')
exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
