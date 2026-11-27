import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

// A standalone source-level ProjectileUtil oracle using the actual MC 26.3
// AABB/Vec3 methods. It does not initialize a game client or claim a full replay.
class ReachReference {
    record Candidate(int id, AABB box, double radius, boolean root, boolean inside) {}
    static Candidate c(int id,double x,double z,double radius,boolean root,boolean inside) {
        return new Candidate(id,new AABB(x-.25,-.25,z,x+.25,.25,z+.5),radius,root,inside);
    }
    static void bits(Vec3 v) {
        System.out.print("["+Long.toUnsignedString(Double.doubleToRawLongBits(v.x))+","+
            Long.toUnsignedString(Double.doubleToRawLongBits(v.y))+","+Long.toUnsignedString(Double.doubleToRawLongBits(v.z))+"]");
    }
    public static void main(String[] args) {
        float[][] rotations={{0,0},{37.123f,-21.456f},{123.456f,-54.321f},{180,90},{-180,-90},{720.125f,13.125f}};
        System.out.print("{\"directions\":[");
        for(int i=0;i<rotations.length;i++) {
            if(i>0)System.out.print(",");
            System.out.print("{\"yaw\":"+rotations[i][0]+",\"pitch\":"+rotations[i][1]+",\"bits\":");
            bits(Vec3.directionFromRotation(rotations[i][1],rotations[i][0]));System.out.print("}");
        }
        Candidate[][] cases={
            {c(1,0,4,0,false,true)},
            {c(1,0,1,0,false,true),c(2,0,3,0,false,true)},
            {c(1,2,1,3,false,true)},
            {c(1,0,-.1,0,true,true)},
            {c(1,0,1,0,true,true),c(2,0,3,0,false,true)},
            {c(1,0,-.1,0,false,false),c(2,0,3,0,false,true)},
            {c(1,0,-.1,0,false,true),c(2,0,3,0,false,true)},
            {c(1,0,1,0,false,true),c(2,0,1,0,false,true)}
        };
        Vec3 eye=Vec3.ZERO,end=new Vec3(0,0,4);
        AABB scan=new AABB(-.3,-1.6,-.3,.3,.2,.3).expandTowards(end).inflate(1,1,1);
        System.out.print("],\"cases\":[");
        for(int j=0;j<cases.length;j++) {
            if(j>0)System.out.print(",");
            System.out.print("{\"candidates\":[");
            Candidate best=null;Vec3 point=null;double remaining=16;
            for(int i=0;i<cases[j].length;i++) {
                Candidate c=cases[j][i];AABB b=c.box;
                if(i>0)System.out.print(",");
                System.out.print("["+c.id+","+b.minX+","+b.minY+","+b.minZ+","+b.maxX+","+b.maxY+","+b.maxZ+","+c.radius+","+c.root+","+c.inside+"]");
                if(!b.intersects(scan))continue;
                b=b.inflate(c.radius);var hit=b.clip(eye,end);
                if(b.contains(eye)) {
                    if(remaining>=0&&c.inside) {best=c;point=hit.orElse(eye);remaining=0;}
                } else if(hit.isPresent()) {
                    Vec3 at=hit.get();double d=eye.distanceToSqr(at);
                    if(d<remaining||remaining==0) {
                        if(c.root) {if(remaining==0) {best=c;point=at;}}
                        else {best=c;point=at;remaining=d;}
                    }
                }
            }
            System.out.print("],\"target\":"+(best==null?-1:best.id)+",\"pointBits\":");
            if(point==null)System.out.print("null");else bits(point);
            System.out.print("}");
        }
        System.out.println("]}");
    }
}
