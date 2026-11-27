// SPDX-License-Identifier: GPL-3.0-or-later
// Actual MC26.3 stack components; policy oracle is collected from untouched Kotlin.
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.SharedConstants;
import net.minecraft.core.component.*;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.Unit;
import net.minecraft.world.item.*;
import java.nio.file.*;
import java.util.*;

public final class ElytraRecastFacts {
    static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    static void stack(JsonArray out, String label, ItemStack stack) {
        JsonObject row = new JsonObject();
        row.addProperty("label", label);
        row.addProperty("item", BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
        row.addProperty("count", stack.getCount());
        row.add("components", DataComponentMap.CODEC.encodeStart(JsonOps.INSTANCE, stack.getComponents()).getOrThrow());
        row.addProperty("glider", stack.has(DataComponents.GLIDER));
        row.addProperty("nextDamageWillBreak", stack.nextDamageWillBreak());
        row.addProperty("damageable", stack.isDamageableItem());
        out.add(row);
    }
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        // 26.3 loads item prototypes as data. Bind the four relevant components
        // from the user's prepared MC report; the others cannot affect this query.
        JsonObject prototypes = JsonParser.parseString(Files.readString(Path.of(args[1]))).getAsJsonObject();
        for (Item item : List.of(Items.AIR, Items.ELYTRA, Items.DIAMOND_CHESTPLATE, Items.STICK)) {
            JsonObject p = prototypes.getAsJsonObject(BuiltInRegistries.ITEM.getKey(item).toString());
            DataComponentMap.Builder b = DataComponentMap.builder();
            if (p.has("minecraft:glider")) b.set(DataComponents.GLIDER, Unit.INSTANCE);
            if (p.has("minecraft:max_damage")) b.set(DataComponents.MAX_DAMAGE, p.get("minecraft:max_damage").getAsInt());
            if (p.has("minecraft:damage")) b.set(DataComponents.DAMAGE, p.get("minecraft:damage").getAsInt());
            if (p.has("minecraft:unbreakable")) b.set(DataComponents.UNBREAKABLE, Unit.INSTANCE);
            item.builtInRegistryHolder().bindComponents(b.build());
        }
        JsonArray stacks = new JsonArray();
        for (Item item : List.of(Items.ELYTRA, Items.DIAMOND_CHESTPLATE, Items.STICK)) {
            for (int glider = 0; glider < 3; glider++) {
                for (int damage : new int[]{-1, 0, 1, 3, 4, 431, 432, 500}) {
                    for (int durability = 0; durability < 5; durability++) {
                        ItemStack s = new ItemStack(item);
                        if (glider == 1) s.set(DataComponents.GLIDER, Unit.INSTANCE);
                        if (glider == 2) s.remove(DataComponents.GLIDER);
                        if (durability == 1) s.set(DataComponents.MAX_DAMAGE, 5);
                        if (durability == 2) s.remove(DataComponents.MAX_DAMAGE);
                        if (durability == 3) s.set(DataComponents.UNBREAKABLE, Unit.INSTANCE);
                        if (damage >= 0) s.set(DataComponents.DAMAGE, damage);
                        if (durability == 4) s.remove(DataComponents.DAMAGE);
                        stack(stacks, item.toString()+":"+glider+":"+damage+":"+durability, s);
                    }
                }
            }
        }
        // Empty getComponents/getItem behavior differs from a count-only shell.
        ItemStack empty = new ItemStack(Items.ELYTRA, 0); stack(stacks, "zero-count", empty);
        ItemStack wrongSlot = new ItemStack(Items.STICK);
        wrongSlot.set(DataComponents.GLIDER, Unit.INSTANCE);
        wrongSlot.remove(DataComponents.EQUIPPABLE); stack(stacks, "no-equippable", wrongSlot);
        JsonObject result = new JsonObject();
        result.addProperty("prototypeSource", "four relevant components from prepared MC26.3 items.json; other components unbound");
        result.add("stacks", stacks);
        Files.writeString(Path.of(args[0]), GSON.toJson(result));
    }
}
