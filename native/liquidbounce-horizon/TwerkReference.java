// Source: ModuleTwerk ae0eb396, evaluated by actual Java signed Int arithmetic.
class TwerkReference {
    public static void main(String[] args) {
        int[] delays={Integer.MIN_VALUE,Integer.MIN_VALUE+1,-1073741825,-1073741824,-20,-2,-1,0,1,2,20,1073741824,Integer.MAX_VALUE};
        int[] ticks={Integer.MIN_VALUE,Integer.MIN_VALUE+1,-1073741825,-1073741824,-41,-21,-5,-4,-3,-2,-1,0,1,2,3,4,5,20,21,40,41,1073741824,Integer.MAX_VALUE-1,Integer.MAX_VALUE};
        System.out.print("[");boolean first=true;
        for(int delay:delays)for(int tick:ticks) {
            if(!first)System.out.print(",");first=false;
            System.out.print("["+delay+","+tick+",");
            try {System.out.print(tick%(delay*2)<delay);}catch(ArithmeticException e){System.out.print("null");}
            System.out.print("]");
        }
        System.out.println("]");
    }
}
