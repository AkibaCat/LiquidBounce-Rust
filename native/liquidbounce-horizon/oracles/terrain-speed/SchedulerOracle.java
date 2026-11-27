package fixture;

import com.google.common.collect.ImmutableList;
import com.google.gson.*;
import java.lang.invoke.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.SharedConstants;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.Holder;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.world.TickRateManager;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.*;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.entity.EntityTickList;
import net.minecraft.world.level.storage.*;
import net.minecraft.world.phys.Vec3;

/** Runs the jar's ClientLevel walk, commonTick and Entity.rideTick unchanged.
 * The entities' tick bodies provide a controlled observable callback only. */
public final class SchedulerOracle {
    static final sun.misc.Unsafe U;
    static final List<Integer> trace = new ArrayList<>();
    static final List<String> history = new ArrayList<>();
    static ClientLevel level;
    static EntityTickList ticking;
    static Entity[] entities;
    static Map<Integer, Runnable> actions;
    static MethodHandle entityRideTick;
    static {
        try {
            var f=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");f.setAccessible(true);U=(sun.misc.Unsafe)f.get(null);
            entityRideTick=MethodHandles.privateLookupIn(Entity.class,MethodHandles.lookup()).findSpecial(Entity.class,"rideTick",MethodType.methodType(void.class),Entity.class);
        } catch(Throwable e){throw new ExceptionInInitializerError(e);}
    }
    static Field field(Class<?> owner,String name)throws Exception{var f=owner.getDeclaredField(name);f.setAccessible(true);return f;}
    static void put(Class<?> owner,Object value,String name,Object data)throws Exception{field(owner,name).set(value,data);}
    static void tick(Entity entity) {
        int id=entity.getId()-1;trace.add(id);
        history.add(id+":"+entity.tickCount+":"+Long.toUnsignedString(Double.doubleToRawLongBits(entity.getDeltaMovement().x)));
        var action=actions.remove(id);if(action!=null)action.run();
    }
    public static class Node extends Entity {
        public Node(){super(EntityTypes.PIG,null);}
        @Override protected void defineSynchedData(SynchedEntityData.Builder b){}
        @Override protected void readAdditionalSaveData(ValueInput v){}
        @Override protected void addAdditionalSaveData(ValueOutput v){}
        @Override public boolean hurtServer(ServerLevel l,DamageSource s,float n){return false;}
        @Override public boolean isPushable(){return true;}
        @Override public void tick(){SchedulerOracle.tick(this);}
        @Override public void stopRiding(){try{put(Entity.class,this,"vehicle",null);}catch(Exception e){throw new RuntimeException(e);}}
        @Override protected void positionRider(Entity p,MoveFunction move){history.add("seat:"+(getId()-1)+":"+(p.getId()-1));}
    }
    public static class PlayerNode extends LocalPlayer {
        public PlayerNode(){super(null,null,null,null,null,null,false,null,null);}
        @Override public void tick(){SchedulerOracle.tick(this);}
        @Override public void rideTick(){try{entityRideTick.invokeExact((Entity)this);}catch(Throwable e){throw new RuntimeException(e);}}
        @Override public void stopRiding(){try{put(Entity.class,this,"vehicle",null);}catch(Exception e){throw new RuntimeException(e);}}
        @Override protected void positionRider(Entity p,MoveFunction move){history.add("seat:"+(getId()-1)+":"+(p.getId()-1));}
    }
    static Entity node(int id,boolean player)throws Exception {
        Entity e=(Entity)U.allocateInstance(player?PlayerNode.class:Node.class);
        put(Entity.class,e,"type",player?EntityTypes.PLAYER:EntityTypes.PIG);
        put(Entity.class,e,"level",level);put(Entity.class,e,"position",Vec3.ZERO);
        put(Entity.class,e,"deltaMovement",new Vec3(id+0.25,0,0));
        put(Entity.class,e,"passengers",ImmutableList.of());
        put(Entity.class,e,"interpolationHandler",InterpolationHandler.NO_OP);
        e.setId(id+1);return e;
    }
    static void passengers(int vehicle,int... children)throws Exception {
        var list=new ArrayList<Entity>();for(int child:children){list.add(entities[child]);put(Entity.class,entities[child],"vehicle",entities[vehicle]);}
        put(Entity.class,entities[vehicle],"passengers",ImmutableList.copyOf(list));
    }
    static JsonObject run(JsonObject input)throws Exception {
        trace.clear();history.clear();actions=new HashMap<>();
        level=(ClientLevel)U.allocateInstance(ClientLevel.class);
        ticking=new EntityTickList();put(ClientLevel.class,level,"tickingEntities",ticking);
        var rates=new TickRateManager();rates.setFrozen(input.get("frozen").getAsBoolean());rates.tick();
        put(ClientLevel.class,level,"tickRateManager",rates);put(Level.class,level,"isClientSide",true);
        int count=input.get("count").getAsInt();entities=new Entity[count];
        for(int i=0;i<count;i++)entities[i]=node(i,i==0);
        for(var v:input.getAsJsonArray("insertion"))ticking.add(entities[v.getAsInt()]);
        for(var edge:input.getAsJsonArray("passengers")){var values=edge.getAsJsonArray();int[] children=new int[values.size()-1];for(int i=1;i<values.size();i++)children[i-1]=values.get(i).getAsInt();passengers(values.get(0).getAsInt(),children);}
        for(var v:input.getAsJsonArray("removed"))put(Entity.class,entities[v.getAsInt()],"removalReason",Entity.RemovalReason.DISCARDED);
        for(var v:input.getAsJsonArray("actions")) {
            var a=v.getAsJsonObject();int turn=a.get("turn").getAsInt(),target=a.get("target").getAsInt();String kind=a.get("kind").getAsString();
            actions.put(turn,()->{try{switch(kind){case "remove"-> {put(Entity.class,entities[target],"removalReason",Entity.RemovalReason.DISCARDED);ticking.remove(entities[target]);}case "detach"->entities[target].stopRiding();case "add"->ticking.add(entities[target]);default->throw new IllegalArgumentException(kind);}}catch(Exception e){throw new RuntimeException(e);}});
        }
        level.tickEntities();
        var gson=new Gson();var out=new JsonObject();out.add("input",input);out.add("trace",gson.toJsonTree(trace));out.add("history",gson.toJsonTree(history));return out;
    }
    public static void main(String[] args)throws Exception {
        SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
        var rows=JsonParser.parseString(Files.readString(Path.of(args[0]))).getAsJsonArray();var results=new JsonArray();
        for(var row:rows)results.add(run(row.getAsJsonObject()));
        Files.writeString(Path.of(args[1]),new Gson().toJson(results));
        if(args.length>2){
            var pushes=new JsonArray();
            double[] axes={0.0,-0.0,0.001,-0.001,(double)0.01f,-(double)0.01f,0.1,-0.1,1.0,-1.0,3.25,-7.5};
            for(double x:axes)for(double z:axes){
                Node a=(Node)node(1,false),b=(Node)node(2,false);
                put(Entity.class,a,"position",Vec3.ZERO);put(Entity.class,b,"position",new Vec3(x,0,z));
                a.setDeltaMovement(new Vec3(0.3,-0.0,-0.4));b.setDeltaMovement(new Vec3(0.3,-0.0,-0.4));
                a.push(b);
                var row=new JsonObject();row.add("offset",bits(new Vec3(x,0,z)));row.add("first",bits(a.getDeltaMovement()));row.add("second",bits(b.getDeltaMovement()));pushes.add(row);
            }
            Files.writeString(Path.of(args[2]),new Gson().toJson(pushes));
        }
        System.out.println("actual ClientLevel scheduler cases="+results.size());
    }
    static JsonArray bits(Vec3 v){var a=new JsonArray();for(double d:new double[]{v.x,v.y,v.z})a.add(Long.toUnsignedString(Double.doubleToRawLongBits(d)));return a;}
}
