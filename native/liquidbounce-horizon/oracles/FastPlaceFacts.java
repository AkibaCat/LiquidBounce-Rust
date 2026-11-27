// Developer oracle only. Reads item classes from the user's pinned game jar.
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.file.Files;
import java.nio.file.Path;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.ProjectileItem;

public final class FastPlaceFacts {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var items = new JsonArray();
        for (var item : BuiltInRegistries.ITEM) {
            var row = new JsonObject();
            row.addProperty("id", BuiltInRegistries.ITEM.getKey(item).toString());
            row.addProperty("projectile", item instanceof ProjectileItem);
            row.addProperty("block", item instanceof BlockItem);
            items.add(row);
        }
        var report = new JsonObject();
        report.addProperty("version", "26.3");
        report.add("items", items);
        Files.writeString(Path.of(args[0]), new GsonBuilder().setPrettyPrinting().create().toJson(report));
    }
}
