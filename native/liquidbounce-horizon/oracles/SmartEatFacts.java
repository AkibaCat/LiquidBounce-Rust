// Read the actual 26.3 Item.getUseAnimation dispatch owners.
import com.google.gson.*;
import java.nio.file.*;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.item.*;

public final class SmartEatFacts {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
        var root=new JsonObject();var overrides=new JsonArray();var uses=new JsonArray();
        for(var item:BuiltInRegistries.ITEM) {
            var owner=item.getClass().getMethod("getUseAnimation",ItemStack.class).getDeclaringClass();
            if(owner!=Item.class) {
                var row=new JsonObject();row.addProperty("item",BuiltInRegistries.ITEM.getKey(item).toString());
                row.addProperty("owner",owner.getSimpleName());overrides.add(row);
            }
            var useOwner=item.getClass().getMethod("use",net.minecraft.world.level.Level.class,net.minecraft.world.entity.player.Player.class,net.minecraft.world.InteractionHand.class).getDeclaringClass();
            var use=new JsonObject();use.addProperty("item",BuiltInRegistries.ITEM.getKey(item).toString());use.addProperty("owner",useOwner.getSimpleName());uses.add(use);
        }
        root.add("animationOverrides",overrides);
        root.add("useOwners",uses);
        Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(root));
    }
}
