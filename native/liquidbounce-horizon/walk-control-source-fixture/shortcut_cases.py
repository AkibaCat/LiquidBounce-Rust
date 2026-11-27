"""StraightWalk and Shortcut use unmodified source and real Terrain caches."""
import itertools

def generate(add,new,state,player,at,b,bits,raw,pos,fbits,rng,step):
 state(14,'fire',0,32)
 scenes=['flat','wall','slab','gap','deep','water','fire','unknown','border','ceiling','diagonal']
 for scene,length,fall in itertools.product(scenes,[0.,.1,.49,.5,.51,1.,2.4,5.5],[0,1,3,5]):
  new(floor=-7 if scene in ['gap','deep','water','fire']else -1)
  if scene in ['gap','deep','water','fire']:
   for x in range(-2,8):
    for z in range(-2,3):at(x,-1 if x!=2 else-3 if scene=='gap'else-7,z,1)
   if scene in ['water','fire']:at(2,-1,0,3 if scene=='water'else 14)
  if scene=='wall':at(2,0,0,1);at(2,1,0,1)
  if scene=='slab':at(2,0,0,5);at(3,0,0,5)
  if scene=='ceiling':at(2,1,0,1)
  if scene=='unknown':add('known','2,0,1','false')
  if scene=='border':add('border',bits(-2),bits(3))
  start=(.5,0,.5);end=(.5+length,0,.5+length if scene=='diagonal'else.5)
  for stop in [-1.,0.,.1,.25,.5,1.]:add('straightwalk',raw(start),raw(end),bits(stop),fall)
  # Retained cache calls intentionally repeat after the same source terrain.
  add('straightwalk',raw(start),raw(end),bits(.25),fall)
 for _ in range(180):
  new();start=(rng.uniform(-1,1),rng.choice([0.,.00001,-.1,.1,.5]),rng.uniform(-1,1));end=(rng.uniform(-5,5),rng.choice([0.,.25,.5,1.]),rng.uniform(-5,5))
  for i in range(7):at(rng.randrange(-3,4),rng.choice([-1,0,1]),rng.randrange(-3,4),rng.choice([0,1,3,5,7,14]))
  add('straightwalk',raw(start),raw(end),bits(rng.choice([0.,.1,.25,.5,1.])),rng.randrange(-1,6))
 # Full route bounds, plain-step stopping, turns, obstruction and candidate
 # search from farthest back. Original abilities.safeFall stays at 3 here.
 for scene,break_at in itertools.product(['flat','turn','wall','gap','unknown'],[-1,1,3,7,9]):
  new()
  for i in range(11):
   x=i if scene!='turn'else min(i,4);z=0 if scene!='turn'else max(0,i-4)
   step(i,(x,0,z),(x+1 if scene!='turn'or i<4 else x,0,z if scene!='turn'or i<4 else z+1),flags=1 if i==break_at else 0)
  add('route',raw((.5,0,.5)),','.join(map(str,range(11))))
  if scene=='wall':at(3,0,0,1)
  if scene=='gap':at(3,-1,0,0);at(3,-1,1,0)
  if scene=='unknown':add('known','3,0,1','false')
  for index in [0,1,3,7,9,10,11]:
   add('shortcut',index,raw((.5+index,0,.5)));add('shortcut',index,raw((.7+index,.05,.9)))
