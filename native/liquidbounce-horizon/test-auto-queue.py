#!/usr/bin/env python3
"""One focused AutoQueue smoke; native patch is checked in memory only."""
from pathlib import Path
import runpy,subprocess,tempfile,ast
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
        if s.count(old)!=1:raise RuntimeError(f'AutoQueue native anchor count {s.count(old)}: {old[:110]}')
        return s.replace(old,new)
    fragment=runpy.run_path(str(HERE/'auto-queue-patch.py'))['apply'](edit,once)
    native=texts['crates/horizon-client/src/game/native_hooks.rs'];remote=texts['crates/horizon-client/src/game/remote.rs']
    assert fragment['hook_names']==['client_auto_queue_message','client_auto_queue_wake']
    assert 'if auto_account_original.is_some(){1}else{0}'in remote and 'self.native_client_auto_queue_message(2,&remote_text(&p.content).plain());'in remote
    compact=''.join(native.split())
    assert 'a.position.y-2.0'in compact and 'self.stack_hover_name(s)'in compact and 'self.border.bounds()'in compact
    raw=method(native,'native_client_auto_queue_raw_use');assert 'UseItem{'in ''.join(raw.split()) and 'remote_use_item('not in raw and 'self.send('not in raw
    use=method(native,'native_client_auto_queue_use');assert 'heights.item_used'not in use and 'remote_use_item(hand)'in use
    plan=runpy.run_path(str(HERE/'integrate-auto-queue.py'))['build_plan']()
    assert 'self.auto_queue_world_change();'in plan.text['native/liquidbounce-horizon/engine/src/runtime.rs']
    scheduler='''mod native_scheduler{use std::cell::Cell;mod winit{pub mod event_loop{#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ControlFlow{Poll,Wait,WaitUntil(std::time::Instant)}}}struct Game{client_auto_queue_wake_at:Cell<Option<i64>>,now:i64,wakes:u32}impl Game{fn native_client_auto_tool_time(&self)->i64{self.now}fn native_client_auto_queue_wake(&mut self){self.wakes+=1;}
METHODS
}pub fn check(){use winit::event_loop::ControlFlow;let mut g=Game{client_auto_queue_wake_at:Cell::new(None),now:10,wakes:0};assert_eq!(g.client_auto_queue_control_flow(ControlFlow::Wait),ControlFlow::Wait);g.native_client_auto_queue_schedule(Some(20));g.client_auto_queue_poll_wake();assert_eq!(g.wakes,0);assert_eq!(g.client_auto_queue_control_flow(ControlFlow::Poll),ControlFlow::Poll);assert!(matches!(g.client_auto_queue_control_flow(ControlFlow::Wait),ControlFlow::WaitUntil(_)));let previous=std::time::Instant::now();assert_eq!(g.client_auto_queue_control_flow(ControlFlow::WaitUntil(previous)),ControlFlow::WaitUntil(previous));g.now=20;g.client_auto_queue_poll_wake();assert_eq!(g.wakes,1);assert_eq!(g.client_auto_queue_wake_at.get(),None);g.client_auto_queue_poll_wake();assert_eq!(g.wakes,1);}}
'''.replace('METHODS','\n'.join(method(native,n)for n in['native_client_auto_queue_schedule','client_auto_queue_poll_wake','client_auto_queue_control_flow']))
    with tempfile.TemporaryDirectory(prefix='liquidbounce-auto-queue-')as temp:
        temp=Path(temp);source=(HERE/'auto-queue-smoke.rs').read_text(encoding='utf8').replace('../liquidbounce-behaviors/src/auto_queue.rs',(ROOT/'native/liquidbounce-behaviors/src/auto_queue.rs').as_posix()).replace('/*NATIVE_SCHEDULER*/',scheduler);fixture=temp/'auto-queue-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temp/'auto-queue-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test',str(fixture),'-o',str(exe)],check=True);subprocess.run([str(exe),'--nocapture'],check=True)
    for file in[HERE/'auto-queue-patch.py',HERE/'integrate-auto-queue.py',ROOT/'scripts/update-auto-queue-localization.py']:ast.parse(file.read_text(encoding='utf8'))
    print('AutoQueue checked:',len(texts),'native files in memory;',len(plan.steps),'shared changes; no shared/staging writes')
if __name__=='__main__':main()
