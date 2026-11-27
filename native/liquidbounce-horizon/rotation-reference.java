// SPDX-License-Identifier: GPL-3.0-or-later
// JVM reference expressions from pinned LiquidBounce Rotation.kt,
// RotationUtil.kt, RequestHandler.kt and MixinKeyboardInput.java.
// Uses actual Kotlin MathKt.roundToInt; run with the local kotlin-stdlib.
import java.util.*;

class RotationReference {
    static float wrap(float value) {
        float out = value % 360.0F;
        if (out >= 180.0F) out -= 360.0F;
        if (out < -180.0F) out += 360.0F;
        return out;
    }
    static double gcd(double sensitivity) {
        double f = sensitivity * 0.6F + 0.2F;
        return (double) ((float)(f * f * f * 8.0) * 0.15F);
    }
    static float[] normalize(float yaw, float pitch, float currentYaw, float currentPitch, double gcd) {
        float dy = wrap(yaw - currentYaw);
        float dp = pitch - currentPitch;
        double g1 = kotlin.math.MathKt.roundToInt((double)dy / gcd) * gcd;
        double g2 = kotlin.math.MathKt.roundToInt((double)dp / gcd) * gcd;
        return new float[] {currentYaw + (float)g1, Math.max(-90.0F,Math.min(90.0F,currentPitch+(float)g2))};
    }
    static int corrected(int flags, float sin, float cos) {
        float z = ((flags&1)!=0?1.0F:0.0F) - ((flags&2)!=0?1.0F:0.0F);
        float x = ((flags&4)!=0?1.0F:0.0F) - ((flags&8)!=0?1.0F:0.0F);
        int sx = Math.round(x*cos - z*sin);
        int sz = Math.round(z*cos + x*sin);
        return (sz>0?1:0)|(sz<0?2:0)|(sx>0?4:0)|(sx<0?8:0);
    }
    record Request(long provider,int priority,long expiresAt,String value) {}
    static String priorityTrace() {
        // Same comparator and heap operations as RequestHandler, including
        // equal-priority requests and bulk remove/heapify provider updates.
        var queue = new java.util.concurrent.PriorityBlockingQueue<Request>(11,Comparator.comparingInt(r->~r.priority));
        for(int i=1;i<=5;i++) queue.add(new Request(i,0,5,"p"+i));
        queue.removeIf(r->r.provider==3);
        queue.add(new Request(3,0,5,"new3"));
        var trace = new StringJoiner(",");
        while(!queue.isEmpty()) trace.add(queue.remove().value);
        return trace.toString();
    }
    public static void main(String[] args) {
        System.out.println("{");
        System.out.print("\"gcdBits\":[");
        double[] sensitivities = {0.0,0.1,0.5,0.9,1.0};
        for(int i=0;i<sensitivities.length;i++) {
            if(i>0)System.out.print(",");
            System.out.print("\""+Long.toHexString(Double.doubleToRawLongBits(gcd(sensitivities[i])))+"\"");
        }
        System.out.println("],");
        float[][] values = {
            {180.0F,45.0F,0.0F,0.0F},
            {-179.92F,90.0F,179.95F,-90.0F},
            {10.0F,-10.0F,350.0F,15.0F},
            {0.001F,-0.001F,0.0F,0.0F},
            {179.99F,89.99F,-179.99F,-89.99F}
        };
        System.out.print("\"normalizedBits\":[");
        for(int i=0;i<values.length;i++) {
            if(i>0)System.out.print(",");
            var v=values[i];var out=normalize(v[0],v[1],v[2],v[3],gcd(0.5));
            System.out.print("[\""+Integer.toHexString(Float.floatToRawIntBits(out[0]))+"\",\""+Integer.toHexString(Float.floatToRawIntBits(out[1]))+"\"]");
        }
        System.out.println("],");
        var sine = new float[65536];
        double scale = 10430.378350470453;
        for(int i=0;i<sine.length;i++)sine[i]=(float)Math.sin((double)i/scale);
        float[] angles={-180,-135,-90,-45,-30,0,30,45,90,135,180};
        System.out.print("\"silent\":[");
        for(int i=0;i<angles.length;i++) {
            if(i>0)System.out.print(",");
            float radians = angles[i] * Float.intBitsToFloat(0x3c8efa35);
            float sin=sine[(int)(((long)((double)radians*scale))&65535)];
            float cos=sine[(int)(((long)((double)radians*scale+16384.0))&65535)];
            System.out.print("{\"angle\":"+(int)angles[i]+",\"sinBits\":\""+Integer.toHexString(Float.floatToRawIntBits(sin))+"\",\"cosBits\":\""+Integer.toHexString(Float.floatToRawIntBits(cos))+"\",\"masks\":[");
            for(int flags=0;flags<16;flags++){if(flags>0)System.out.print(",");System.out.print(corrected(flags,sin,cos));}
            System.out.print("]}");
        }
        System.out.println("],");
        System.out.println("\"samePriorityTrace\":\""+priorityTrace()+"\"");
        System.out.println("}");
    }
}
