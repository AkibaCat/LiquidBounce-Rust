// SPDX-License-Identifier: GPL-3.0-or-later
// Actual MC registry, shape metadata and complete BlockCollisions cursor.
import com.google.gson.*;
import com.google.gson.stream.*;
import net.minecraft.SharedConstants;
import net.minecraft.core.*;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.resources.Identifier;
import net.minecraft.world.level.*;
import net.minecraft.world.level.block.*;
import net.minecraft.world.level.block.state.*;
import net.minecraft.world.level.block.state.properties.*;
import net.minecraft.world.phys.*;
import net.minecraft.world.phys.shapes.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;

public class BlockWalkFacts {
 static Gson gson=new Gson();
 static JsonElement json(Object value){return gson.toJsonTree(value);}
 static List<double[]> boxes(VoxelShape shape){return shape.toAabbs().stream().map(b->new double[]{b.minX,b.minY,b.minZ,b.maxX,b.maxY,b.maxZ}).toList();}
 static final List<BlockState> states=new ArrayList<>();
 static Map<BlockPos,Integer> cells=new HashMap<>();
 static int index(BlockState state){states.add(state);return states.size()-1;}
 static CollisionGetter world;
 static CollisionContext context(boolean active,Set<String> selected,boolean suppressed){
  return context(active,selected,suppressed,null);
 }
 static CollisionContext context(boolean active,Set<String> selected,boolean suppressed,Integer ghostY){
  CollisionContext original=CollisionContext.empty();
  return (CollisionContext)Proxy.newProxyInstance(BlockWalkFacts.class.getClassLoader(),new Class<?>[]{CollisionContext.class},(proxy,method,args)->{
   Object result=method.invoke(original,args);
   if(method.getName().equals("getCollisionShape") && !suppressed &&
      (active && selected.contains(BuiltInRegistries.BLOCK.getKey(((BlockState)args[0]).getBlock()).toString()) || ghostY!=null && ((BlockPos)args[2]).getY()<ghostY))return Shapes.block();
   return result;
  });
 }
 public static void main(String[] args)throws Exception {
  SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
  world=(CollisionGetter)Proxy.newProxyInstance(BlockWalkFacts.class.getClassLoader(),new Class<?>[]{CollisionGetter.class},(proxy,method,values)->{
   return switch(method.getName()){
    case "getBlockState" -> states.get(cells.getOrDefault((BlockPos)values[0],0));
    case "getFluidState" -> states.get(cells.getOrDefault((BlockPos)values[0],0)).getFluidState();
    case "getBlockEntity" -> null;
    case "getChunkForCollisions" -> proxy;
    case "getMinY" -> -64;
    case "getHeight" -> 384;
    case "getEntityCollisions" -> List.of();
    case "getWorldBorder" -> new net.minecraft.world.level.border.WorldBorder();
    default -> {if(method.isDefault())yield InvocationHandler.invokeDefault(proxy,method,values);throw new UnsupportedOperationException(method.toString());}
   };
  });
  index(Blocks.AIR.defaultBlockState());index(Blocks.COBWEB.defaultBlockState());
  for(int layers=1;layers<=8;layers++)index(Blocks.SNOW.defaultBlockState().setValue(BlockStateProperties.LAYERS,layers));
  index(Blocks.STONE.defaultBlockState());
  for(SlabType type:SlabType.values())index(Blocks.OAK_SLAB.defaultBlockState().setValue(BlockStateProperties.SLAB_TYPE,type));
  index(Blocks.WATER.defaultBlockState());index(Blocks.LAVA.defaultBlockState());
  index(Blocks.OAK_FENCE.defaultBlockState());index(Blocks.CACTUS.defaultBlockState());
  index(Blocks.SCAFFOLDING.defaultBlockState());index(Blocks.BARRIER.defaultBlockState());
  JsonObject out=new JsonObject();JsonArray registry=new JsonArray();
  for(var block:BuiltInRegistries.BLOCK)registry.add(BuiltInRegistries.BLOCK.getKey(block).toString());out.add("registry",registry);
  JsonArray metadata=new JsonArray();
  for(var state:states){
   JsonObject row=new JsonObject();row.addProperty("id",BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
   JsonObject props=new JsonObject();state.getValues().forEach(entry->props.addProperty(entry.property().getName(),entry.valueName()));row.add("properties",props);
   row.add("boxes",json(boxes(CollisionContext.empty().getCollisionShape(state,world,BlockPos.ZERO))));
   row.addProperty("large",state.hasLargeCollisionShape());row.addProperty("piston",state.is(Blocks.MOVING_PISTON));
   row.addProperty("suffocating",state.isSuffocating(world,BlockPos.ZERO));metadata.add(row);
  }
  out.add("states",metadata);JsonArray scans=new JsonArray();
  double[][] bounds={{.1,.1,.1,.9,.9,.9},{-.3,.9999999,-.3,1.3,1.0000001,1.3},
   {-1.2,-.2,-1.2,.2,.2,.2},{-16.8,.3,30.8,-14.2,1.2,32.2},{0,0,0,1,1,1},{.5,1.,.5,.5,1.,.5}};
  int[][] positions={{0,0,0},{1,0,0},{-1,-1,-1},{-16,0,31},{-15,0,31}};
  for(int stateIndex=0;stateIndex<states.size();stateIndex++)for(int which=0;which<bounds.length;which++)
   for(int policy=0;policy<5;policy++)for(boolean onlySuffocating:new boolean[]{false,true}){
    cells.clear();for(int[] p:positions)cells.put(new BlockPos(p[0],p[1],p[2]),stateIndex);
    boolean active=policy!=0,suppressed=policy==4;
    Set<String> selected=switch(policy){case 1,4->Set.of("minecraft:cobweb","minecraft:snow");case 2->Set.of(BuiltInRegistries.BLOCK.getKey(states.get(stateIndex).getBlock()).toString());case 3->Set.of("minecraft:air");default->Set.of();};
    double[] b=bounds[which];AABB box=new AABB(b[0],b[1],b[2],b[3],b[4],b[5]);
    var scan=new BlockCollisions<JsonObject>(world,context(active,selected,suppressed),box,onlySuffocating,(p,s)->{
     JsonObject hit=new JsonObject();hit.add("position",json(new int[]{p.getX(),p.getY(),p.getZ()}));hit.add("boxes",json(boxes(s)));return hit;});
    JsonArray hits=new JsonArray();while(scan.hasNext())hits.add(scan.next());
    JsonObject row=new JsonObject();row.addProperty("state",stateIndex);row.add("bounds",json(b));row.add("positions",json(positions));row.add("selected",json(selected));row.addProperty("active",active);row.addProperty("suppressed",suppressed);row.addProperty("onlySuffocating",onlySuffocating);row.add("hits",hits);scans.add(row);
   }
  out.add("scans",scans);
  JsonArray combined=new JsonArray();
  for(int stateIndex=0;stateIndex<states.size();stateIndex++)for(double[] b:bounds)
   for(int ghostY:new int[]{0,1})for(int policy=0;policy<3;policy++)
   for(boolean suppressed:new boolean[]{false,true})for(boolean onlySuffocating:new boolean[]{false,true}){
    cells.clear();for(int[] p:positions)cells.put(new BlockPos(p[0],p[1],p[2]),stateIndex);
    boolean active=policy!=0;
    Set<String> selected=switch(policy){case 1->Set.of("minecraft:cobweb","minecraft:snow");case 2->Set.of(BuiltInRegistries.BLOCK.getKey(states.get(stateIndex).getBlock()).toString());default->Set.of();};
    var scan=new BlockCollisions<JsonObject>(world,context(active,selected,suppressed,ghostY),new AABB(b[0],b[1],b[2],b[3],b[4],b[5]),onlySuffocating,(p,s)->{
     JsonObject hit=new JsonObject();hit.add("position",json(new int[]{p.getX(),p.getY(),p.getZ()}));hit.add("boxes",json(boxes(s)));return hit;});
    JsonArray hits=new JsonArray();while(scan.hasNext())hits.add(scan.next());
    JsonObject row=new JsonObject();row.addProperty("state",stateIndex);row.add("bounds",json(b));row.add("positions",json(positions));row.add("selected",json(selected));row.addProperty("active",active);row.addProperty("suppressed",suppressed);row.addProperty("onlySuffocating",onlySuffocating);row.addProperty("ghostFloor",ghostY);row.add("hits",hits);combined.add(row);
   }
  out.add("combinedScans",combined);
  TypeAdapter<Block> adapter=new TypeAdapter<>(){
   public Block read(JsonReader reader)throws java.io.IOException{return BuiltInRegistries.BLOCK.getValue(Identifier.tryParse(reader.nextString()));}
   public void write(JsonWriter writer,Block value)throws java.io.IOException{writer.value(BuiltInRegistries.BLOCK.getKey(value).toString());}
  };
  Gson typed=new GsonBuilder().registerTypeHierarchyAdapter(Block.class,adapter).create();JsonArray imports=new JsonArray();
  String[] inputs={"[]","\"snow\"","[\"snow\",\":snow\",\"cobweb\"]","[\"minecraft:missing\",\"BAD ID\",17]","null","true","[\"snow\",false]","[\"snow\",null]","[\"snow\",{}]","[[\"snow\"]]","{}","17","[\"stone\",\"snow\",\"air\"]"};
  for(String input:inputs){
   SortedSet<Block> current=new it.unimi.dsi.fastutil.objects.ObjectRBTreeSet<>(Comparator.comparing(BuiltInRegistries.BLOCK::getKey,Comparator.nullsFirst(Comparator.naturalOrder())));current.add(Blocks.COBWEB);current.add(Blocks.SNOW);boolean success=true;
   try{JsonElement element=JsonParser.parseString(input);List<Block> replacement=new ArrayList<>();if(element.isJsonArray())for(var entry:element.getAsJsonArray())replacement.add(typed.fromJson(entry,Block.class));else replacement.add(typed.fromJson(element,Block.class));current.clear();current.addAll(replacement);}catch(Exception failure){success=false;}
   JsonObject row=new JsonObject();row.add("input",JsonParser.parseString(input));row.addProperty("success",success);row.add("blocks",json(current.stream().map(b->b==null?null:BuiltInRegistries.BLOCK.getKey(b).toString()).toList()));imports.add(row);
  }
  out.add("imports",imports);Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(out)+"\n");
 }
 @SuppressWarnings({"unchecked","rawtypes"})static String propertyName(Property property,Comparable value){return property.getName(value);}
}
