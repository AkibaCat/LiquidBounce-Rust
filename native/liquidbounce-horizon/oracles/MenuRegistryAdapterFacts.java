// Developer oracle. The adapter read/write bodies follow the pinned Kotlin
// IdentifierWithRegistryAdapter; Gson and the actual Menu registry are real.
import com.google.gson.*;
import com.google.gson.reflect.TypeToken;
import com.google.gson.stream.JsonReader;
import com.google.gson.stream.JsonWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.inventory.MenuType;

public final class MenuRegistryAdapterFacts {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var adapter = new TypeAdapter<MenuType<?>>() {
            public MenuType<?> read(JsonReader source) throws IOException {
                return BuiltInRegistries.MENU.getValue(Identifier.tryParse(source.nextString()));
            }
            public void write(JsonWriter sink, MenuType<?> value) throws IOException {
                var id = value == null ? null : BuiltInRegistries.MENU.getKey(value);
                if (id == null) sink.nullValue(); else sink.value(id.toString());
            }
        };
        var gson = new GsonBuilder().registerTypeHierarchyAdapter(MenuType.class, adapter).create();
        var rows = new JsonArray();
        for (var json : List.of("null", "[null]", "\"minecraft:generic_9x3\"", "\"generic_9x3\"",
                "\"minecraft:missing\"", "\"bad ID\"", "123", "true")) {
            var row = new JsonObject(); row.addProperty("input", json);
            try {
                if (json.startsWith("[")) {
                    List<MenuType<?>> result = gson.fromJson(JsonParser.parseString(json), new TypeToken<List<MenuType<?>>>() {}.getType());
                    row.add("output", gson.toJsonTree(result));
                } else {
                    MenuType<?> result = gson.fromJson(JsonParser.parseString(json), MenuType.class);
                    row.addProperty("null", result == null);
                    if (result != null) row.addProperty("id", BuiltInRegistries.MENU.getKey(result).toString());
                }
            } catch (Throwable error) {
                row.addProperty("error", error.getClass().getSimpleName());
                row.addProperty("message", error.getMessage());
            }
            rows.add(row);
        }
        var menus = new JsonArray();
        for (var menu : BuiltInRegistries.MENU) menus.add(BuiltInRegistries.MENU.getKey(menu).toString());
        var report = new JsonObject(); report.add("cases", rows); report.add("menuIds", menus);
        report.addProperty("gson", Gson.class.getProtectionDomain().getCodeSource().getLocation().toString());
        Files.writeString(Path.of(args[0]), new GsonBuilder().setPrettyPrinting().create().toJson(report));
    }
}
