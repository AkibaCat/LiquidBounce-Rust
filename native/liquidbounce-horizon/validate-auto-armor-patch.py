"""Generate the normal patch into owned scratch, without changing staging."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
path=HERE/'make-behavior-patch.py';source=path.read_text(encoding='utf8')
if "'auto-armor-patch.py'"not in source:source=source.replace("'chest-stealer-patch.py',","'chest-stealer-patch.py', 'auto-armor-patch.py',")
destination=HERE.parents[1]/'build/auto-armor-smoke-01/hooks-preview.patch'
destination.parent.mkdir(parents=True,exist_ok=True)
source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(destination)!r})')
exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
