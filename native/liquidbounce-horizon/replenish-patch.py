"""Observe the resulting real GUI replacement, excluding temporary menu swaps."""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
PATTERN=re.compile(r'\{ (?P<owner>(?:self|game|g|guest)(?:\.game)?)\.client_inventory_screen_replaced\(\); ')
def wrap(text):
    edits=[]
    for m in PATTERN.finditer(text):
        start=m.end();i=start;depth=1;quoted=False;escape=False
        while i<len(text):
            c=text[i]
            if quoted:
                if escape:escape=False
                elif c=='\\':escape=True
                elif c=='"':quoted=False
            elif c=='"':quoted=True
            elif c=='{':depth+=1
            elif c=='}':
                depth-=1
                if depth==0:break
            i+=1
        if depth:raise ValueError('Unterminated observed screen replacement')
        original=text[start:i].rstrip()
        edits.append((start,i,'let replenish_screen = '+original+'; '+m['owner']+'.native_client_replenish_screen(replenish_screen) '))
    for start,end,replacement in reversed(edits):text=text[:start]+replacement+text[end:]
    return text
def apply(edit,once):
    paths=[ROOT/'upstream/Horizon/crates/horizon-client/src/game.rs',*sorted((ROOT/'upstream/Horizon/crates/horizon-client/src/game').rglob('*.rs'))]
    for path in paths:
        if path.name=='native_machines.rs':continue
        original=path.read_text(encoding='utf8')
        if path.name in ('game.rs','native_hooks.rs') or re.search(r'\b(?:self|game|g|guest)(?:\.game)?\.screen\s*=(?![=>])',original):
            edit(path.relative_to(ROOT/'upstream/Horizon').as_posix(),wrap)
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'replenish-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    return {'hook_names':[],'helper_names':[]}
