// SPDX-License-Identifier: GPL-3.0-or-later
// Runs untouched LiquidBounce TweakedMethods against actual Minecraft shapes.
import com.google.gson.*;
import net.ccbluex.liquidbounce.common.TweakedMethods;
import net.ccbluex.liquidbounce.features.module.modules.exploit.ModuleGhostHand;
import net.minecraft.SharedConstants;
import net.minecraft.core.*;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.*;
import net.minecraft.world.level.block.*;
import net.minecraft.world.level.block.state.*;
import net.minecraft.world.level.block.state.properties.*;
import net.minecraft.world.phys.*;
import net.minecraft.world.phys.shapes.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;

public final class GhostHandFacts {
    static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    static final Map<BlockPos, Integer> CELLS = new HashMap<>();
    static final List<BlockState> STATES = new ArrayList<>();
    static BlockGetter world;
    static BlockState stateAt(BlockPos pos) { return STATES.get(CELLS.getOrDefault(pos, 0)); }
    static JsonElement json(Object value) { return GSON.toJsonTree(value); }
    static List<double[]> boxes(VoxelShape shape) {
        return shape.toAabbs().stream().map(b -> new double[]{b.minX,b.minY,b.minZ,b.maxX,b.maxY,b.maxZ}).toList();
    }
    static int add(BlockState state) { STATES.add(state); return STATES.size() - 1; }
    static int[] position(BlockPos pos) { return new int[]{pos.getX(),pos.getY(),pos.getZ()}; }
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        add(Blocks.AIR.defaultBlockState()); add(Blocks.STONE.defaultBlockState());
        add(Blocks.CHEST.defaultBlockState()); add(Blocks.ENDER_CHEST.defaultBlockState());
        add(Blocks.TRAPPED_CHEST.defaultBlockState()); add(Blocks.SHULKER_BOX.defaultBlockState());
        add(Blocks.DYED_SHULKER_BOX.red().defaultBlockState());
        add(Blocks.OAK_SLAB.defaultBlockState().setValue(BlockStateProperties.SLAB_TYPE, SlabType.BOTTOM));
        add(Blocks.OAK_SLAB.defaultBlockState().setValue(BlockStateProperties.SLAB_TYPE, SlabType.TOP));
        add(Blocks.WATER.defaultBlockState());
        add(Blocks.WATER.defaultBlockState().setValue(BlockStateProperties.LEVEL, 4));
        add(Blocks.LAVA.defaultBlockState()); add(Blocks.TORCH.defaultBlockState());
        add(Blocks.SNOW.defaultBlockState()); add(Blocks.GLASS.defaultBlockState());
        add(Blocks.POWDER_SNOW.defaultBlockState()); add(Blocks.STRUCTURE_VOID.defaultBlockState());
        add(Blocks.OAK_FENCE.defaultBlockState());
        world = (BlockGetter)Proxy.newProxyInstance(GhostHandFacts.class.getClassLoader(), new Class<?>[]{BlockGetter.class},
            (proxy, method, values) -> switch(method.getName()) {
                case "getBlockState" -> stateAt((BlockPos)values[0]);
                case "getFluidState" -> stateAt((BlockPos)values[0]).getFluidState();
                case "getBlockEntity" -> null;
                case "getMinY" -> -64;
                case "getHeight" -> 384;
                default -> { if(method.isDefault()) yield InvocationHandler.invokeDefault(proxy, method, values);
                    throw new UnsupportedOperationException(method.toString()); }
            });
        Set<Block> defaults = new HashSet<>(List.of(Blocks.CHEST, Blocks.ENDER_CHEST, Blocks.TRAPPED_CHEST, Blocks.SHULKER_BOX));
        defaults.addAll(Blocks.DYED_SHULKER_BOX.asList());
        JsonObject out = new JsonObject();
        out.add("defaults", json(defaults.stream().map(b -> BuiltInRegistries.BLOCK.getKey(b))
            .sorted().map(Object::toString).toList()));
        out.add("registry", json(BuiltInRegistries.BLOCK.keySet().stream().map(Object::toString).toList()));
        JsonArray metadata = new JsonArray();
        for (BlockState state : STATES) {
            JsonObject row = new JsonObject();
            row.addProperty("id", BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
            JsonObject shapes = new JsonObject();
            for (ClipContext.Block mode : ClipContext.Block.values()) {
                ClipContext context = new ClipContext(Vec3.ZERO, new Vec3(1,1,1), mode, ClipContext.Fluid.NONE, CollisionContext.empty());
                shapes.add(mode.name(), json(boxes(context.getBlockShape(state,world,BlockPos.ZERO))));
            }
            row.add("shapes", shapes);
            row.add("interaction", json(boxes(state.getInteractionShape(world,BlockPos.ZERO))));
            var fluid = state.getFluidState();
            row.addProperty("source", fluid.isSource()); row.addProperty("empty", fluid.isEmpty());
            row.addProperty("water", fluid.is(net.minecraft.tags.FluidTags.WATER));
            row.add("fluid", json(boxes(fluid.getShape(world,BlockPos.ZERO))));
            metadata.add(row);
        }
        out.add("states", metadata);
        JsonArray cases = new JsonArray();
        for (int state = 0; state < STATES.size(); state++)
        for (ClipContext.Block mode : ClipContext.Block.values())
        for (ClipContext.Fluid fluids : ClipContext.Fluid.values())
        for (int policy = 0; policy < 4; policy++)
        for (int variant = 0; variant < 4; variant++) {
            CELLS.clear();
            Vec3 start, end;
            switch (variant) {
                case 1 -> { CELLS.put(new BlockPos(3,0,0),1); CELLS.put(new BlockPos(1,0,0),state);
                    start=new Vec3(5.5,.5,.5); end=new Vec3(.5,.5,.5); }
                case 2 -> { CELLS.put(new BlockPos(1,1,1),1); CELLS.put(new BlockPos(3,3,3),state);
                    start=new Vec3(.5,.5,.5); end=new Vec3(5.5,5.5,5.5); }
                case 3 -> { CELLS.put(BlockPos.ZERO,state); start=new Vec3(.5,.5,.5);end=start; }
                default -> { CELLS.put(new BlockPos(1,0,0),1); CELLS.put(new BlockPos(3,0,0),state);
                    start=new Vec3(.5,.5,.5);end=new Vec3(5.5,.5,.5); }
            }
            var module=ModuleGhostHand.INSTANCE; module.running=policy!=0;
            module.targetedBlocks=switch(policy) {case 1 -> defaults;case 2 -> Set.of(STATES.get(state).getBlock());default -> Set.of();};
            var context=new ClipContext(start,end,mode,fluids,CollisionContext.empty());
            var hit=TweakedMethods.tweakedRaycast(world,context);
            JsonObject row=new JsonObject(); row.addProperty("mode",mode.name());row.addProperty("fluids",fluids.name());
            row.addProperty("running",module.running);
            row.add("targets",json(module.targetedBlocks.stream().map(b->BuiltInRegistries.BLOCK.getKey(b).toString()).sorted().toList()));
            row.add("start",json(new double[]{start.x,start.y,start.z}));row.add("end",json(new double[]{end.x,end.y,end.z}));
            JsonArray cells=new JsonArray();CELLS.forEach((p,s)-> {JsonObject cell=new JsonObject();cell.add("pos",json(position(p)));cell.addProperty("state",s);cells.add(cell);});
            row.add("cells",cells);JsonObject result=new JsonObject();
            result.addProperty("miss",hit.getType()==HitResult.Type.MISS);result.add("pos",json(position(hit.getBlockPos())));
            var point=hit.getLocation();result.add("point",json(new double[]{point.x,point.y,point.z}));
            var normal=hit.getDirection().getUnitVec3i();result.add("face",json(new int[]{normal.getX(),normal.getY(),normal.getZ()}));
            result.addProperty("inside",hit.isInside());row.add("result",result);cases.add(row);
        }
        out.add("cases",cases);
        Files.writeString(Path.of(args[0]),GSON.toJson(out)+"\n");
    }
}
