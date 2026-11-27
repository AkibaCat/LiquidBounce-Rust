#!/usr/bin/env python3
"""Reuse the frozen independent cumulative native verifier with ItemScroller appended."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
source=(HERE/'test-auto-break-native.py').read_text(encoding='utf8')
before="'inventory-manager','better-inventory','auto-break']:"
assert source.count(before)==1
source=source.replace(before,"'inventory-manager','better-inventory','auto-break','item-scroller','gui-closer']:")
before="'auto-break':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_auto_break_key(')}"
assert source.count(before)==1
source=source.replace(before,"'auto-break':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_auto_break_key('), 'item-scroller':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_item_scroller_query('), 'gui-closer':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_gui_closer_screen(')}")
source=source.replace('auto-break-full-api-gen','gui-closer-full-api-gen').replace('transformations plus AutoBreak','transformations plus GUICloser').replace('exact Game helper, screen lifecycle observer, local/remote input seams and generated event ABI','exact prospective container title, pre-lifecycle screen cancellation, Text notification and generated event ABI')
exec(compile(source,str(HERE/'test-auto-break-native.py'),'exec'),{'__file__':str(HERE/'test-auto-break-native.py'),'__name__':'__main__'})
