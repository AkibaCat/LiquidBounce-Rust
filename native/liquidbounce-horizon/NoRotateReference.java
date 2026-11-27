// SPDX-License-Identifier: GPL-3.0-or-later
// Pinned Linear/Sigmoid/AccelerationAngleSmooth expressions, using actual JDK
// bounded nextFloat and Kotlin primitive/range coerceIn overloads.
import java.util.random.RandomGenerator;
import kotlin.ranges.RangesKt;

class NoRotateReference {
    static class Units implements RandomGenerator {
        int index;
        final float[] values = {0F,.1F,.25F,.5F,.75F,.9F,.99999F,.33F,.67F};
        public long nextLong() { throw new AssertionError("Only nextFloat is expected"); }
        public float nextFloat() { return values[index++ % values.length]; }
        float range(float a,float b) { return a>=b ? a : nextFloat(a,Math.nextUp(b)); }
    }
    static float wrap(float value) {
        value %= 360F;
        if(value>=180F)value-=360F;
        if(value< -180F)value+=360F;
        return value;
    }
    static float[] delta(float[] a,float[] b){return new float[]{wrap(b[0]-a[0]),b[1]-a[1]};}
    static float length(float[] d){return (float)Math.sqrt(d[0]*d[0]+d[1]*d[1]);}
    static double sigmoid(float difference,float steepness,float midpoint){return 1.0/(1.0+Math.exp((double)(-steepness*(difference/120F-midpoint))));}
    static float[] process(int kind,float[] current,float[] target,float[] previous,Units random) {
        var diff=delta(current,target);
        if(kind<2) {
            float horizontal=random.range(13F,79F),vertical=random.range(3F,45F);
            if(kind==1) {
                double factor=sigmoid(Math.min(length(diff),180F),7.3F,.24F);
                horizontal=RangesKt.coerceIn((float)(factor*horizontal),0F,180F);
                vertical=RangesKt.coerceIn((float)(factor*vertical),0F,180F);
            }
            float len=length(diff);
            float h=Math.abs(diff[0]/len)*horizontal,v=Math.abs(diff[1]/len)*vertical;
            return new float[]{current[0]+RangesKt.coerceIn(diff[0],-h,h),current[1]+RangesKt.coerceIn(diff[1],-v,v)};
        }
        var prev=delta(previous,current);
        float decel=(float)sigmoid(length(diff),10F,.3F);
        decel=kind==3?RangesKt.coerceIn(decel,0F,180F):1F;
        float distance=(float)(-1.393F*0.0);
        float loYaw=-random.range(20F,25F)+distance,hiYaw=random.range(20F,25F)+distance;
        float loPitch=-random.range(20F,25F)+distance,hiPitch=random.range(20F,25F)+distance;
        float ay=RangesKt.coerceIn(Float.valueOf(wrap(diff[0]-prev[0])),RangesKt.rangeTo(loYaw,hiYaw))*decel;
        float ap=RangesKt.coerceIn(Float.valueOf(wrap(diff[1]-prev[1])),RangesKt.rangeTo(loPitch,hiPitch))*decel;
        float ey=ay*random.range(-.1F,.1F)+random.range(-.1F,.1F);
        float ep=ap*random.range(-.1F,.1F)+random.range(-.1F,.1F);
        return new float[]{current[0]+(prev[0]+ay+ey),current[1]+(prev[1]+ap+ep)};
    }
    static String bits(float[] value){return "["+Integer.toUnsignedString(Float.floatToRawIntBits(value[0]))+","+Integer.toUnsignedString(Float.floatToRawIntBits(value[1]))+"]";}
    public static void main(String[] args) {
        System.out.print("{\"steps\":[");
        for(int kind=0;kind<4;kind++) {
            if(kind>0)System.out.print(",");
            var current=new float[]{179F,-88F};var previous=new float[]{170F,-85F};var target=new float[]{-179F,88F};var random=new Units();
            System.out.print("[");
            for(int tick=0;tick<5;tick++) {
                if(tick>0)System.out.print(",");
                var next=process(kind,current,target,previous,random);
                System.out.print(bits(next));previous=current;current=next;
            }
            System.out.print("]");
        }
        System.out.print("],\"rangeNaNThrows\":");
        try { RangesKt.coerceIn(Float.valueOf(1F),RangesKt.rangeTo(Float.NaN,Float.NaN));System.out.print("false"); }
        catch(IllegalArgumentException error){System.out.print("true");}
        System.out.println("}");
    }
}
