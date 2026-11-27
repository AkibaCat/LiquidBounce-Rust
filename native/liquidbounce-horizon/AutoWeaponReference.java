// SPDX-License-Identifier: GPL-3.0-or-later
// Direct JVM arithmetic oracle of ae0eb396 WeaponItemFacet/MaceItemFacet.
import java.util.Random;
public class AutoWeaponReference {
    static float weighted(int[] e, int[] indexes, float[] weights) {
        float value=0.0F;
        for(int i=0;i<indexes.length;i++) value += e[indexes[i]]*weights[i];
        return value;
    }
    static double damage(double damage,double speed,int[] e,boolean legacy) {
        double p=Math.pow(.85,1/20.0), probability=Math.pow(p,Math.ceil(20.0/speed*.9));
        double adjusted=legacy?damage:damage*speed*(float)probability;
        float fire=Math.max(e[1]*4.0F-1,0.0F)*.33F;
        float additional=weighted(e,new int[]{2,3,4},new float[]{2.0F*.1F,2.0F*.1F,.2F});
        return adjusted*(1.0+additional)+fire;
    }
    static double mace(double damage,double speed,int[] e) {
        float probability=(float)Math.pow(.9919069797821398,Math.ceil(20.0/speed*.9));
        double adjusted=damage*speed*probability;
        float additional=weighted(e,new int[]{10,11,2,3,12},new float[]{.5F,.15F,2.0F*.1F,2.0F*.1F,.2F});
        return adjusted+additional+29.0F;
    }
    static float secondary(int[] e) {
        return weighted(e,new int[]{5,6,7,8,9,4},new float[]{.05F,.05F,.1F,-.1F,.2F,.25F});
    }
    static String bits(double value){return "\""+Long.toUnsignedString(Double.doubleToLongBits(value))+"\"";}
    public static void main(String[] args) {
        var random=new Random(0xAE0EB396L);
        double[] speeds={0.0,-0.0,.1,1,1.6,4,20,1024,Double.POSITIVE_INFINITY,Double.NEGATIVE_INFINITY,Double.NaN,-1};
        StringBuilder out=new StringBuilder("[");
        for(int i=0;i<384;i++) {
            if(i>0)out.append(',');
            double damage=i<speeds.length?1.0:(random.nextInt(2049)-1024)/8.0;
            double speed=speeds[i%speeds.length];
            int[] e=new int[13];
            for(int j=0;j<e.length;j++)e[j]=i<speeds.length?0:random.nextInt(1<<14)-32;
            out.append("{\"damage\":").append(bits(damage)).append(",\"speed\":").append(bits(speed)).append(",\"e\":[");
            for(int j=0;j<e.length;j++){if(j>0)out.append(',');out.append(e[j]);}
            out.append("],\"modern\":").append(bits(damage(damage,speed,e,false)))
                .append(",\"legacy\":").append(bits(damage(damage,speed,e,true)))
                .append(",\"mace\":").append(bits(mace(damage,speed,e)))
                .append(",\"secondary\":").append(Integer.toUnsignedString(Float.floatToIntBits(secondary(e)))).append('}');
        }
        System.out.println(out.append(']'));
    }
}
