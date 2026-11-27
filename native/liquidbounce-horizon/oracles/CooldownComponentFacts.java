// SPDX-License-Identifier: GPL-3.0-or-later
// Actual pinned UseCooldown stream codec, ticks and identifier normalization.
import com.google.gson.*;
import io.netty.buffer.Unpooled;
import net.minecraft.core.RegistryAccess;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.resources.Identifier;
import net.minecraft.world.item.component.UseCooldown;
import java.nio.file.*;
import java.util.*;

public final class CooldownComponentFacts {
    public static void main(String[] args) throws Exception {
        JsonArray rows = new JsonArray();
        for (float seconds : new float[]{0.0f, 0.05f, 0.1f, 0.3f, 1.0f, 4.0f, 30.0f,
                Float.MIN_VALUE, 107374184.0f, Float.MAX_VALUE}) {
            for (String group : new String[]{null, "liquidbounce:integration", "ender_pearl", "minecraft:shield"}) {
                var expected = new UseCooldown(seconds, Optional.ofNullable(group).map(Identifier::parse));
                var buffer = new RegistryFriendlyByteBuf(Unpooled.buffer(), RegistryAccess.EMPTY);
                try {
                    UseCooldown.STREAM_CODEC.encode(buffer, expected);
                    byte[] bytes = new byte[buffer.readableBytes()];
                    buffer.getBytes(buffer.readerIndex(), bytes);
                    var decoded = UseCooldown.STREAM_CODEC.decode(buffer);
                    if (buffer.readableBytes() != 0 || !decoded.equals(expected)) {
                        throw new IllegalStateException("Actual codec round trip changed UseCooldown");
                    }
                    JsonObject row = new JsonObject();
                    row.addProperty("bytes", HexFormat.of().formatHex(bytes));
                    row.addProperty("secondsBits", Float.floatToRawIntBits(decoded.seconds()));
                    row.addProperty("ticks", decoded.ticks());
                    if (group != null) row.addProperty("savedGroup", group);
                    if (decoded.cooldownGroup().isPresent()) {
                        row.addProperty("group", decoded.cooldownGroup().get().toString());
                    } else {
                        row.add("group", JsonNull.INSTANCE);
                    }
                    rows.add(row);
                } finally { buffer.release(); }
            }
        }
        Files.writeString(Path.of(args[0]), new GsonBuilder().setPrettyPrinting().create().toJson(rows));
    }
}
