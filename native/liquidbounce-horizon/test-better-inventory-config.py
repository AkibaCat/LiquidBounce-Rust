#!/usr/bin/env python3
"""Complete configuration codec with explicit registration-only test shim."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def color_fixture(out):
    fixture=out/'color-source';fixture.mkdir()
    root=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    adapter=root/'config/gson/adapter/ColorAdapter.kt';shutil.copyfile(adapter,fixture/'ColorAdapter.kt')
    source=(root/'render/engine/type/Color4b.kt').read_text(encoding='utf-8');start=source.index('        fun fromHex(');end=source.index('\n        /**',start)
    (fixture/'Color4b.kt').write_text('package net.ccbluex.liquidbounce.render.engine.type\ndata class Color4b(val argb:Int) {companion object {fun fullAlpha(rgb:Int)=Color4b(rgb or 0xff000000.toInt())\n'+source[start:end]+'\n}}\n',encoding='utf-8')
    inputs=['-1','4294967295','4294967296','1.5','-1.5','18446744073709551615','9223372036854775808','1e1000','-1e1000','1.2345e5','0.00001','1e-1000','null','true','{}','[]']+[json.dumps(s) for s in ['#4677ff','80123456','+12345','-12345','+1234567','-1234567','12345','#gggggg','##123456','0x123456','-fffff','ffffffff']]
    (out/'color-inputs.json').write_text(json.dumps(inputs),encoding='utf-8')
    (fixture/'Main.kt').write_text('''import com.google.gson.*
import java.io.File
import net.ccbluex.liquidbounce.config.gson.adapter.ColorAdapter
import net.ccbluex.liquidbounce.render.engine.type.Color4b
fun main(args:Array<String>) {
 val gson=GsonBuilder().registerTypeAdapter(Color4b::class.java,ColorAdapter).create()
 val inputs=Gson().fromJson(File(args[0]).readText(),Array<String>::class.java)
 val rows=inputs.map { source -> try {val color=gson.fromJson(JsonParser.parseString(source),Color4b::class.java) ?: error("Value rejects null");mapOf("input" to source,"value" to color.argb)} catch(_:Exception){mapOf("input" to source,"error" to true)} }
 print(gson.toJson(rows))
}
''',encoding='utf-8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';libs=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';stdlib=next(libs.glob('kotlin-stdlib-*.jar'));gson=next(libs.glob('gson-*.jar'));classes=fixture/'classes';classes.mkdir()
    compile_run=subprocess.run([str(java),'-cp',str(libs/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-classpath',str(stdlib)+';'+str(gson),'-d',str(classes)]+[str(p) for p in fixture.glob('*.kt')],cwd=ROOT,capture_output=True,text=True)
    (fixture/'compile.stderr.log').write_text(compile_run.stderr,encoding='utf-8')
    if compile_run.returncode:raise RuntimeError(compile_run.stderr)
    run=subprocess.run([str(java),'-cp',str(classes)+';'+str(stdlib)+';'+str(gson),'MainKt',str(out/'color-inputs.json')],cwd=ROOT,capture_output=True,text=True)
    if run.returncode:raise RuntimeError(run.stderr)
    (out/'color-oracle.json').write_text(run.stdout,encoding='utf-8')
    return {'rows':len(inputs),'unchangedAdapterSha256':hashlib.sha256(adapter.read_bytes()).hexdigest(),'fromHexFunctionSha256':hashlib.sha256(source[start:end].encode()).hexdigest()}
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);args=p.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    color_source=color_fixture(out)
    spec=importlib.util.spec_from_file_location('config_base',HERE/'test-hand-derp.py');base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base);base.behaviors(out)
    lib=out/'behaviors/src/lib.rs';source=lib.read_text(encoding='utf-8')
    if 'pub mod better_inventory;' not in source:lib.write_text(source+'\npub mod better_inventory;\n',encoding='utf-8')
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8');shim='mod better_inventory;' not in config
    if shim:
        def once(before,after):
            nonlocal config
            if config.count(before)!=1:raise ValueError((before,config.count(before)))
            config=config.replace(before,after,1)
        once('mod skin_derp;','mod skin_derp;\n#[path="config/better_inventory.rs"]\nmod better_inventory;\npub use better_inventory::{BetterInventorySetting,serialize_better_inventory};')
        once('    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),','    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),\n    BetterInventory => ("BetterInventory", "Render", "betterInventory"),')
        if config.count('    SkinDerp(SkinDerpSetting),')!=2:raise ValueError('SettingChange/RuntimeRequest anchors')
        config=config.replace('    SkinDerp(SkinDerpSetting),','    SkinDerp(SkinDerpSetting),\n    BetterInventory(BetterInventorySetting),')
        once('        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),','        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),\n        ModuleKind::BetterInventory => better_inventory::parse_better_inventory(&mut patch, &root),')
        once('        SettingChange::SkinDerp(value) => {','        SettingChange::BetterInventory(value) => { return ApplyResult::Runtime(RuntimeRequest::BetterInventory(*value)); }\n        SettingChange::SkinDerp(value) => {')
    (out/'lib.rs').write_text(config,encoding='utf-8');shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
    colors=out/'config/better_inventory.rs'
    with colors.open('a',encoding='utf-8') as f:f.write('''
#[cfg(test)] mod original_color_adapter_oracle {use super::*;
#[test] fn json_tree_color_adapter_matches_original_jvm(){
 let rows:Value=serde_json::from_str(include_str!("../color-oracle.json")).unwrap();
 for row in rows.as_array().unwrap(){let value:Value=serde_json::from_str(row["input"].as_str().unwrap()).unwrap();let actual=parse_color(&value,"Color");if row["error"]==true{assert!(actual.is_err(),"{row}");}else{assert_eq!(actual.unwrap(),row["value"].as_i64().unwrap() as i32,"{row}");}}
}}
''')
    (out/'Cargo.toml').write_text('[package]\nname="better-inventory-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/better-inventory-target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:run=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'registrationShim':shim,'colorSource':color_source,'commands':commands,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*.rs')},'scope':'Complete production config plus behavior crate, test-only BetterInventory registration if absent. All 14 leaves, inactive mode color, case-sensitive mode vs Java Unicode insensitive enum, raw floats. Original unchanged ColorAdapter plus exact extracted Color4b.fromHex on the JVM produce the numeric/hex/null oracle through the same Gson JsonElement path as Value.deserializeFrom. No game execution.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
