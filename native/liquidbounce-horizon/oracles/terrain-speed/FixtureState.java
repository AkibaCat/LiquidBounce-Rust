package fixture;
import java.util.*;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.*;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.level.block.*;
public class FixtureState {
 public static boolean running;
 public static Player player;
 public static Client client=new Client();
 public static int pattern;
 public static final List<String> trace=new ArrayList<>();
 public static final List<Vec3> snaps=new ArrayList<>();
 public static final sun.misc.Unsafe unsafe;
 static {try{var f=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");f.setAccessible(true);unsafe=(sun.misc.Unsafe)f.get(null);}catch(Exception e){throw new RuntimeException(e);}}
 public static Player newPlayer() throws Exception {return (Player)unsafe.allocateInstance(Player.class);}
 public static class Key {public boolean down;public String label;public boolean isDown(){trace.add(label);return down;}}
 public static class Options {
  public Key keyUp=new Key(),keySprint=new Key(),keyJump=new Key(),keyShift=new Key();
  Options(){keyUp.label="forward";keySprint.label="sprint";keyJump.label="jump";keyShift.label="sneak";}
 }
 public static class Client {public Options options=new Options();}
 public static Block block(BlockPos p){
  trace.add("block:"+p.getX()+":"+p.getY()+":"+p.getZ());
  int dy=p.getY()-player.startBlockY;
  return switch(pattern){
   case 0->Blocks.LADDER;
   case 1->dy<=3?Blocks.VINE:Blocks.AIR;
   case 2->Blocks.AIR;
   case 3->Blocks.SCAFFOLDING;
   case 4->dy<8?Blocks.LADDER:Blocks.AIR;
   default->dy%2==0?Blocks.VINE:Blocks.LADDER;
  };
 }
 public static class Player extends LocalPlayer {
  public boolean climbing,water;
  public Vec3 fixturePosition,fixtureVelocity;
  public Direction fixtureDirection;
  public int startBlockY;
  public Player(){super(null,null,null,null,null,null,false,null,null);}
  @Override public boolean onClimbable(){trace.add("climbable");return climbing;}
  @Override public boolean isInWater(){trace.add("water");return water;}
  @Override public Vec3 position(){trace.add("position");return fixturePosition;}
  @Override public BlockPos blockPosition(){trace.add("blockPosition");return BlockPos.containing(fixturePosition);}
  @Override public Direction getDirection(){trace.add("direction");return fixtureDirection;}
  @Override public void absSnapTo(double x,double y,double z){trace.add("snap");super.absSnapTo(x,y,z);snaps.add(fixturePosition);}
  @Override public void setPos(double x,double y,double z){fixturePosition=new Vec3(x,y,z);}
  @Override public Vec3 getDeltaMovement(){trace.add("velocity");return fixtureVelocity;}
  @Override public void setDeltaMovement(Vec3 v){trace.add("setVelocity");fixtureVelocity=v;}
 }
}
