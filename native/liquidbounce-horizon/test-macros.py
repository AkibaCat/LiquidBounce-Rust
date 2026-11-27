#!/usr/bin/env python3
"""One focused model/lexer/scheduler smoke, with native edits in memory only."""
from pathlib import Path
import ast,runpy,subprocess,tempfile
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def method(source,name):
    start=source.index('    pub(super) fn '+name+'(');brace=source.index('{',start);end=brace+1;depth=1
    while depth:depth+=(source[end]=='{')-(source[end]=='}');end+=1
    return source[start:end]
def main():
    texts={};staged=ROOT/'build/staging/horizon'
    def edit(name,f):
        if name not in texts:texts[name]=(staged/name).read_text(encoding='utf8').replace('\r\n','\n')
        texts[name]=f(texts[name])
    def once(s,old,new):
        if s.count(old)!=1:raise RuntimeError(f'Macros native anchor count {s.count(old)}: {old[:100]}')
        return s.replace(old,new)
    if 'client_auto_queue_wake_at'not in(staged/'crates/horizon-client/src/game.rs').read_text(encoding='utf8'):runpy.run_path(str(HERE/'auto-queue-patch.py'))['apply'](edit,once)
    fragment=runpy.run_path(str(HERE/'macros-patch.py'))['apply'](edit,once)
    native=texts['crates/horizon-client/src/game/native_hooks.rs'];chat=texts['crates/horizon-client/src/game/chat.rs']
    assert fragment['hook_names']==['client_macros_wake','client_commands_chat']
    assert 'self.native_client_commands_chat(message,&mut handled)'in chat
    assert 'self.native_client_auto_disable_command(message,'not in chat and 'self.native_client_auto_account_command(message,'not in chat
    send=method(native,'native_client_macros_send');say=method(native,'native_client_commands_say')
    assert 'self.send_command(command)'in send and 'self.send_chat(message)'in send and 'send_command'not in say
    item=method(native,'native_client_macros_item');compact=''.join(item.split())
    assert 'self.pick_entity_hit().is_none()'in item and 'self.pick().or_else('in item
    assert 'self.remote_use_item_on(hand_enum)'in item and 'self.native_client_use_item(hand,yaw,pitch)'in compact and 'g.native_client_auto_queue_use(hand)'in compact
    assert 'client_macros_swung&&before.is_some()'in compact and 'self.creative||before!=after'in compact
    plan=runpy.run_path(str(HERE/'integrate-macros.py'))['build_plan']()
    runtime=plan.text['native/liquidbounce-horizon/engine/src/runtime.rs'];feed=runtime[runtime.index('    fn feed_keyboard('):runtime.index('/// The old launcher')]
    assert feed.count('self.macros_key(host,code);')==1 and feed.index('self.macros_key(host,code);')>feed.index('apply_effects(self, host, effects);')
    assert 'client_command_message(host,event.message)'in runtime
    scheduler='''mod native_scheduler{use std::cell::Cell;mod winit{pub mod event_loop{#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ControlFlow{Poll,Wait,WaitUntil(std::time::Instant)}}}mod mod_api{pub fn mods(){}pub fn dispatch_client_macros_wake(g:&mut super::Game,_:()){g.wakes+=1;}}struct Game{client_macros_wake_at:Cell<Option<i64>>,now:i64,wakes:u32}impl Game{fn native_client_auto_tool_time(&self)->i64{self.now}fn client_auto_tool_player(&self)->bool{true}
METHODS
}pub fn check(){use winit::event_loop::ControlFlow;let mut g=Game{client_macros_wake_at:Cell::new(None),now:10,wakes:0};assert_eq!(g.client_macros_control_flow(ControlFlow::Wait),ControlFlow::Wait);g.native_client_macros_schedule(Some(20));g.client_macros_poll_wake();assert_eq!(g.wakes,0);assert_eq!(g.client_macros_control_flow(ControlFlow::Poll),ControlFlow::Poll);assert!(matches!(g.client_macros_control_flow(ControlFlow::Wait),ControlFlow::WaitUntil(_)));let old=std::time::Instant::now();assert_eq!(g.client_macros_control_flow(ControlFlow::WaitUntil(old)),ControlFlow::WaitUntil(old));g.now=20;g.client_macros_poll_wake();assert_eq!(g.wakes,1);assert_eq!(g.client_macros_wake_at.get(),None);g.client_macros_poll_wake();assert_eq!(g.wakes,1);}}
'''.replace('METHODS','\n'.join(method(native,n)for n in ['native_client_macros_schedule','native_client_macros_wake','client_macros_poll_wake','client_macros_control_flow']))
    with tempfile.TemporaryDirectory(prefix='liquidbounce-macros-')as temp:
        temp=Path(temp);source=(HERE/'macros-smoke.rs').read_text(encoding='utf8')
        for name in ['auto_queue','macros','client_commands']:source=source.replace('../liquidbounce-behaviors/src/'+name+'.rs',(ROOT/'native/liquidbounce-behaviors/src'/f'{name}.rs').as_posix())
        source=source.replace('/*NATIVE_SCHEDULER*/',scheduler);fixture=temp/'macros-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temp/'macros-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test','-A','dead_code',str(fixture),'-o',str(exe)],check=True);subprocess.run([str(exe),'--nocapture'],check=True)
    for file in [HERE/'macros-patch.py',HERE/'integrate-macros.py',ROOT/'scripts/update-macros-localization.py']:ast.parse(file.read_text(encoding='utf8'))
    strings=runpy.run_path(str(ROOT/'scripts/update-macros-localization.py'));assert len(strings['TEXTS'])==73
    print('Macros checked:',len(texts),'native files in memory;',len(plan.steps),'shared changes across',len(plan.text),'files; no shared/staging writes')
if __name__=='__main__':main()
