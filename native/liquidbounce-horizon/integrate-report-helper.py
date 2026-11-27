#!/usr/bin/env python3
"""Check ReportHelper shared wiring; integration owner applies exactly once."""
from pathlib import Path
import argparse,importlib.util
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('report_helper_shared',HERE/'integrate-super-knockback.py')
shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
E='native/liquidbounce-horizon/engine/src/'
def build_plan():
    p=shared.Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_account;\n','pub mod auto_account;\npub mod report_helper;\n','behavior export')
    p.replace(E+'lib.rs','mod auto_account_runtime;\n','mod auto_account_runtime;\n#[cfg(feature="behavior-hooks")]\nmod report_helper_runtime;\n','runtime export')
    b=E+'behaviors.rs'
    p.replace(b,'    pub auto_account: liquidbounce_behaviors::auto_account::AutoAccount,\n','    pub auto_account: liquidbounce_behaviors::auto_account::AutoAccount,\n    pub report_helper:liquidbounce_behaviors::report_helper::State,\n','state')
    p.replace(b,'            "AutoAccount",\n','            "AutoAccount",\n            "ReportHelper",\n','registration')
    p.replace(b,'            auto_account: Default::default(),\n','            auto_account: Default::default(),\n            report_helper:Default::default(),\n','defaults')
    anchor='            ModuleCallback::CancelSequences(id)\n                if self.query_ids.get("AutoAccount") == Some(&id) =>\n'
    p.replace(b,anchor,'''            ModuleCallback::OnDisabled(id) if self.query_ids.get("ReportHelper")==Some(&id)=>{self.report_helper.disabled();}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("ReportHelper")==Some(&id)=>{self.report_helper.cancel();}
'''+anchor,'lifecycle')
    c=E+'config.rs'
    anchor='pub use auto_account::{AutoAccountSetting, serialize_auto_account};\n'
    p.replace(c,anchor,anchor+'#[path="config/report_helper.rs"]\nmod report_helper;\npub use report_helper::{ReportHelperSetting,serialize_report_helper};\n','config export')
    p.replace(c,'    AutoAccount => ("AutoAccount", "Misc", "autoAccount"),\n','    AutoAccount => ("AutoAccount", "Misc", "autoAccount"),\n    ReportHelper => ("ReportHelper", "Player", "reportHelper"),\n','category')
    p.replace(c,'    AutoAccount(AutoAccountSetting),\n','    AutoAccount(AutoAccountSetting),\n    ReportHelper(ReportHelperSetting),\n','requests',count=2)
    p.replace(c,'            | ModuleKind::AutoAccount\n','            | ModuleKind::AutoAccount\n            | ModuleKind::ReportHelper\n','dedicated tree')
    p.replace(c,'        ModuleKind::AutoAccount => auto_account::parse_auto_account(&mut patch, &root),\n','        ModuleKind::AutoAccount => auto_account::parse_auto_account(&mut patch, &root),\n        ModuleKind::ReportHelper=>report_helper::parse_report_helper(&mut patch,&root),\n','parse')
    anchor='        SettingChange::AutoAccount(value) => {\n'
    p.replace(c,anchor,'        SettingChange::ReportHelper(value)=>return ApplyResult::Runtime(RuntimeRequest::ReportHelper(value.clone())),\n'+anchor,'runtime request')
    a=E+'actor.rs'
    anchor='            ModuleKind::AutoAccount => {\n'
    p.replace(a,anchor,'            ModuleKind::ReportHelper=>config::serialize_report_helper(&b.report_helper,&common,format,&describe),\n'+anchor,'serialize')
    p.replace(a,' ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"],',' ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"], ModuleKind::ReportHelper=>vec!["AutoReport"],','alias')
    anchor='                ApplyResult::Runtime(RuntimeRequest::AutoAccount(setting)) => {\n'
    p.replace(a,anchor,'''                ApplyResult::Runtime(RuntimeRequest::ReportHelper(setting))=>{
                    if let Err(error)=setting.clone().apply(&mut self.state.behaviors.borrow_mut().report_helper){eprintln!("[LiquidBounce] ReportHelper config leaf skipped: {error}");}
                }
'''+anchor,'actor update')
    p.replace(E+'coroutine_ticker.rs','    AutoAccount,\n','    AutoAccount,\n    ReportHelperReport,\n    ReportHelperConfirm,\n','FIFO owners')
    r=E+'runtime.rs'
    anchor='                WaitOwner::AutoAccount => self.auto_account_callback(host, task.callback),\n'
    p.replace(r,anchor,anchor+'                WaitOwner::ReportHelperReport=>self.report_helper_report_callback(host,in_game,task.callback),\n                WaitOwner::ReportHelperConfirm=>self.report_helper_confirm_callback(host,in_game,task.callback),\n','FIFO callbacks')
    p.section(r,'    fn client_auto_account_message(','    fn client_auto_account_command(','        _host: &mut dyn Host,','        host: &mut dyn Host,','real chat host')
    anchor='        self.auto_account_message(event.source, event.message);\n'
    p.replace(r,anchor,anchor+'        if event.source==0{self.report_helper_message(host,event.message);}\n','ChatReceive source')
    anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_auto_account_command('
    p.replace(r,anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_report_helper_screen(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientReportHelperScreenEvent){self.report_helper_screen(host,event.token);}
'''+anchor,'real ScreenEvent')
    anchor='            modules.combat_cooldown.world_change();\n'
    p.replace(r,anchor,anchor+'            modules.report_helper.disabled();\n','world teardown')
    anchor='            modules.combat_cooldown.disconnect();\n'
    p.replace(r,anchor,anchor+'            modules.report_helper.disabled();\n','disconnect')
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'report-helper-patch.py', 'step-patch.py'):",'native extension')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);mode=parser.add_mutually_exclusive_group();mode.add_argument('--apply',action='store_true');mode.add_argument('--check',action='store_true');args=parser.parse_args();p=build_plan();print(f'Validated {len(p.steps)} ReportHelper changes across {len(p.text)} shared files')
    if args.apply:p.write();print('Applied ReportHelper integration')
    else:print('Check only; no shared writes')
if __name__=='__main__':main()
