#!/usr/bin/env python3
"""Execute untouched ModuleElytraRecast.kt and exact original mixin method body.

Real MC ItemStack/packet/sound IDs; constructor-free LocalPlayer subclass and
client/module/input/sound providers are explicit recording shells.
"""
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT / 'build/elytra-recast-tests'


def main():
    work = OUT / 'original'
    work.mkdir(parents=True, exist_ok=True)
    classes = work / 'classes'
    classes.mkdir(exist_ok=True)
    original = ROOT / 'upstream/LiquidBounce/src/main'
    module = original / 'kotlin/net/ccbluex/liquidbounce/features/module/modules/movement/ModuleElytraRecast.kt'
    mixin = original / 'java/net/ccbluex/liquidbounce/injection/mixins/minecraft/entity/MixinLivingEntity.java'
    items = original / 'kotlin/net/ccbluex/liquidbounce/utils/item/ItemCategorizations.kt'
    block = runpy.run_path(str(HERE / 'test-attack-patch.py'))['block']
    body = block(mixin.read_text(encoding='utf-8'), '    public void recastIfLanded(')
    glider_source = items.read_text(encoding='utf-8')
    start = glider_source.index('val DataComponentHolder.isGlider: Boolean')
    glider = glider_source[start:glider_source.index('\n\n', start)]
    shutil.copyfile(module, work / module.name)
    (work / 'Module.kt').write_text('''package net.ccbluex.liquidbounce.features.module
import fixture.FixtureState
enum class ModuleCategories {MOVEMENT}
open class ClientModule(val name:String,val category:ModuleCategories) {
 val player get()=FixtureState.player
 val mc get()=net.minecraft.client.Minecraft.getInstance()
 val network get()=FixtureState.network
 val running get()=FixtureState.bits and 1 != 0
}
''', encoding='utf-8')
    (work / 'Glider.kt').write_text('''package net.ccbluex.liquidbounce.utils.item
import net.minecraft.core.component.DataComponentHolder
import net.minecraft.core.component.DataComponents
''' + glider + '\n', encoding='utf-8')
    (work / 'CallbackInfo.java').write_text('package org.spongepowered.asm.mixin.injection.callback; public class CallbackInfo {}\n', encoding='utf-8')
    fixture = '''package fixture;
import com.google.gson.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.client.*;
import net.minecraft.client.player.*;
import net.minecraft.client.sounds.SoundManager;
import net.minecraft.core.Holder;
import net.minecraft.core.component.DataComponents;
import net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket;
import net.minecraft.resources.Identifier;
import net.minecraft.sounds.*;
import net.minecraft.util.Unit;
import net.minecraft.world.effect.*;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.player.*;
import net.minecraft.world.item.*;
import net.ccbluex.liquidbounce.features.module.modules.movement.ModuleElytraRecast;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
public class FixtureState {
 public static int bits;
 public static FixtureMixin player;
 public static Network network=new Network();
 public static final List<String> trace=new ArrayList<>();
 public static final sun.misc.Unsafe unsafe;
 static {try {var f=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");f.setAccessible(true);unsafe=(sun.misc.Unsafe)f.get(null);}catch(Exception e){throw new RuntimeException(e);}}
 static <T> T allocate(Class<T> c) throws InstantiationException{return c.cast(unsafe.allocateInstance(c));}
 public static class Network {
  public void send(ServerboundPlayerCommandPacket packet){
   if(packet.getId()!=71 || packet.getAction()!=ServerboundPlayerCommandPacket.Action.START_FALL_FLYING || packet.getData()!=0)throw new AssertionError("wrong actual packet");
   trace.add("send");
  }
 }
 public static class Key extends KeyMapping {
  public Key(){super("fixture",0,KeyMapping.Category.MOVEMENT);}
  @Override public boolean isDown(){trace.add("jumpKey");return (bits&512)!=0;}
 }
 public static class Sounds extends SoundManager {
  public Sounds(){super(null);}
  @Override public void stop(Identifier sound,SoundSource category){
   if(!sound.toString().equals("minecraft:item.elytra.flying") || category!=SoundSource.PLAYERS)throw new AssertionError("wrong sound");
   trace.add("stop");
  }
 }
 public static class Client extends Minecraft {
  public Sounds sound;
  public Client(){super(null);}
  @Override public SoundManager getSoundManager(){return sound;}
 }
 public static class Player extends LocalPlayer {
  public Abilities fixtureAbilities;
  public ItemStack chest;
  public boolean glide;
  public int noJumpDelay;
  public Player(){super(null,null,null,null,null,null,false,null,null);}
  @Override public Abilities getAbilities(){trace.add("flying");return fixtureAbilities;}
  @Override public int getId(){return 71;}
  @Override public ItemStack getItemBySlot(EquipmentSlot slot){if(slot!=EquipmentSlot.CHEST)throw new AssertionError("wrong slot");trace.add("chest");return chest;}
  @Override public boolean isPassenger(){trace.add("passenger");return (bits&8)!=0;}
  @Override public boolean onClimbable(){trace.add("climbable");return (bits&16)!=0;}
  @Override public boolean isInWater(){trace.add("water");return (bits&32)!=0;}
  @Override public boolean hasEffect(Holder<MobEffect> effect){if(effect!=MobEffects.LEVITATION)throw new AssertionError("wrong effect");trace.add("levitation");return (bits&64)!=0;}
  @Override public boolean isFallFlying(){trace.add("fallFlying");return glide;}
  @Override public void startFallFlying(){trace.add("start");glide=true;}
 }
 public static class FixtureMixin extends Player {
  private boolean previousElytra=false;
  public boolean liquid_bounce$isClientPlayer(){return (bits&2048)!=0;}
__METHOD__
 }
 static void setup(int flags,boolean previous)throws Exception{
  bits=flags;player=allocate(FixtureMixin.class);player.previousElytra=previous;
  player.fixtureAbilities=new Abilities();player.fixtureAbilities.flying=(bits&4)!=0;
  player.input=new ClientInput();player.input.keyPresses=new Input(false,false,false,false,(bits&1024)!=0,false,false);
  player.glide=(bits&2)!=0;player.noJumpDelay=7;
  player.chest=new ItemStack(Items.STICK);
  if((bits&128)!=0)player.chest.set(DataComponents.GLIDER,Unit.INSTANCE);else player.chest.remove(DataComponents.GLIDER);
  player.chest.set(DataComponents.MAX_DAMAGE,2);player.chest.set(DataComponents.DAMAGE,(bits&256)!=0?1:0);
  trace.clear();
 }
 static JsonObject row(int flags,boolean previous){
  JsonObject row=new JsonObject();row.addProperty("bits",flags);row.addProperty("was",previous);
  row.addProperty("glide",player.glide);row.addProperty("previous",player.previousElytra);row.addProperty("delay",player.noJumpDelay);
  row.add("trace",new Gson().toJsonTree(trace));return row;
 }
 public static void main(String[] args)throws Exception{
  net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap();
  for(Item item:List.of(Items.AIR,Items.STICK))item.builtInRegistryHolder().bindComponents(net.minecraft.core.component.DataComponentMap.builder().build());
  Client client=allocate(Client.class);client.sound=allocate(Sounds.class);
  Options options=allocate(Options.class);
  var jump=Options.class.getDeclaredField("keyJump");jump.setAccessible(true);jump.set(options,allocate(Key.class));
  var opts=Minecraft.class.getDeclaredField("options");opts.setAccessible(true);opts.set(client,options);
  var instance=Minecraft.class.getDeclaredField("instance");instance.setAccessible(true);instance.set(null,client);
  JsonArray flows=new JsonArray();
  for(int flags=0;flags<4096;flags++)for(boolean previous:new boolean[]{false,true}){
   setup(flags,previous);player.recastIfLanded(null);flows.add(row(flags,previous));
  }
  JsonArray direct=new JsonArray();
  for(int flags=0;flags<2048;flags++){
   setup(flags,false);boolean result=ModuleElytraRecast.INSTANCE.recastElytra();JsonObject row=row(flags,false);row.addProperty("result",result);direct.add(row);
  }
  JsonObject out=new JsonObject();out.add("flows",flows);out.add("direct",direct);
  Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(out));
 }
}
'''
    (work / 'FixtureState.java').write_text(fixture.replace('__METHOD__', body), encoding='utf-8')
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    minecraft = cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    jars = [minecraft]
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j', 'org.joml',
                  'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        jars.extend((cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    jdk = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    kotlin = ROOT.parent / 'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    jars += [next(kotlin.glob('kotlin-stdlib-*.jar'))]
    classpath = os.pathsep.join(map(str, [classes, *jars]))
    sources = [*sorted(work.glob('*.kt')), *sorted(work.glob('*.java'))]
    commands = [
        ('kotlin-compile', [str(jdk/'java.exe'), '-cp', str(kotlin/'*'), 'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler', '-no-stdlib', '-no-reflect', '-classpath', classpath, '-d', str(classes), *map(str, sources)]),
        ('original-java-compile', [str(jdk/'javac.exe'), '-cp', classpath, '-d', str(classes), *map(str, sorted(work.glob('*.java')))]),
        ('original-oracle', [str(jdk/'java.exe'), '-cp', classpath, 'fixture.FixtureState', str(work/'flows.json')]),
    ]
    for name, command in commands:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (OUT/(name+'.log')).write_text(result.stdout+result.stderr, encoding='utf-8')
        if result.returncode:
            raise SystemExit(result.stdout+result.stderr)
    facts = json.loads((OUT/'oracle.json').read_text(encoding='utf-8'))
    original_flows = json.loads((work/'flows.json').read_text(encoding='utf-8'))
    facts.update(original_flows)
    facts['policySource'] = 'unchanged original ModuleElytraRecast.kt; exact extracted original MixinLivingEntity.recastIfLanded method body; exact original DataComponentHolder.isGlider property'
    facts['policyBoundaries'] = ['constructor-free real LocalPlayer subclass with recording state providers', 'module registry/running shell', 'Minecraft options/physical key/sound manager recording shells', 'actual MC ItemStack, Input, PlayerCommand and sound identifiers; controlled empty stick/air prototypes for synthetic policy flags', 'Input.jump record accessor and primitive noJumpDelay assignment uninstrumented; final cooldown asserted independently']
    facts['originalModuleCopySha256'] = hashlib.sha256((work/module.name).read_bytes()).hexdigest()
    facts['originalModuleUnchanged'] = module.read_bytes() == (work/module.name).read_bytes()
    facts['exactMixinMethodSha256'] = hashlib.sha256(body.encode()).hexdigest()
    facts['exactGliderPropertySha256'] = hashlib.sha256(glider.encode()).hexdigest()
    facts['sourceSha256'].update({str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [module, mixin, items, Path(__file__)]})
    (OUT/'oracle.json').write_text(json.dumps(facts, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({'stacks':len(facts['stacks']), 'flows':len(facts['flows']), 'direct':len(facts['direct']), 'originalModuleUnchanged':facts['originalModuleUnchanged']}))


if __name__ == '__main__':
    main()
