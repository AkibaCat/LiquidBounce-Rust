// SPDX-License-Identifier: GPL-3.0-or-later
// Actual MC getters/consumers; constructor-free probes supply equipment/passenger inputs.
import com.google.gson.*;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.*;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.entity.animal.equine.*;
import net.minecraft.world.entity.animal.pig.Pig;
import net.minecraft.world.entity.monster.Strider;
import net.minecraft.world.entity.animal.nautilus.Nautilus;
import net.minecraft.world.item.*;
import net.minecraft.world.level.GameType;
import net.minecraft.client.player.LocalPlayer;
import java.nio.file.*;
import sun.misc.Unsafe;
public class EntityControlFacts {
 static Unsafe unsafe;
 interface Probe {void setup(boolean physical,boolean force,Entity first);}
 static class ProbePlayer extends Player {
  boolean holding;
  ProbePlayer(){super(null,null);}
  public GameType gameMode(){return GameType.SURVIVAL;}
  @Override public boolean isHolding(Item item){return holding;}
 }
 static class ProbeLocal extends LocalPlayer {
  boolean force;
  ProbeLocal(){super(null,null,null,null,null,null,false,null,null);}
  @Override public float getJumpRidingScale(){return force?1f:super.getJumpRidingScale();}
 }
 static class ProbeHorse extends Horse implements Probe {
  boolean physical,force;Entity first;
  ProbeHorse(){super(null,null);}
  public void setup(boolean p,boolean f,Entity r){physical=p;force=f;first=r;}
  @Override public Entity getFirstPassenger(){return first;}
  @Override public boolean isNoAi(){return false;}
  @Override public boolean hasItemInSlot(EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public ItemStack getItemBySlot(EquipmentSlot slot){return ItemStack.EMPTY;}
  @Override public boolean isEquippableInSlot(ItemStack stack,EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public boolean isSaddled(){return force||super.isSaddled();}
 }
 static class ProbeLlama extends Llama implements Probe {
  boolean physical,force;Entity first;
  ProbeLlama(){super(null,null);}
  public void setup(boolean p,boolean f,Entity r){physical=p;force=f;first=r;}
  @Override public Entity getFirstPassenger(){return first;}
  @Override public boolean isNoAi(){return false;}
  @Override public boolean hasItemInSlot(EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public ItemStack getItemBySlot(EquipmentSlot slot){return ItemStack.EMPTY;}
  @Override public boolean isEquippableInSlot(ItemStack stack,EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public boolean isSaddled(){return force||super.isSaddled();}
 }
 static class ProbePig extends Pig implements Probe {
  boolean physical,force;Entity first;
  ProbePig(){super(null,null);}
  public void setup(boolean p,boolean f,Entity r){physical=p;force=f;first=r;}
  @Override public Entity getFirstPassenger(){return first;}
  @Override public boolean isNoAi(){return false;}
  @Override public boolean hasItemInSlot(EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public ItemStack getItemBySlot(EquipmentSlot slot){return ItemStack.EMPTY;}
  @Override public boolean isEquippableInSlot(ItemStack stack,EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public boolean isSaddled(){return force||super.isSaddled();}
 }
 static class ProbeStrider extends Strider implements Probe {
  boolean physical,force;Entity first;
  ProbeStrider(){super(null,null);}
  public void setup(boolean p,boolean f,Entity r){physical=p;force=f;first=r;}
  @Override public Entity getFirstPassenger(){return first;}
  @Override public boolean isNoAi(){return false;}
  @Override public boolean hasItemInSlot(EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public ItemStack getItemBySlot(EquipmentSlot slot){return ItemStack.EMPTY;}
  @Override public boolean isEquippableInSlot(ItemStack stack,EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public boolean isSaddled(){return force||super.isSaddled();}
 }
 static class ProbeNautilus extends Nautilus implements Probe {
  boolean physical,force;Entity first;
  ProbeNautilus(){super(null,null);}
  public void setup(boolean p,boolean f,Entity r){physical=p;force=f;first=r;}
  @Override public Entity getFirstPassenger(){return first;}
  @Override public boolean isNoAi(){return false;}
  @Override public boolean hasItemInSlot(EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public ItemStack getItemBySlot(EquipmentSlot slot){return ItemStack.EMPTY;}
  @Override public boolean isEquippableInSlot(ItemStack stack,EquipmentSlot slot){return slot==EquipmentSlot.SADDLE && physical;}
  @Override public boolean isSaddled(){return force||super.isSaddled();}
 }
 public static void main(String[] args)throws Exception {
  SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
  var field=Unsafe.class.getDeclaredField("theUnsafe");field.setAccessible(true);unsafe=(Unsafe)field.get(null);
  JsonObject out=new JsonObject();JsonArray rows=new JsonArray();
  Class<?>[] types={ProbeHorse.class,ProbeLlama.class,ProbePig.class,ProbeStrider.class,ProbeNautilus.class};
  for(int mask=0;mask<4;mask++)for(boolean running:new boolean[]{false,true})
   for(boolean physical:new boolean[]{false,true})for(boolean rider:new boolean[]{false,true})
    for(boolean steer:new boolean[]{false,true})for(int species=0;species<types.length;species++){
     var player=(ProbePlayer)unsafe.allocateInstance(ProbePlayer.class);player.holding=steer;
     var mob=(Mob)unsafe.allocateInstance(types[species]);
     ((Probe)mob).setup(physical,running&&(mask&1)!=0,rider?player:null);
     JsonObject row=new JsonObject();row.addProperty("mask",mask);row.addProperty("running",running);
     row.addProperty("physical",physical);row.addProperty("rider",rider);row.addProperty("steer",steer);
     row.addProperty("species",species);row.addProperty("saddled",mob.isSaddled());
     row.addProperty("controlled",mob.getControllingPassenger()!=null);
     row.addProperty("jumpable",mob instanceof PlayerRideableJumping j && j.canJump());rows.add(row);
    }
  out.add("controls",rows);JsonArray scales=new JsonArray();
  var raw=LocalPlayer.class.getDeclaredField("jumpRidingScale");raw.setAccessible(true);
  for(int mask=0;mask<4;mask++)for(boolean running:new boolean[]{false,true})for(float original:new float[]{0f,.3f,.9f,1f,-3f}){
   var player=(ProbeLocal)unsafe.allocateInstance(ProbeLocal.class);player.force=running&&(mask&2)!=0;raw.setFloat(player,original);
   JsonObject row=new JsonObject();row.addProperty("mask",mask);row.addProperty("running",running);row.addProperty("original",original);
   row.addProperty("bits",Integer.toUnsignedLong(Float.floatToRawIntBits(player.getJumpRidingScale())));scales.add(row);
  }
  out.add("scales",scales);Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(out)+"\n");
 }
}
