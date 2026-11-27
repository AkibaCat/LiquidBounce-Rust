// Development oracle only. The runtime is Rust and never starts Java.
import java.lang.reflect.Field;
import java.util.concurrent.ThreadLocalRandom;

class AntiAfkRandomReference {
    public static void main(String[] args) throws Exception {
        ThreadLocalRandom random = ThreadLocalRandom.current();
        Field seedField = Thread.class.getDeclaredField("threadLocalRandomSeed");
        seedField.setAccessible(true);
        long[] seeds = {0, 123, -1, 0x123456789abcdef0L};
        System.out.print("[");
        for (int s=0;s<seeds.length;s++) {
            if(s!=0) System.out.print(",");
            seedField.setLong(Thread.currentThread(),seeds[s]);
            System.out.print("{\"seed\":"+Long.toUnsignedString(seeds[s])+",\"thread\":"+Thread.currentThread().threadId()+",\"draws\":[");
            for(int n=0;n<64;n++) {
                long value=random.nextLong(); boolean b=random.nextBoolean();
                long bits=Double.doubleToRawLongBits(random.nextDouble(-5.0,5.0)); int slot=random.nextInt(0,9);
                int bound=1+(n%6),index=random.nextInt(bound);
                if(n!=0) System.out.print(",");
                System.out.print("{\"long\":\""+value+"\",\"boolean\":"+b+",\"doubleBits\":\""+Long.toUnsignedString(bits)+"\",\"slot\":"+slot+",\"bound\":"+bound+",\"index\":"+index+"}");
            }
            System.out.print("]}");
        }
        System.out.println("]");
    }
}
