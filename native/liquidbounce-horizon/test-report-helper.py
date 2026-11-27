#!/usr/bin/env python3
"""One focused ReportHelper smoke, including actual native identity methods."""
from pathlib import Path
import base64,json,runpy,subprocess,tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
PATTERNS=['report %s','%1$-8.3S/%<b/%<h','%s:%1$s','%1$s %s','%1$S %<B %<H','%% %n %1$s','%5%/%-5%','%1$%/%1$n',
          '%s %s','%2$s','%0$s','%<s','%d','%#s','%0s','%-s','%.s','%','%-n','%.2%','%2147483648s','%1$<s','%1$s/%1$<s']
def function(text,name):
    start=text.index('    pub(super) fn '+name+'(');brace=text.index('{',start);end=brace+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]
def main():
    staged=ROOT/'build/staging/horizon';texts={}
    def edit(name,f):
        if name not in texts:texts[name]=(staged/name).read_text(encoding='utf8').replace('\r\n','\n')if(staged/name).exists()else''
        texts[name]=f(texts[name])
    def once(s,old,new):
        if s.count(old)!=1:raise RuntimeError(f'ReportHelper native anchor count {s.count(old)}: {old[:100]}')
        return s.replace(old,new)
    runpy.run_path(str(HERE/'report-helper-patch.py'))['apply'](edit,once)
    native=texts['crates/horizon-client/src/game/native_hooks.rs']
    close=function(native,'native_client_report_helper_close')
    assert 'ContainerClose('not in close and 'remote_close_local()'in close
    plan=runpy.run_path(str(HERE/'integrate-report-helper.py'))['build_plan']()
    assert 'ReportHelperConfirm' in plan.text['native/liquidbounce-horizon/engine/src/coroutine_ticker.rs']
    with tempfile.TemporaryDirectory(prefix='liquidbounce-report-helper-')as temp:
        temp=Path(temp)
        reference=temp/'FormatReference.java'
        reference.write_text('import java.util.*;public class FormatReference{public static void main(String[] a){for(String p:a){try{System.out.println(Base64.getEncoder().encodeToString(String.format(p,"Player").getBytes(java.nio.charset.StandardCharsets.UTF_8)));}catch(IllegalFormatException e){System.out.println("ERROR");}}}}',encoding='utf8')
        java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
        lines=subprocess.run([str(java),str(reference),*PATTERNS],check=True,stdout=subprocess.PIPE,encoding='utf8').stdout.splitlines()
        assert len(lines)==len(PATTERNS)
        goldens='['+','.join('('+json.dumps(pattern)+','+('None'if line=='ERROR'else'Some('+json.dumps(base64.b64decode(line).decode('utf8'))+')')+')'for pattern,line in zip(PATTERNS,lines))+']'
        methods='\n'.join(function(native,name)for name in ['client_inventory_screen_replaced','client_report_helper_screen_event','native_client_report_helper_screen','native_client_report_helper_current'])
        identity='''mod native_identity {
            use std::cell::{Cell,RefCell};
            #[derive(Clone,Copy,PartialEq,Eq)]enum Screen{Playing,Inventory,Creative}
            struct Game{screen:Screen,client_inventory_screen_serial:Cell<u64>,client_report_helper_next:Cell<u64>,client_report_helper_pending:Cell<Option<u64>>,client_report_helper_current:Cell<Option<u64>>,events:RefCell<Vec<u64>>}
            mod mod_api{pub fn mods(){}pub fn dispatch_client_report_helper_screen(game:&super::Game,_mods:(),token:u64){game.events.borrow_mut().push(token);}}
            impl Game{fn client_auto_tool_player(&self)->bool{true}
            METHODS
            fn replace(&mut self,next:Screen){self.client_inventory_screen_replaced();self.screen=next;}}
            pub fn check(){
                let mut g=Game{screen:Screen::Playing,client_inventory_screen_serial:Cell::new(1),client_report_helper_next:Cell::new(0),client_report_helper_pending:Cell::new(None),client_report_helper_current:Cell::new(None),events:RefCell::new(vec![])};
                g.client_report_helper_screen_event(Screen::Inventory,true);
                assert!(!g.native_client_report_helper_current(1));g.replace(Screen::Inventory);assert!(g.native_client_report_helper_current(1));
                g.client_report_helper_screen_event(Screen::Inventory,false);
                assert!(!g.native_client_report_helper_current(2));assert!(g.native_client_report_helper_current(1));
                g.client_report_helper_screen_event(Screen::Inventory,true);g.replace(Screen::Inventory);
                assert!(!g.native_client_report_helper_current(1));assert!(!g.native_client_report_helper_current(2));assert!(g.native_client_report_helper_current(3));
                g.replace(Screen::Playing);assert!(!g.native_client_report_helper_current(3));
                g.client_report_helper_screen_event(Screen::Creative,true);g.replace(Screen::Creative);assert!(g.native_client_report_helper_current(4));
                assert_eq!(*g.events.borrow(),vec![1,2,3,4]);assert_eq!(g.client_inventory_screen_serial.get(),5);
            }
        }'''.replace('METHODS',methods)
        source=(HERE/'report-helper-smoke.rs').read_text(encoding='utf8').replace('../liquidbounce-behaviors/src/report_helper.rs',(ROOT/'native/liquidbounce-behaviors/src/report_helper.rs').as_posix()).replace('/*FORMAT_GOLDENS*/[]',goldens).replace('/*NATIVE_IDENTITY*/',identity)
        fixture=temp/'report-helper-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temp/'report-helper-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test',str(fixture),'-o',str(exe)],check=True)
        subprocess.run([str(exe),'--nocapture'],check=True)
    print('ReportHelper native extension checked in memory:',len(texts),'files;',len(plan.steps),'shared changes; no shared/staging writes')
if __name__=='__main__':main()
