// SPDX-License-Identifier: GPL-3.0-or-later
// Source-derived behavior formula with actual MC26.3 Mth/InputConstants/PoseStack.
import com.google.gson.*;
import com.mojang.blaze3d.platform.InputConstants;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.math.Axis;
import net.minecraft.util.Mth;
import java.nio.file.*;
import java.util.*;

public class CameraClipDankFacts {
    static long bits(float value) { return Integer.toUnsignedLong(Float.floatToRawIntBits(value)); }
    static float easing(int choice,float x) {
        return switch(choice) {
            case 0 -> x;
            case 1 -> x*x;
            case 2 -> 1-(1-x)*(1-x);
            case 3 -> 2*(1-x)*x*x+x*x;
            case 4 -> x==0f?0f:(float)Math.pow(2f,10f*x-10f);
            case 5 -> x==1f?1f:1f-(float)Math.pow(2f,-10f*x);
            default -> 1f;
        };
    }
    public static void main(String[] args) throws Exception {
        JsonObject out=new JsonObject();out.addProperty("leftControl",InputConstants.KEY_LCONTROL);
        out.addProperty("unknown",InputConstants.UNKNOWN.getValue());
        JsonArray keys=new JsonArray();
        for(String name:Files.readAllLines(Path.of(args[1]))) {
            JsonObject row=new JsonObject();row.addProperty("input",name);
            try {var key=InputConstants.getKey(name);row.addProperty("name",key.getName());
                row.addProperty("value",key.getValue());row.addProperty("keyboard",key.getType()==InputConstants.Type.KEYBOARD);
            } catch(Exception exception) {row.addProperty("error",exception.getClass().getSimpleName());}
            keys.add(row);
        }
        out.add("keys",keys);
        JsonArray animations=new JsonArray();
        for(int choice=0;choice<7;choice++) for(float speed:new float[]{0f,.1f,.6f,1f,1.25f,-.3f}) {
            JsonObject row=new JsonObject();row.addProperty("choice",choice);row.addProperty("speed",speed);
            row.addProperty("lastBits",bits(2.4f));row.addProperty("targetBits",bits(14.5f));
            row.addProperty("resultBits",bits(Mth.lerp(easing(choice,speed),2.4f,14.5f)));
            animations.add(row);
        }
        out.add("animations",animations);
        JsonArray matrices=new JsonArray();
        float[][] inputs={{0,0,5},{1,.05f,5},{1.2345f,.1f,5},{1000,.13f,50},{-.3f,-.07f,1},
            {6.1f,.08f,0},{.001f,.2f,-20},{4,.31f,72.25f},{.7f,.17f,3.2f}};
        for(boolean hurt:new boolean[]{false,true}) for(float[] in:inputs) {
            float g=in[0],h=in[1],motion=in[2];PoseStack pose=new PoseStack();
            if(hurt) {float f=3.5f/10f;f=Mth.sin(f*f*f*f*Mth.PI);
                pose.rotateDegrees(Axis.YP,-32.5f);pose.rotateDegrees(Axis.ZP,(float)((double)-f*14.0*.73));pose.rotateDegrees(Axis.YP,32.5f);}
            pose.translate(Mth.sin(g*Mth.PI)*h*.5f,-Math.abs(Mth.cos(g*Mth.PI)*h),0f);
            pose.rotate(Axis.ZP.rotationDegrees(Mth.sin(h*Mth.PI)*h*(3f+motion)));
            pose.rotate(Axis.XP.rotationDegrees(Math.abs(Mth.cos(h*Mth.PI-(.2f+motion))*h)*5f));
            JsonObject row=new JsonObject();row.addProperty("walk",g);row.addProperty("bob",h);row.addProperty("motion",motion);row.addProperty("hurt",hurt);
            float[] matrix=new float[16];pose.last().pose().get(matrix);JsonArray values=new JsonArray();
            for(float value:matrix)values.add(bits(value));row.add("matrixBits",values);matrices.add(row);
        }
        out.add("matrices",matrices);
        Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(out)+"\n");
    }
}
