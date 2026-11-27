// Actual MC26.3 ItemStack component access and getMenuProvider dispatch owners.
import com.google.gson.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.SharedConstants;
import net.minecraft.core.*;
import net.minecraft.core.component.*;
import net.minecraft.core.registries.*;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.item.*;
import net.minecraft.world.level.*;
import net.minecraft.world.level.block.*;
import net.minecraft.world.level.block.state.*;

public final class AutoToolFacts {
    private static String menuOwner(Class<?> type) throws Exception {
        while (type != null) {
            try {type.getDeclaredMethod("getMenuProvider",BlockState.class,Level.class,BlockPos.class);return type.getSimpleName();}
            catch (NoSuchMethodException absent) {type=type.getSuperclass();}
        }
        throw new IllegalStateException("No getMenuProvider");
    }
    private static JsonObject row(String name,ItemStack stack) {
        var row=new JsonObject();row.addProperty("name",name);row.addProperty("maxDamage",stack.getMaxDamage());
        row.addProperty("damage",stack.getDamageValue());row.addProperty("damageable",stack.isDamageableItem());return row;
    }
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
        for(var item:BuiltInRegistries.ITEM) item.builtInRegistryHolder().bindComponents(DataComponentMap.EMPTY);
        var root=new JsonObject();var blocks=new JsonArray();
        for (var block:BuiltInRegistries.BLOCK) {
            var row=new JsonObject();row.addProperty("id",BuiltInRegistries.BLOCK.getKey(block).toString());
            row.addProperty("menuOwner",menuOwner(block.getClass()));
            if(block instanceof EntityBlock entity){var be=entity.newBlockEntity(BlockPos.ZERO,block.defaultBlockState());row.addProperty("blockEntity",be==null?null:be.getClass().getSimpleName());row.addProperty("entityIsMenu",be instanceof net.minecraft.world.MenuProvider);}
            blocks.add(row);
        }
        root.add("blocks",blocks);var stacks=new JsonArray();
        // Bind explicit fixture prototypes; getMaxDamage/getDamageValue below
        // execute the actual client methods, independently of item defaults.
        Items.DIAMOND_PICKAXE.builtInRegistryHolder().bindComponents(DataComponentMap.builder().set(DataComponents.MAX_DAMAGE,1561).set(DataComponents.DAMAGE,0).build());
        Items.STICK.builtInRegistryHolder().bindComponents(DataComponentMap.EMPTY);
        var stack=new ItemStack(Items.DIAMOND_PICKAXE);stacks.add(row("explicitPrototype",stack));
        stack.set(DataComponents.UNBREAKABLE,net.minecraft.util.Unit.INSTANCE);stack.setDamageValue(1559);stacks.add(row("unbreakable",stack));
        stack.remove(DataComponents.DAMAGE);stacks.add(row("removedDamage",stack));
        stack.remove(DataComponents.MAX_DAMAGE);stacks.add(row("removedMaxDamage",stack));
        stack=new ItemStack(Items.STICK);stack.set(DataComponents.MAX_DAMAGE,20);stacks.add(row("customMaxOnly",stack));
        stack.set(DataComponents.DAMAGE,99);stacks.add(row("customDamageClamped",stack));
        root.add("stacks",stacks);
        var comparisons=new JsonArray();
        for (int level:new int[]{0,1,5,31,32,46340,46341,Integer.MIN_VALUE,Integer.MAX_VALUE}) {
            float speed=2f;if(speed>1f&&level!=0)speed+=Math.clamp((float)(level*level)+1f,0f,1024f);
            var row=new JsonObject();row.addProperty("level",level);row.addProperty("speed",speed);comparisons.add(row);
        }
        root.add("efficiency",comparisons);
        var menu=new net.minecraft.world.inventory.InventoryMenu(new net.minecraft.world.entity.player.Inventory(null,new net.minecraft.world.entity.EntityEquipment()),true,null);
        var positions=new JsonArray();
        for(var slot:menu.slots){var xy=new JsonArray();xy.add(slot.x);xy.add(slot.y);positions.add(xy);}
        root.add("inventoryMenuSlots",positions);
        var center=new JsonArray();
        center.add((menu.slots.stream().mapToInt(s->s.x).min().orElseThrow()+menu.slots.stream().mapToInt(s->s.x).max().orElseThrow())/2f+8f);
        center.add((menu.slots.stream().mapToInt(s->s.y).min().orElseThrow()+menu.slots.stream().mapToInt(s->s.y).max().orElseThrow())/2f+8f);
        root.add("inventoryMenuCenter",center);
        Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(root));
    }
}
