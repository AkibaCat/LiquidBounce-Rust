#!/usr/bin/env python3
"""One narrow test crate for the seven modules in the 197-module batch."""
from __future__ import annotations
import argparse, hashlib, json, os
from pathlib import Path
import re, runpy, subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]

def prepare(out:Path):
    runpy.run_path(str(HERE/'test-debug-focused.py'))['prepare'](out)
    lib=(out/'lib.rs').read_text(encoding='utf8')
    sources=[]
    for name in ['nametags','item_chams','trajectories']:
        source=ROOT/f'native/liquidbounce-behaviors/src/{name}.rs'
        sources.append(source)
        lib+=f'#[path="{source.as_posix()}"]pub mod {name};\n'
    smoke_names=['clutch','scaffold-aim','scaffold-clutch','scaffold-lifecycle','scaffold-planner','scaffold-prediction','scaffold-selection','scaffold-techniques','auto-mob-heal']
    for name in smoke_names:
        source=HERE/f'{name}-smoke.rs';sources.append(source)
        content=source.read_text(encoding='utf8')
        content=re.sub(r'^#!\[[^\n]*\]\s*','',content,flags=re.MULTILINE)
        content=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',lambda m:f'#[path="{(source.parent/m[1]).resolve().as_posix()}"]',content)
        if 'fn main(' in content: content+='\n#[test]fn prepared_smoke(){main();}\n'
        target=out/f'{name}.rs';target.write_text(content,encoding='utf8')
        lib+=f'#[path="{target.name}"]mod {name.replace("-","_")}_smoke;\n'
    # GodBridge/Breezily assertions refer to the production rules through super.
    special=HERE/'scaffold-special-techniques-smoke.rs';sources.append(special)
    rules=ROOT/'native/liquidbounce-behaviors/src/scaffold_special_techniques.rs';sources.append(rules)
    special_content=f'#[path="{rules.as_posix()}"]mod rules;\n'+special.read_text(encoding='utf8')
    (out/'scaffold-special.rs').write_text(special_content,encoding='utf8')
    lib+='#[path="scaffold-special.rs"]mod scaffold_special_smoke;\n'
    trajectories=ROOT/'native/liquidbounce-behaviors/examples/trajectories_smoke.rs';sources.append(trajectories)
    (out/'trajectories-smoke.rs').write_text(trajectories.read_text(encoding='utf8')+'\n#[test]fn prepared_smoke(){main();}\n',encoding='utf8')
    lib+='#[path="trajectories-smoke.rs"]mod trajectories_smoke;\n'
    lib+='''
#[cfg(test)]mod item_chams_smoke {
 use super::item_chams::*;use serde_json::json;
 #[test]fn shield_multiply_override_and_disable_preserve_settings(){
  let mut settings=ItemChams::default();
  settings.values.insert("Shield.Tint".into(),json!(0x80402010i64));
  assert_eq!(settings.shield_tint(0x80ff8040),0x40401004);
  settings.values.insert("Shield.TintMode".into(),json!("Override"));
  assert_eq!(settings.shield_tint(0x80ff8040),0x80402010);
  settings.values.insert("Shield.Enabled".into(),json!(false));
  assert_eq!(settings.shield_tint(0x80ff8040),0x80ff8040);
  let before=settings.values.clone();settings.clear();assert_eq!(settings.values,before);
 }
 #[test]fn shader_parameters_keep_argb_order_uniform_alignment_and_animation_phase(){
  let mut settings=ItemChams::default();settings.values.insert("Lightmap.BlendColor".into(),json!(0x80402010i64));
  settings.values.insert("Effects.Flow.Enabled".into(),json!(true));settings.values.insert("Effects.Glow.Enabled".into(),json!(true));
  let output=settings.render_settings(5.);assert_eq!(output["lightUniform"].as_array().unwrap().len(),20);assert_eq!(output["effectUniform"].as_array().unwrap().len(),52);
  let bits=|field:&str,index:usize|output[field][index].as_u64().unwrap()as u32;
  assert_eq!(f32::from_bits(bits("lightUniform",4)),64./255.);assert_eq!(f32::from_bits(bits("lightUniform",7)),128./255.);
  assert_eq!(bits("effectUniform",1)&5,5);assert_eq!(f32::from_bits(bits("effectUniform",0)),5.);
  assert!((0.0..1.0).contains(&f32::from_bits(bits("effectUniform",44))));
 }
}
'''
    (out/'lib.rs').write_text(lib,encoding='utf8')
    # Keep the cache shared with earlier small config fixtures; no full behavior-suite run.
    report={'scope':'Actual production behavior/config for ItemChams, Nametags, Scaffold, Clutch, AutoMobHeal, Debug and Trajectories only; no GPU or game execution','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources},'nativeCompile':'separate batch report','gameValidation':'not_run'}
    (out/'batch-preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    return report

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');parser.add_argument('--filter');args=parser.parse_args()
    out=(ROOT/args.out).resolve();prepare(out)
    if not args.run:print(f'Batch focused fixture prepared: {out}');return 0
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    command=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]
    if args.filter:command.append(args.filter)
    command+=['--','--test-threads=1']
    result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
    log=out/('test-'+args.filter+'.log'if args.filter else 'test.log');log.write_text(result.stdout+result.stderr,encoding='utf8');print(result.stdout+result.stderr)
    report={'command':command,'exitCode':result.returncode,'passed':result.returncode==0,'log':str(log),'gameValidation':'not_run','upstreamDifferentialValidation':'not_run'}
    (out/('result-'+args.filter+'.json'if args.filter else 'result.json')).write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    return result.returncode
if __name__=='__main__':raise SystemExit(main())
