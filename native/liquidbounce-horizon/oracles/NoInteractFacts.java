// Developer oracle: real 26.3 registries, classes, tags and Kotlin/Fastutil lookup.
import com.google.gson.*;
import it.unimi.dsi.fastutil.objects.ObjectRBTreeSet;
import java.nio.file.*;
import java.util.*;
import kotlin.comparisons.ComparisonsKt;
import com.google.gson.stream.*;
import net.minecraft.SharedConstants;
import net.minecraft.core.*;
import net.minecraft.core.registries.*;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.tags.*;
import net.minecraft.world.entity.*;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.item.*;

public final class NoInteractFacts {
    private static final String[] ITEM_CLASSES = {
        "BoatItem", "BowItem", "ArrowItem", "BucketItem", "CrossbowItem", "EggItem", "EmptyMapItem",
        "EnderEyeItem", "EnderpearlItem", "ExperienceBottleItem", "FireworkRocketItem", "FishingRodItem",
        "BottleItem", "InstrumentItem", "KnowledgeBookItem", "PlaceOnWaterBlockItem", "SnowballItem",
        "SpawnEggItem", "SpyglassItem", "TridentItem", "WindChargeItem", "WritableBookItem", "WrittenBookItem",
        "ArmorStandItem", "BlockItem", "BrushItem", "HangingEntityItem", "FireChargeItem", "FlintAndSteelItem", "PotionItem"
    };
    private static final String[] MINING_TAGS = {"axes", "pickaxes", "shovels", "hoes"};
    private static final String[] BLOCK_CLASSES = {
        "AbstractBedBlock", "AbstractChestBlock", "AbstractFurnaceBlock", "AnvilBlock", "BarrelBlock", "BeaconBlock", "BellBlock", "BrewingStandBlock", "ButtonBlock", "CakeBlock", "CandleCakeBlock", "CartographyTableBlock", "CaveVinesPlantBlock", "CaveVinesBlock", "ComparatorBlock", "ComposterBlock", "CrafterBlock", "CraftingTableBlock", "DaylightDetectorBlock", "DecoratedPotBlock", "DispenserBlock", "DoorBlock", "DragonEggBlock", "EnchantingTableBlock", "FenceGateBlock", "FlowerPotBlock", "GrindstoneBlock", "HopperBlock", "GameMasterBlock", "JukeboxBlock", "LecternBlock", "LeverBlock", "LightBlock", "NoteBlock", "RedstoneWireBlock", "RepeaterBlock", "RespawnAnchorBlock", "ShulkerBoxBlock", "StonecutterBlock", "SweetBerryBushBlock", "TrapDoorBlock"
    };
    private static final Map<String, List<Holder<Item>>> TAGS = new HashMap<>();
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonArray registryImports(Registry registry, Class target) {
        var gson = new GsonBuilder().registerTypeHierarchyAdapter(target, new TypeAdapter<Object>() {
            public Object read(JsonReader source) throws java.io.IOException { return registry.getValue(Identifier.tryParse(source.nextString())); }
            public void write(JsonWriter sink, Object value) throws java.io.IOException {
                var id = value == null ? null : registry.getKey(value);
                if (id == null) sink.nullValue(); else sink.value(id.toString());
            }
        }).create();
        Comparator<Object> comparator = (a, b) -> ComparisonsKt.compareValues(registry.getKey(a), registry.getKey(b));
        var report = new JsonArray();
        for (var text : List.of("null", "[null]", "true", "[\"minecraft:villager\",null,\"minecraft:armor_stand\"]", "[\"minecraft:pig\",true]", "[\"absent:unknown\",\"bad id\",17]", "[]")) {
            var set = new ObjectRBTreeSet<Object>(comparator);set.add(registry.getValue(Identifier.parse("minecraft:villager")));set.add(registry.getValue(Identifier.parse("minecraft:armor_stand")));
            var row = new JsonObject();row.addProperty("input", text);
            try {
                var element = JsonParser.parseString(text);var parsed = new ArrayList<Object>();
                if (element.isJsonArray()) for (var value : element.getAsJsonArray()) parsed.add(gson.fromJson(value, target));
                else parsed.add(gson.fromJson(element, target));
                set.clear();set.addAll(parsed);row.addProperty("success",true);
            } catch (Exception error) {row.addProperty("success",false);row.addProperty("error",error.getClass().getName());}
            var after = new JsonArray();for (var value : set) after.add(gson.toJsonTree(value, target));row.add("after",after);report.add(row);
        }
        return report;
    }
    private static List<Holder<Item>> tag(String name) throws Exception {
        if (TAGS.containsKey(name)) return TAGS.get(name);
        var values = new LinkedHashSet<Holder<Item>>();
        var path = "/data/minecraft/tags/item/" + name + ".json";
        try (var source = NoInteractFacts.class.getResourceAsStream(path)) {
            if (source == null) throw new IllegalStateException("Missing official tag " + path);
            for (var value : JsonParser.parseString(new String(source.readAllBytes(), java.nio.charset.StandardCharsets.UTF_8)).getAsJsonObject().getAsJsonArray("values")) {
                var id = value.isJsonObject() ? value.getAsJsonObject().get("id").getAsString() : value.getAsString();
                if (id.startsWith("#")) values.addAll(tag(id.substring(1).replace("minecraft:", "")));
                else values.add(BuiltInRegistries.ITEM.get(Identifier.parse(id)).orElseThrow());
            }
        }
        var result = List.copyOf(values); TAGS.put(name, result); return result;
    }
    @SuppressWarnings({"rawtypes", "unchecked"})
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        var registry = (Registry) BuiltInRegistries.ENTITY_TYPE;
        var comparator = (Comparator<Object>) (a, b) -> ComparisonsKt.compareValues(registry.getKey(a), registry.getKey(b));
        var set = new ObjectRBTreeSet<Object>(comparator);
        set.add(EntityTypes.VILLAGER); set.add(EntityTypes.ARMOR_STAND);
        var unsafeField = sun.misc.Unsafe.class.getDeclaredField("theUnsafe"); unsafeField.setAccessible(true);
        var unsafe = (sun.misc.Unsafe) unsafeField.get(null);
        var entity = (Entity) unsafe.allocateInstance(ArmorStand.class);
        var membership = new JsonObject();
        membership.addProperty("entityClass", entity.getClass().getName());
        var key = registry.getKey(entity);
        membership.addProperty("entityRegistryFallback", key == null ? null : key.toString());
        membership.addProperty("defaultEntitySetContainsInstance", set.contains(entity));
        set.add(EntityTypes.PIG);
        membership.addProperty("withPigContainsInstance", set.contains(entity));
        set.clear(); membership.addProperty("emptyContainsInstance", set.contains(entity));
        membership.addProperty("unknownTypeFallback", BuiltInRegistries.ENTITY_TYPE.getKey(BuiltInRegistries.ENTITY_TYPE.getValue(Identifier.parse("absent:unknown"))).toString());
        membership.addProperty("nullTypeFallback", BuiltInRegistries.ENTITY_TYPE.getKey(BuiltInRegistries.ENTITY_TYPE.getValue((Identifier) null)).toString());
        membership.addProperty("unknownItemFallback", BuiltInRegistries.ITEM.getKey(BuiltInRegistries.ITEM.getValue(Identifier.parse("absent:unknown"))).toString());
        var bound = new HashMap<TagKey<Item>, List<Holder<Item>>>();
        for (var name : MINING_TAGS) bound.put(TagKey.create(Registries.ITEM, Identifier.withDefaultNamespace(name)), tag(name));
        BuiltInRegistries.ITEM.prepareTagReload(new TagLoader.LoadResult<>(Registries.ITEM, bound)).apply();
        var defaults = new TreeSet<String>(List.of("minecraft:air", "minecraft:shears", "minecraft:tnt", "minecraft:water_bucket", "minecraft:lava_bucket", "minecraft:cobweb"));
        var items = new JsonArray();
        for (var item : BuiltInRegistries.ITEM) {
            var id = BuiltInRegistries.ITEM.getKey(item).toString();
            var stack = item == Items.AIR ? null : new ItemStackTemplate(item);
            boolean mining = stack != null && (stack.is(ItemTags.AXES) || stack.is(ItemTags.PICKAXES) || stack.is(ItemTags.SHOVELS) || stack.is(ItemTags.HOES));
            if (mining) defaults.add(id);
            var row = new JsonObject(); row.addProperty("id", id); row.addProperty("miningTool", mining);
            var classes = new JsonArray();
            for (var name : ITEM_CLASSES) if (Class.forName("net.minecraft.world.item." + name).isInstance(item)) classes.add(name);
            row.add("classes", classes); items.add(row);
        }
        var blocks = new JsonArray();
        for (var block : BuiltInRegistries.BLOCK) {
            var row = new JsonObject(); row.addProperty("id", BuiltInRegistries.BLOCK.getKey(block).toString());
            var classes = new JsonArray();
            for (var name : BLOCK_CLASSES) if (Class.forName("net.minecraft.world.level.block." + name).isInstance(block)) classes.add(name);
            row.add("classes", classes); blocks.add(row);
        }
        var report = new JsonObject(); report.addProperty("version", "26.3"); report.add("blocks", blocks);
        report.add("entityRegistryImports", registryImports(BuiltInRegistries.ENTITY_TYPE, EntityType.class));
        report.add("itemRegistryImports", registryImports(BuiltInRegistries.ITEM, Item.class));
        report.add("entityFilter", membership); report.add("defaultHoldingItems", new Gson().toJsonTree(defaults)); report.add("items", items);
        Files.writeString(Path.of(args[0]), new GsonBuilder().setPrettyPrinting().create().toJson(report));
    }
}
