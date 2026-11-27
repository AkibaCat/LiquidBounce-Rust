#!/usr/bin/env python3
"""Recheck the refactored actual title bodies with AutoF5's complete fixture."""
from pathlib import Path
import argparse,hashlib,json,os
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def body(text,signature):
    start=text.index(signature);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--native',required=True);a=p.parse_args();out=(ROOT/a.out).resolve();native=(ROOT/a.native).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    host=(native/'horizon/crates/horizon-client/src/game/native_hooks.rs').read_text(encoding='utf8')
    signatures=['    pub(super) fn native_auto_f5_screen(','    fn auto_f5_menu_title(','    fn client_container_menu_title(','    pub(super) fn client_container_menu_text(']
    helpers='\n'.join(body(host,s) for s in signatures)
    storage=(native/'horizon/crates/horizon-client/src/game/storage.rs').read_text(encoding='utf8')
    storage_helpers='\n'.join(body(storage,s) for s in ['    pub(super) fn auto_f5_storage_menu(', '    pub(super) fn client_gui_closer_storage_text('])
    extra='''
    pub(super) fn gui_closer_fixture_title(&self,next:Screen)->Option<(Option<String>,String)> { self.client_container_menu_title(next) }
'''
    fixture=(HERE/'auto-f5-fixture.rs').read_text(encoding='utf8').replace('__NATIVE_HELPERS__',helpers+extra)
    fixture=fixture.replace('pub fn literal(text:String)->Self{Self(text)}','''pub fn empty()->Self{Self(String::new())}
        pub fn translate_with_fallback(key:&str,_fallback:&str,args:&[Text])->Self{Self::translate(key,args)}
        pub fn literal(text:String)->Self{Self(text)}''')
    fixture=fixture.replace('        fn native_window(&self)->bool{true}','''        fn client_gui_closer_display_title(&self)->Option<(Option<String>,String)>{None}
        fn container_parts(&self)->Vec<((i32,i32,i32),usize)>{self.container.map(|key|vec![(key,self.container_size())]).unwrap_or_default()}
        fn block_name_plain(&self,_key:(i32,i32,i32))->Option<String>{self.storage.as_ref().and_then(|s|s.custom.clone())}
        fn native_window(&self)->bool{true}''')
    fixture=fixture.replace('mod storage{use super::Game;', 'mod storage{use super::Game;use crate::text::Text;')
    fixture+='''
'''
    # Add a prospective-screen test inside the same recording Game module.
    at=fixture.rfind('\n}')
    fixture=fixture[:at]+'''
    #[test] fn prospective_container_titles_are_resolved_before_replacing_old_screen_in_all_languages(){
        for language in [0,1,2] {
            crate::text::language(language);
            let mut game=Game{screen:Screen::Inventory,container:Some((0,0,0)),..Default::default()};
            for custom in [None,Some("Vote Store 😀".to_owned())] {
                game.storage=Some(Storage{entity:BlockEntity::Chest(()),block:Some("minecraft:chest".into()),size:27,rows:Some(3),custom,cart:false});
                game.screen=Screen::Inventory;let expected=context(&game).unwrap();
                game.screen=Screen::Playing;assert!(context(&game).is_none());
                assert_eq!(game.gui_closer_fixture_title(Screen::Inventory),Some((expected.0,expected.1)));
                assert_eq!(game.gui_closer_fixture_title(Screen::Creative),Some((None,String::new())));
                assert!(game.gui_closer_fixture_title(Screen::Chat).is_none());
            }
            game.title=Some(());assert!(context(&game).is_none());
            assert!(game.gui_closer_fixture_title(Screen::Inventory).is_some());
        }
        crate::text::language(0);
    }
'''+fixture[at:]
    (out/'fixture.rs').write_text(fixture,encoding='utf8')
    source=(HERE/'test-auto-f5-patch.py').read_text(encoding='utf8')
    source=source.replace("OUT=ROOT/'build/auto-f5-hook-tests'",'OUT=Path('+repr(str(out))+')')
    source=source.replace("source=(HERE/'auto-f5-fixture.rs').read_text(encoding='utf-8')","source=(OUT/'fixture.rs').read_text(encoding='utf-8')")
    source=source.replace("source=source.replace('__STORAGE_HELPER__',helper['STORAGE'])","source=source.replace('__STORAGE_HELPER__',"+repr(storage_helpers)+")")
    # The fixture's placeholder was already filled from the exact final native body.
    source=source.replace("'fixtureResult':'passed' if result.returncode==0 else 'failed'","'fixtureResult':'passed' if result.returncode==0 else 'failed'")
    os.environ['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    try:exec(compile(source,str(HERE/'test-auto-f5-patch.py'),'exec'),{'__file__':str(HERE/'test-auto-f5-patch.py'),'__name__':'__main__'})
    except SystemExit as result:
        passed=result.code in [0,None]
    else:passed=False
    report={'passed':passed,'scope':'Exact final native_auto_f5_screen, auto_f5_menu_title, client_container_menu_title, auto_f5_storage_menu and original chest_title, unchanged complete AutoF5 fixture plus prospective title checks before old screen replacement. Actual EN/SC/TC MC26.3 language resources, custom titles, remote titles, mounts, creative inventory, all original title keys; real generated Host API. Explicit Game/title state providers; actual game separate.','nativeSnapshot':str(native.relative_to(ROOT)),'actualBodySha256':hashlib.sha256(helpers.encode()).hexdigest(),'result':json.loads((out/'result.json').read_text(encoding='utf8')) if (out/'result.json').exists() else None,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*.rs')}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':passed,'report':str(out/'report.json')}))
if __name__=='__main__':main()
