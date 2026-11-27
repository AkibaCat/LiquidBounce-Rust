// SPDX-License-Identifier: GPL-3.0-or-later
// Actual MC26.3 registry class membership/AABB; source-derived module jump formula.
import com.google.gson.*;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.block.*;
import net.minecraft.world.phys.AABB;
import java.nio.file.*;

public class BlockBounceFacts {
    static long bits(float value) {return Integer.toUnsignedLong(Float.floatToRawIntBits(value));}
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
        JsonObject out=new JsonObject();JsonArray blocks=new JsonArray();
        for(var block:BuiltInRegistries.BLOCK) {
            JsonObject row=new JsonObject();row.addProperty("id",BuiltInRegistries.BLOCK.getKey(block).toString());
            row.addProperty("bouncy",block instanceof SlimeBlock || block instanceof BedBlock || block instanceof HoneyBlock);
            blocks.add(row);
        }
        out.add("blocks",blocks);JsonArray boxes=new JsonArray();
        double[][] coords={{-.1,64.,-.1,1.,65.8,1.},{1.,64.01,2.,2.,65.,3.},
            {-1.,-.001,-1.,0.,1.,0.},{.1,10.0099999999,.1,.7,11.81,.7},
            {.1,10.0100000001,.1,.7,11.81,.7},{29999999.5,50.,-29999999.5,30000000.,51.,-29999999.}};
        for(double[] c:coords) {
            AABB box=new AABB(c[0],c[1],c[2],c[3],c[4],c[5]);
            AABB detection=box.setMinY(box.minY-.01);
            JsonObject row=new JsonObject();row.add("bounds",new Gson().toJsonTree(c));
            JsonArray visited=new JsonArray();
            int y=(int)Math.floor(detection.minY);
            for(int x=(int)Math.floor(detection.minX);x<=(int)Math.floor(detection.maxX);x++) {
                for(int z=(int)Math.floor(detection.minZ);z<=(int)Math.floor(detection.maxZ);z++) {
                    visited.add(new Gson().toJsonTree(new int[]{x,y,z}));
                }
            }
            row.add("visited",visited);boxes.add(row);
        }
        out.add("boxes",boxes);JsonArray jumps=new JsonArray();
        for(float attribute:new float[]{0f,.42f,1.234567f}) for(float block:new float[]{.5f,1f})
            for(int boost:new int[]{-1,0,4}) for(float motion:new float[]{-3f,0f,.42f,2f,7.25f}) {
                float power=attribute*1f*block+(boost<0?0f:.1f*(boost+1f));
                float adjusted=power+motion;
                JsonObject row=new JsonObject();row.addProperty("attribute",attribute);row.addProperty("block",block);
                row.addProperty("boost",boost);row.addProperty("motion",motion);
                row.addProperty("powerBits",bits(power));row.addProperty("adjustedBits",bits(adjusted));jumps.add(row);
            }
        out.add("jumps",jumps);
        Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(out)+"\n");
    }
}
