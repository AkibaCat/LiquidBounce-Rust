#!/usr/bin/env python3
"""Opaque movable stack identity; component/value equality remains unchanged.

Clone is a bridge/snapshot operation and preserves identity. A game operation
that creates a separate stack calls fresh_copy, just as ItemStack.copy does.
No identity is encoded in NBT or protocol codecs.
"""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
SOURCE=ROOT/'upstream/Horizon'

IDENTITY='''/// Runtime object identity, deliberately absent from value equality and codecs.
#[derive(Clone, Copy)]
pub struct ItemIdentity(u64);
impl Default for ItemIdentity {
    fn default() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self(NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}
impl std::fmt::Debug for ItemIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("ItemIdentity") }
}
impl PartialEq for ItemIdentity { fn eq(&self, _: &Self) -> bool { true } }
impl Eq for ItemIdentity {}
impl ItemStack {
    pub fn object_identity(&self) -> u64 { self.identity.0 }
    pub fn object_identity_hash(&self) -> i32 {
        // JVM identity hashes are process-specific arbitrary final tie breaks.
        // Mix the actual allocation identity, never the slot or memory address.
        let mut value = self.identity.0;
        value ^= value >> 30; value = value.wrapping_mul(0xbf58476d1ce4e5b9);
        value ^= value >> 27; value = value.wrapping_mul(0x94d049bb133111eb);
        ((value ^ (value >> 31)) & 0x7fff_ffff) as i32
    }
    pub fn fresh_copy(&self) -> Self {
        Self { identity: ItemIdentity::default(), ..self.clone() }
    }
}
#[cfg(test)]
mod item_object_identity_tests {
    use super::ItemStack;

    #[test]
    fn item_object_identity_survives_snapshots_and_changes_on_copy() {
        let original = ItemStack::new("minecraft:diamond_sword", 1);
        let id = original.object_identity();
        let snapshot = original.clone();
        let moved = original;
        assert_eq!(snapshot.object_identity(), id);
        assert_eq!(moved.object_identity(), id);
        let copied = moved.fresh_copy();
        assert_ne!(copied.object_identity(), id);
        assert_eq!(copied, moved);
        assert!(copied.same_item_same_components(&moved));
        let encoded = crate::item_nbt::item_to_nbt(&moved);
        assert_eq!(crate::item_nbt::item_to_nbt(&copied), encoded);
        let decoded = crate::item_nbt::item_from_nbt(&encoded).unwrap();
        assert_eq!(decoded, moved);
        assert_ne!(decoded.object_identity(), id);
        assert_eq!(crate::item_nbt::item_to_nbt(&decoded), encoded);
        assert!(!format!("{moved:?}").contains(&format!("ItemIdentity({id})")));
        assert!(copied.object_identity_hash() >= 0);
        assert_eq!(copied.object_identity_hash(), copied.clone().object_identity_hash());
    }
}
'''

def mask_rust(text):
    # Preserve offsets while hiding comments and string/character contents.
    # Raw strings require a matching hash count. A regex can otherwise treat
    # the r" suffix of an ordinary quoted identifier as a new raw string and
    # hide real constructors much later in the file.
    masked=list(text);cursor=0
    while cursor<len(text):
        start=cursor;end=None
        if text.startswith('//',cursor):
            end=text.find('\n',cursor)
            if end<0:end=len(text)
        elif text.startswith('/*',cursor):
            depth=1;end=cursor+2
            while end<len(text) and depth:
                if text.startswith('/*',end):depth+=1;end+=2
                elif text.startswith('*/',end):depth-=1;end+=2
                else:end+=1
        elif text[cursor]=='r' and (raw:=re.match(r'r(#{0,255})"',text[cursor:])):
            terminator='"'+raw.group(1);close=text.find(terminator,cursor+raw.end())
            end=len(text) if close<0 else close+len(terminator)
        elif text[cursor]=='"':
            end=cursor+1
            while end<len(text):
                if text[end]=='\\':end+=2
                elif text[end]=='"':end+=1;break
                else:end+=1
        elif text[cursor]=="'" and (char:=re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'",text[cursor:])):
            end=cursor+char.end()
        if end is None:cursor+=1;continue
        for index in range(start,min(end,len(text))):
            if text[index]!='\n':masked[index]=' '
        cursor=end
    return ''.join(masked)


def top_level_fields(body):
    depth=0;visible=[]
    for character in body:
        if character in '{[(':
            depth+=1;visible.append(' ')
        elif character in '}])':
            depth-=1;visible.append(' ')
        else:visible.append(character if depth==0 else ' ')
    return ''.join(visible)

def constructors(text):
    masked=mask_rust(text);inserts=[]
    for match in re.finditer(r'\bItemStack\s*\{',masked):
        cursor=match.end()
        if not re.match(r'\s*(?:(?:item|count|damage|extra)\s*[:,]|\.\.)',masked[cursor:]):continue
        depth=1;end=cursor
        while depth:
            depth+=(masked[end]=='{')-(masked[end]=='}');end+=1
        body=top_level_fields(masked[cursor:end-1])
        # Struct update inherits its actual object's identity. Explicit copies
        # and splits are changed to fresh_copy below, at the operation itself.
        if re.search(r'\bidentity\s*:|\.\.',body):continue
        inserts.append(cursor)
    for cursor in reversed(inserts):
        text=text[:cursor]+' identity: Default::default(),'+text[cursor:]
    return text

def apply(edit,replace_once):
    # Full literal constructors across production, tests and probe examples.
    # Passing every edit through the cumulative callback preserves prior mods.
    for path in sorted((SOURCE/'crates').rglob('*.rs')):
        original=path.read_text(encoding='utf-8')
        if re.search(r'\bItemStack\s*\{',original):
            edit(path.relative_to(SOURCE).as_posix(),constructors)
    def definition(text):
        return replace_once(text,'pub struct ItemStack {\n','pub struct ItemStack {\n    pub identity: ItemIdentity,\n')+'\n'+IDENTITY
    edit('crates/horizon-items/src/items.rs',definition)
    edit('crates/horizon-items/src/item_data.rs',lambda text:replace_once(text,
        '    pub fn new(item: &str, count: u32) -> Self {\n        Self {\n',
        '    pub fn new(item: &str, count: u32) -> Self {\n        Self {\n            identity: Default::default(),\n'))
    # Core inventory split/copy paths. Remainders retain the original identity.
    copies={
        'crates/horizon-client-render/src/containers.rs':[('..held.clone()','..held.fresh_copy()')],
        'crates/horizon-client/src/game/menu_clicks.rs':[('count: half,\n                    ..s.clone()','count: half,\n                    ..s.fresh_copy()'),('count: 1,\n                    ..c.clone()','count: 1,\n                    ..c.fresh_copy()')],
        'crates/horizon-client/src/game/native_item_container.rs':[('let mut one = stack.clone();','let mut one = stack.fresh_copy();'),('count: put,\n            ..item.clone()','count: put,\n            ..item.fresh_copy()'),('count: take,\n        ..held.clone()','count: take,\n        ..held.fresh_copy()')],
        'crates/horizon-client/src/game/stations.rs':[('count: take,\n                    ..s.clone()','count: take,\n                    ..s.fresh_copy()'),('count: n,\n                        ..c.clone()','count: n,\n                        ..c.fresh_copy()'),('count: n,\n                    ..stack.clone()','count: n,\n                    ..stack.fresh_copy()')],
        'crates/horizon-client/src/menu.rs':[
            ('let mut out = stack.clone();\n    out.count = real;','let mut out = stack.fresh_copy();\n    out.count = real;'),
            ('count,\n                ..map.clone()','count,\n                ..map.fresh_copy()'),
            ('let mut empty = stack.clone();\n            empty.count = 0;','let mut empty = stack.fresh_copy();\n            empty.count = 0;'),
            ('self.containers[PLAYER][slot] = Some(stack.clone());\n                stack.count = 0;','self.containers[PLAYER][slot] = Some(stack.fresh_copy());\n                stack.count = 0;'),
            ('count: 1,\n                        ..stack.clone()','count: 1,\n                        ..stack.fresh_copy()'),
            ('let mut placed = source.clone();','let mut placed = source.fresh_copy();'),
            ('let mut copy = stack.clone();\n                copy.count = Self::max_stack(env, stack);','let mut copy = stack.fresh_copy();\n                copy.count = Self::max_stack(env, stack);'),
        ],
        'crates/horizon-client/src/lan/menu.rs':[
            ('count: moved,\n                    ..stack.clone()','count: moved,\n                    ..stack.fresh_copy()',2),
            ('count: 1,\n                    ..stack.clone()','count: 1,\n                    ..stack.fresh_copy()'),
            ('let mut one = carried.clone();','let mut one = carried.fresh_copy();'),
            ('count: max,\n                        ..stack','count: max,\n                        ..stack.fresh_copy()'),
            ('let mut one = stack.clone();','let mut one = stack.fresh_copy();'),
            ('count: amount,\n                        ..c.clone()','count: amount,\n                        ..c.fresh_copy()'),
            ('count: take,\n                ..s.clone()','count: take,\n                ..s.fresh_copy()'),
            ('let mut taken = other.clone();','let mut taken = other.fresh_copy();'),
            ('count: amount,\n                                ..stack.clone()','count: amount,\n                                ..stack.fresh_copy()'),
            ('count: existing + put,\n                        ..carried.clone()','count: existing + put,\n                        ..carried.fresh_copy()'),
        ],
        'crates/horizon-items/src/bundle.rs':[('let mut part = stack.clone();','let mut part = stack.fresh_copy();')],
        'crates/horizon-client/src/creative.rs':[('let copy = |s: &ItemStack, count: u32| ItemStack { count, ..s.clone() };','let copy = |s: &ItemStack, count: u32| ItemStack { count, ..s.fresh_copy() };')],
        'crates/horizon-client/src/game.rs':[('count: moved,\n                ..stack.clone()','count: moved,\n                ..stack.fresh_copy()')],
    }
    for path,replacements in copies.items():
        def transform(text,replacements=replacements):
            for replacement in replacements:
                before,after=replacement[:2]
                if len(replacement)==2:text=replace_once(text,before,after)
                else:
                    count=replacement[2]
                    if text.count(before)!=count:raise ValueError(f'Expected {count} source occurrences: {before}')
                    text=text.replace(before,after)
            return text
        edit(path,transform)
    return {'hook_names':[],'helper_names':[]}
