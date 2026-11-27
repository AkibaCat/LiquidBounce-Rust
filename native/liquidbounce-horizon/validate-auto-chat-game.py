from pathlib import Path
import ast,json,runpy,shutil,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];out=ROOT/'build/auto-chat-game-static-01'
out.mkdir(parents=True,exist_ok=True)
path=HERE/'make-behavior-patch.py';source=path.read_text(encoding='utf8').replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(out/"hooks-preview.patch")!r})')
exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
folder=out/'localization';folder.mkdir(exist_ok=True)
for code in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{code}.json',folder/f'display_{code}.json')
update=runpy.run_path(str(HERE/'update-auto-chat-game-locales.py'))['update'];update(folder);before=[p.read_bytes()for p in sorted(folder.glob('*.json'))];update(folder);assert before==[p.read_bytes()for p in sorted(folder.glob('*.json'))]
for code in ['zh_cn','zh_tw']:
 text=(folder/f'display_{code}.json').read_text(encoding='utf8');data=json.loads(text);assert '\ufffd'not in text;assert sum(k.startswith('liquidbounce.module.autoChatGame.')for k in data['descriptions'])==12
for p in [HERE/'auto-chat-game-patch.py',HERE/'integrate-auto-chat-game.py',HERE/'integrate-auto-chat-game-wake.py',HERE/'test-auto-chat-game.py',HERE/'update-auto-chat-game-locales.py',Path(__file__)]:ast.parse(p.read_text(encoding='utf8'))
report={'passed':True,'scope':'Accumulated native patch generation including independent idle deadlines; Python AST; UTF-8, twelve descriptions per Chinese locale and idempotent locale updates. No Rust/Cargo or network execution.','patchSha256':hashlib.sha256((out/'hooks-preview.patch').read_bytes()).hexdigest()}
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
