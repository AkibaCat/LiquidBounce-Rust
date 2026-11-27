"""Live AbilityCapture getter order, immutable tool data and BreakTimes formulas."""
import itertools

def generate(add,new,state,player,at,b,bits,raw,pos,fbits,rng,step):
 methods=['AUTO','NINJA','GODBRIDGE','MOONWALK','BREEZILY','ONE_STACK','SWIFT_SNEAK','TELLY']
 def opts(mask=7,fall=3,safe=3.,health=20.,climbs=True,method='AUTO',allowed=True):
  add('capopts',mask,fall,bits(safe),fbits(health),b(climbs),method,b(allowed))
 def setup():
  new();add('targetnew');opts();add('capsets','1,-2,99','8,12','1024,-2048')
 def stack(slot,count=1,empty=False,speed=1.,eff=0,correct=False,place=False):
  add('targetstack',slot,'false',b(correct),fbits(speed),eff,b(place),'false');add('capstack',slot,count,b(empty))
 # Every gate and bridge-cost branch, both mechanics and snapshot retention.
 for case in range(256):
  setup();mask=case%64;method=methods[(case//8)%8]
  opts(mask,fall=rng.choice([-10,0,3,25,2**31-1]),safe=rng.choice([-1.,0.,2.999,3.,3.999,4.,10.5,1e30,float('nan')]),health=rng.choice([0.,.5,20.]),climbs=bool(case&4),method=method,allowed=bool(case&1))
  for slot in [*range(9),40]:
   stack(slot,count=rng.choice([0,1,8,64,-2,2**31-1]),empty=rng.choice([False,True]),speed=rng.choice([0.,1.,1.001,2.,16.,float('inf')]),eff=rng.choice([0,1,2,32,32768,46341,65536,-2]),correct=rng.choice([False,True]),place=rng.choice([False,True]))
  player(case,speed=.13,flags=3 if case&1 else 1,food=case%10,air=case,boost=.2 if case&2 else 0)
  add('capsafe');add('caphot');add('capwalk')
  add('capsets','88','999','1');add('capquery')
  for hardness,requires in itertools.product([-1.,-0.,.5,1.,50.,float('nan'),float('inf')],[False,True]):
   add('caphard',1,fbits(hardness),b(requires));add('capticks',1)
  # Replacing inventory stacks must leave copied tools intact; break policy is
  # a retained callback, so its current value is observed at query time.
  stack(0,64,False,100.,50000,True,True);add('capticks',1)
  opts(mask,method=method,allowed=not bool(case&1));add('capticks',1);add('capquery')
 # Direct capture has one filtering pass; context.captureWalk has two.
 for hardness,speed,eff,correct,requires in itertools.product([0.,.1,1.,-1.],[0.,1.,2.,16.],[0,2,32,46341],[False,True],[False,True]):
  setup();stack(0,1,False,speed,eff,correct);add('caphard',1,fbits(hardness),b(requires));add('capbreak',1)
