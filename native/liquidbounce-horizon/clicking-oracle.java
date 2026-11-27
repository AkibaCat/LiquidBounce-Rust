// SPDX-License-Identifier: GPL-3.0-or-later
// One-second upstream HumanClickTiming/ClickPlan vector, seed 0.
import java.util.*;
class ClickingOracle {
    public static void main(String[] args) {
        Random r=new Random(0);
        double cps=11+r.nextDouble()*3;
        ArrayList<Long> times=new ArrayList<>(); times.add(0L);
        while(times.get(times.size()-1)<=1000) {
            double mu=Math.log(1000/cps)-.45*.45/2;
            long step=Math.max(10,Math.min(1000,Math.round(Math.exp(mu+.45*r.nextGaussian()))));
            long next=times.get(times.size()-1)+step;
            if(times.size()>=2)next=Math.max(next,times.get(times.size()-2)+50);
            times.add(next);
        }
        ArrayList<Integer> counts=new ArrayList<>(); counts.add(1);
        for(int tick=1;tick<20;tick++) {
            long from=(tick-1)*50,to=from+50;
            int count=0;for(long time:times)if(time>from&&time<=to)count++;
            counts.add(Math.min(count,2));
        }
        System.out.println("human="+counts);
    }
}
