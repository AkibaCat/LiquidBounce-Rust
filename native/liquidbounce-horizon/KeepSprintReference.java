import java.lang.reflect.Field;
import java.util.concurrent.ThreadLocalRandom;
import kotlin.random.Random;
import kotlin.random.RandomKt;
import kotlin.ranges.IntRange;

class KeepSprintReference {
    public static void main(String[] args) throws Exception {
        ThreadLocalRandom random = ThreadLocalRandom.current();
        Field seed = Thread.class.getDeclaredField("threadLocalRandomSeed");
        seed.setAccessible(true);
        seed.setLong(Thread.currentThread(), 123L);
        System.out.print("{\"seed\":123,\"threadId\":" + Thread.currentThread().threadId() + ",\"drawBits\":[");
        for (int i = 0; i < 16; i++) {
            if (i > 0) System.out.print(",");
            System.out.print(Integer.toUnsignedString(Float.floatToRawIntBits(random.nextFloat())));
        }
        System.out.print("],\"inclusive\":[");
        int[][] ranges = {{0,0},{4,8},{0,7},{Integer.MIN_VALUE,Integer.MAX_VALUE},
            {Integer.MIN_VALUE,Integer.MAX_VALUE-1},{Integer.MIN_VALUE,0},
            {0,1073741824},{Integer.MAX_VALUE,Integer.MAX_VALUE},{-71,Integer.MAX_VALUE}};
        for (int j=0;j<ranges.length;j++) {
            if(j>0)System.out.print(",");
            seed.setLong(Thread.currentThread(),123L);
            System.out.print("{\"first\":"+ranges[j][0]+",\"last\":"+ranges[j][1]+",\"values\":[");
            for(int i=0;i<64;i++) {
                if(i>0)System.out.print(",");
                System.out.print(RandomKt.nextInt(Random.Default,new IntRange(ranges[j][0],ranges[j][1])));
            }
            System.out.print("],\"next\":"+random.nextInt()+"}");
        }
        System.out.println("]}");
    }
}
