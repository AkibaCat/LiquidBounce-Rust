// SPDX-License-Identifier: GPL-3.0-or-later
// Actual 26.3 entity classes, evaluated in CombatExtensions.isInteresting order.
import com.google.gson.*;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.*;
import net.minecraft.world.entity.ambient.Bat;
import net.minecraft.world.entity.animal.allay.Allay;
import net.minecraft.world.entity.animal.fish.WaterAnimal;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.entity.monster.Enemy;
import net.minecraft.world.entity.monster.Monster;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.phys.AABB;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;

public class TargetEntityFacts {
    static String category(Class<?> type) {
        if (Player.class.isAssignableFrom(type)) return "Players";
        if (WaterAnimal.class.isAssignableFrom(type)) return "WaterCreature";
        if (AgeableMob.class.isAssignableFrom(type) || Bat.class.isAssignableFrom(type) || Allay.class.isAssignableFrom(type)) return "Passive";
        if (ArmorStand.class.isAssignableFrom(type)) return "ArmorStand";
        if (Monster.class.isAssignableFrom(type) || Enemy.class.isAssignableFrom(type)) return "Hostile";
        if (NeutralMob.class.isAssignableFrom(type)) return "Angerable";
        return "None";
    }
    static JsonArray bits(AABB box) {
        JsonArray bits=new JsonArray();
        for(double value:new double[]{box.minX,box.minY,box.minZ,box.maxX,box.maxY,box.maxZ}) bits.add(Long.toUnsignedString(Double.doubleToRawLongBits(value)));
        return bits;
    }
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        TreeMap<String,JsonObject> ordered=new TreeMap<>();
        for(Field field:EntityTypes.class.getFields()) {
            if (field.getType()!=EntityType.class) continue;
            var signature=(ParameterizedType)field.getGenericType();
            Class<?> type=(Class<?>)signature.getActualTypeArguments()[0];
            EntityType<?> entityType=(EntityType<?>)field.get(null);
            JsonObject row=new JsonObject();
            row.addProperty("type",EntityType.getKey(entityType).toString());
            row.addProperty("class",type.getName());
            row.addProperty("living",LivingEntity.class.isAssignableFrom(type));
            row.addProperty("attackable",Attackable.class.isAssignableFrom(type));
            row.addProperty("category",category(type));
            ordered.put(row.get("type").getAsString(),row);
        }
        JsonObject out=new JsonObject();JsonArray rows=new JsonArray();ordered.values().forEach(rows::add);out.add("entities",rows);
        JsonArray margins=new JsonArray();
        for(float size:new float[]{0f,.1f,1f,-.2f,12.75f}) {
            AABB original=new AABB(-.3,64,-.3,.3,65.8,.3);
            JsonObject row=new JsonObject();row.addProperty("size",size);row.add("inflatedBits",bits(original.inflate((double)size)));
            row.addProperty("componentMarginBits",Integer.toUnsignedLong(Float.floatToRawIntBits(size+.125f)));
            margins.add(row);
        }
        out.add("margins",margins);
        Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(out)+"\n");
    }
}
