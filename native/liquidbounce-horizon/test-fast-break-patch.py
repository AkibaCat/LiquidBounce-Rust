#!/usr/bin/env python3
"""Unmodified FastBreak Kotlin, actual MC tags/packets and native mining/transport."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
UPSTREAM=ROOT/'upstream/Horizon'
def block(text,anchor):
    start=text.index(anchor);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/fast-break-01');args=parser.parse_args()
    out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
    if out.exists():raise SystemExit('Choose a fresh evidence directory')
    out.mkdir(parents=True);records=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (out/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        records.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        if result.returncode:print((result.stdout+result.stderr)[-13000:])
        return result
    module=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/ModuleFastBreak.kt'
    categories=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/item/ItemCategorizations.kt'
    shutil.copyfile(module,out/'ModuleFastBreak.kt')
    original=categories.read_text(encoding='utf-8');helpers='package net.ccbluex.liquidbounce.utils.item\nimport net.minecraft.core.TypedInstance\nimport net.minecraft.world.item.Item\nimport net.minecraft.tags.ItemTags\n'
    for name in ['isPickaxe','isAxe','isShovel','isHoe','isMiningTool']:
        anchor='val TypedInstance<Item>.'+name+'\n';start=original.index(anchor);end=original.index('\n\n',start)
        helpers+=original[start:end]+'\n\n'
    (out/'ItemHelpers.kt').write_text(helpers,encoding='utf-8')
    fixtures={
        'Events.kt':'''package net.ccbluex.liquidbounce.event.events
class GameTickEvent
class PacketEvent(val packet:Any)
''',
        'Handlers.kt':'''package net.ccbluex.liquidbounce.event
fun <E> handler(callback:(E)->Unit):(E)->Unit=callback
''',
        'Modes.kt':'''package net.ccbluex.liquidbounce.config.types.group
open class Mode(val name:String,val aliases:List<String> = emptyList()) {open val parent:ModeValueGroup<Mode> get()=error("parent")}
class ModeValueGroup<T:Mode>(val index:Int) {var values:Array<Mode> = emptyArray()}
class NoneMode(override val parent:ModeValueGroup<Mode>):Mode("None")
''',
        'Module.kt':'''package net.ccbluex.liquidbounce.features.module
import kotlin.reflect.KProperty
import net.ccbluex.liquidbounce.config.types.group.*
import net.minecraft.world.item.ItemStack
import net.minecraft.world.item.Items
class Value<T>(var value:T) {operator fun getValue(receiver:Any?,property:KProperty<*>):T=value}
enum class ModuleCategories {WORLD}
class FixturePlayer {var mainHandItem=ItemStack(Items.AIR)}
class FixtureInteraction {var destroyDelay=0;var destroyProgress=0f}
class FixtureNetwork {val sent=mutableListOf<Any>();fun send(packet:Any){sent.add(packet)}}
open class ClientModule(val name:String,val category:ModuleCategories) {
 val player=FixturePlayer();val interaction=FixtureInteraction();val network=FixtureNetwork()
 val floats=mutableMapOf<String,Value<Float>>();val bools=mutableMapOf<String,Value<Boolean>>()
 fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>)=Value(value).also {floats[name]=it}
 fun boolean(name:String,value:Boolean)=Value(value).also {bools[name]=it}
 fun choices(name:String,index:Int,provider:(ModeValueGroup<Mode>)->Array<Mode>):ModeValueGroup<Mode> = ModeValueGroup<Mode>(index).also {it.values=provider(it)}
 fun tagBy(group:ModeValueGroup<Mode>) {}
 open fun onDisabled() {}
}
''',
        'Main.kt':'''import com.google.gson.*
import net.ccbluex.liquidbounce.features.module.modules.world.ModuleFastBreak as M
import net.ccbluex.liquidbounce.event.events.*
import net.ccbluex.liquidbounce.utils.item.isMiningTool
import net.minecraft.SharedConstants
import net.minecraft.core.*
import net.minecraft.core.registries.*
import net.minecraft.resources.Identifier
import net.minecraft.server.Bootstrap
import net.minecraft.tags.*
import net.minecraft.world.item.*
import net.minecraft.network.protocol.game.ServerboundPlayerActionPacket as ActionPacket
val cached=mutableMapOf<String,List<Holder<Item>>>()
fun tag(name:String):List<Holder<Item>> = cached.getOrPut(name) {
 val stream=M::class.java.getResourceAsStream("/data/minecraft/tags/item/$name.json")!!
 val result=linkedSetOf<Holder<Item>>()
 stream.use {for(value in JsonParser.parseString(String(it.readAllBytes(),Charsets.UTF_8)).asJsonObject["values"].asJsonArray) {
  val id=if(value.isJsonObject)value.asJsonObject["id"].asString else value.asString
  if(id.startsWith("#"))result.addAll(tag(id.substring(1).removePrefix("minecraft:")))
  else result.add(BuiltInRegistries.ITEM.get(Identifier.parse(id)).orElseThrow())
 }}
 result.toList()
}
fun main() {
 SharedConstants.tryDetectVersion();Bootstrap.bootStrap()
 // Item components are unrelated to isMiningTool; bind empty prototypes so the
 // real ItemStack can apply its real count/isEmpty/typeHolder semantics.
 for(holder in BuiltInRegistries.ITEM.listElements())holder.bindComponents(net.minecraft.core.component.DataComponentMap.EMPTY)
 val bound=mutableMapOf<TagKey<Item>,List<Holder<Item>>>()
 val tags=JsonObject()
 for(name in listOf("axes","pickaxes","shovels","hoes")) {
  val values=tag(name);bound[TagKey.create(Registries.ITEM,Identifier.withDefaultNamespace(name))]=values
  val ids=JsonArray();for(holder in values)ids.add(BuiltInRegistries.ITEM.getKey(holder.value()).toString())
  tags.add("minecraft:$name",ids)
 }
 BuiltInRegistries.ITEM.prepareTagReload(TagLoader.LoadResult(Registries.ITEM,bound)).apply()
 val items=JsonArray()
 for(item in BuiltInRegistries.ITEM) {val row=JsonObject();row.addProperty("id",BuiltInRegistries.ITEM.getKey(item).toString());row.addProperty("tool",ItemStack(item).isMiningTool);row.addProperty("toolEmpty",ItemStack(item,0).isMiningTool);row.addProperty("toolNegative",ItemStack(item,-1).isMiningTool);items.add(row)}
 val oracle=JsonObject();oracle.add("tags",tags);oracle.add("items",items)
 java.io.File(System.getProperty("oracle")).writeText(oracle.toString())
 val writer=java.io.File(System.getProperty("results")).bufferedWriter()
 java.io.File(System.getProperty("commands")).forEachLine {line ->
  val t=line.split(" ");M.player.mainHandItem=ItemStack(BuiltInRegistries.ITEM.getValue(Identifier.parse(t[0])))
  M.floats["BreakDamage"]!!.value=t[1].toFloat();M.bools["OnlyTool"]!!.value=t[2].toBoolean()
  M.interaction.destroyDelay=t[5].toInt();M.interaction.destroyProgress=t[6].toFloat();M.network.sent.clear()
  val packet=ActionPacket(ActionPacket.Action.values()[t[7].toInt()],BlockPos(3,t[9].toInt(),-5),Direction.values()[t[8].toInt()],91)
  if(t[4].toBoolean()) {
   M.repeatable(GameTickEvent())
   if(t[3].toBoolean())M.AbortAnother.packetHandler(PacketEvent(packet))
  }
  val row=JsonObject();row.addProperty("delay",M.interaction.destroyDelay);row.addProperty("progressBits",Integer.toUnsignedLong(java.lang.Float.floatToRawIntBits(M.interaction.destroyProgress)))
  val extra=M.network.sent.firstOrNull() as? ActionPacket
  if(extra==null)row.add("abort",JsonNull.INSTANCE) else {
   val abort=JsonObject();val pos=JsonArray();pos.add(extra.pos.x);pos.add(extra.pos.y);pos.add(extra.pos.z)
   abort.add("pos",pos);abort.addProperty("direction",extra.direction.ordinal);abort.addProperty("sequence",extra.sequence);row.add("abort",abort)
  }
  M.onDisabled();row.addProperty("disabledDelay",M.interaction.destroyDelay)
  writer.write(row.toString());writer.newLine()
 }
 writer.close()
}
''',
    }
    for name,text in fixtures.items():(out/name).write_text(text,encoding='utf-8')
    commands=[]
    items=['air','diamond_axe','iron_pickaxe','stone_shovel','wooden_hoe','shears','diamond_sword','trident','dirt','stick']
    pairs=[(-4,0),(0,-0.0),(0.8,0.8),(0.8,0.8000000715255737),(0.8,1),(2,1),(2,2.1),(0.8,-4),(-2,-1),(0.8,'NaN'),('NaN',1),(0.8,'Infinity')]
    for item in items:
     for damage,progress in pairs:
      for only in [False,True]:
       for running in [False,True]:
        for grim in [False,True]:
         for action in range(5):
          for face in range(6):
           y=2147483647 if face==5 else 42
           commands.append(' '.join(map(str,[f'minecraft:{item}',damage,str(only).lower(),str(grim).lower(),str(running).lower(),4,progress,action,face,y])))
    (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    jars=[cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    jars.extend((ROOT/'build/resource-runtime/libraries').rglob('*.jar'))
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';stdlib=next(kotlin.glob('kotlin-stdlib-*.jar'))
    classpath=os.pathsep.join(map(str,[stdlib,*jars]));java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(out/'classes'),*[str(p)for p in sorted(out.glob('*.kt'))]])
    if result.returncode==0:
        result=run('kotlin-oracle',[str(java),'-Duser.language=en','-Duser.country=US','-Dfile.encoding=UTF-8','-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-Doracle='+str(out/'oracle.json'),'-cp',os.pathsep.join([str(out/'classes'),classpath]),'MainKt'])
    changed,originals={},{}
    def edit(path,action):
        source=UPSTREAM/path
        if source.exists():originals[path]=hashlib.sha256(source.read_bytes()).hexdigest()
        changed[path]=action(changed.get(path,source.read_text(encoding='utf-8')if source.exists()else''))
    def once(text,before,after):
        assert text.count(before)==1,(before,text.count(before));return text.replace(before,after,1)
    for name in ['packet','hand-derp','mining','fast-break']:
        added=runpy.run_path(str(HERE/(name+'-patch.py')))['apply'](edit,once)
        if name=='packet':
            for path,text in added.items():changed[path]=text
    native=changed['crates/horizon-client/src/game/native_hooks.rs'];remote=changed['crates/horizon-client/src/game/remote.rs'];network=changed['crates/horizon-client/src/net.rs']
    source=(HERE/'fast-break-fixture.rs').read_text(encoding='utf-8')
    helpers=runpy.run_path(str(HERE/'fast-break-patch.py'))['HELPERS']
    for key,text in {
        'HELPERS':helpers,
        'IDENTITY':block(changed['crates/horizon-client/src/motion.rs'],'    pub fn native_client_queries_allowed('),
        'GAME_SEND':block(remote,'    pub(super) fn send(&self, packet: S)'),
        'REMOTE_FIELDS':'\n'.join(line for line in block(remote,'pub(super) struct Remote').splitlines() if ':' in line and line.split(':',1)[0].split()[-1] in ['destroying','destroy_delay']),
        'REMOTE_MINING':block(remote,'    fn remote_mining('),
        'SESSION_METHODS':'\n'.join(block(network,signature)for signature in ['    pub fn send(&self,','    pub fn send_silently(&self,','    fn send_with_path(&self,']),
    }.items():source=source.replace('/* '+key+' */',text)
    (out/'lib.rs').write_text(source,encoding='utf-8')
    for dest,src in {
        'fast_break.rs':ROOT/'native/liquidbounce-behaviors/src/fast_break.rs','no_swing.rs':ROOT/'native/liquidbounce-behaviors/src/no_swing.rs',
        'fast_break_packets.rs':HERE/'engine/src/fast_break_packets.rs','fast_break_bridge.rs':HERE/'fast-break-bridge.rs','packets.rs':HERE/'engine/src/packets.rs',
    }.items():shutil.copyfile(src,out/dest)
    (out/'packet_bridge.rs').write_text(changed['crates/horizon-client/src/packet_bridge.rs'],encoding='utf-8')
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
    wrapper='#![allow(dead_code)]\nuse serde_json::{Value,Map,Number,json};use std::collections::BTreeMap;\n'
    for declaration in ['pub enum JsonFormat','pub struct CommonSettings','pub struct ConfigError']:
        wrapper+='#[derive(Clone,Copy,Debug,PartialEq,Eq)]\n'if declaration.endswith('JsonFormat')else'#[derive(Clone,Debug,PartialEq)]\n'
        wrapper+=block(config,declaration)+'\n'
    wrapper+='enum ModuleKind {FastBreak}impl ModuleKind {fn key(self)->&\'static str {"liquidbounce.module.fastBreak"}}\n'
    for declaration in ['impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn float_json(','fn scalar_string(',"fn nodes<'a>(","fn value_of<'a>(",'fn parse_bool(','fn parse_float(','\nfn leaf(']:wrapper+=block(config,declaration)+'\n'
    wrapper+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;enum SettingChange{FastBreak(child::FastBreakSetting),Enabled(bool)}#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}\n'
    wrapper+='#[path="config_child.rs"]mod child;\nfn parse_settings(_:ModuleKind,value:&Value)->Result<SettingsPatch,ConfigError>{let root=nodes(value,"FastBreak","FastBreak")?;let mut patch=SettingsPatch::default();leaf(&mut patch,&root,"Enabled","FastBreak.Enabled",|v,p|parse_bool(v,p).map(SettingChange::Enabled));child::parse_fast_break(&mut patch,&root);Ok(patch)}\n'
    (out/'config.rs').write_text(wrapper,encoding='utf-8');shutil.copyfile(HERE/'engine/src/config/fast_break.rs',out/'config_child.rs')
    api=out/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True);(game.parent/'mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{\n    pub(super) fn native_fixture_tick(&mut self) { mod_api::dispatch_fixture_tick(self, mod_api::mods()); }\n"+helpers+'\n}',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True);render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'hooks':[{'name':'fixture_tick','target':"horizon_client::game::native_hooks::Game<'_>::native_fixture_tick",'mod_api':{'contract':'listen'}}],'helpers':[{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"}for name in ['client_fast_break_update','client_fast_break_reset']]}),encoding='utf-8')
    (out/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n',encoding='utf-8')
    manifest='[package]\nname="liquidbounce-fast-break-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\nhorizon-protocol={path="'+(UPSTREAM/'crates/horizon-protocol').as_posix()+'"}\n[build-dependencies]\nhorizon-mod-api-gen={path="'+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n'
    (out/'Cargo.toml').write_text(manifest,encoding='utf-8')
    if result.returncode==0:
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target')
        result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],env)
        if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    files=[module,categories,HERE/'fast-break-patch.py',HERE/'fast-break-bridge.rs',HERE/'fast-break-fixture.rs',Path(__file__),ROOT/'native/liquidbounce-behaviors/src/fast_break.rs',HERE/'engine/src/config/fast_break.rs',HERE/'engine/src/fast_break_packets.rs',HERE/'engine/src/fast_break_runtime.rs']
    report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'commands':records,'cases':len(commands),'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'upstreamSha256':originals,'transformedSha256':{p:hashlib.sha256(t.encode()).hexdigest()for p,t in changed.items()},
        'scope':'Unmodified ModuleFastBreak; byte-exact item-tag helpers; actual MC registry, official item tags and packet constructors; actual native helpers/generated Host, remote_mining, Game.send, packet listener/pipeline, Session methods, Pipe and codec.',
        'boundaries':['ClientModule value delegate, handler activation and interaction field shell','Game/window, resource map and collision/prediction world shell','parent config/registry/lifecycle and full game execution'],'fullGameIntegrationPassed':False}
    report['upstreamUnchanged']=all(hashlib.sha256((UPSTREAM/p).read_bytes()).hexdigest()==h for p,h in originals.items());report['passed']&=report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)
if __name__=='__main__':main()
